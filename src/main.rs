//! Left Command sends the native macOS Eisu key; Right Command sends Kana.
//! A Command key only triggers when pressed and released on its own.

use std::ffi::{c_char, c_int, c_void, CString};
use std::fs::{File, OpenOptions};
use std::os::fd::AsRawFd;
use std::path::PathBuf;
use std::ptr;
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant};

mod input;
use input::{hiragana_length, roman_text, Action, InputState, RecordedKey, Side};

type CFTypeRef = *const c_void;
type CFStringRef = *const c_void;
type CFIndex = isize;
type CGEventTapProxy = *const c_void;
type CGEventRef = *const c_void;
type CFMachPortRef = *const c_void;
type CFRunLoopSourceRef = *const c_void;
type CFRunLoopRef = *const c_void;

const FLAGS_CHANGED: u32 = 12;
const KEY_DOWN: u32 = 10;
const LEFT_MOUSE_DOWN: u32 = 1;
const RIGHT_MOUSE_DOWN: u32 = 3;
const TAP_DISABLED_TIMEOUT: u32 = 0xffff_fffe;
const TAP_DISABLED_USER_INPUT: u32 = 0xffff_ffff;
const SHIFT_FLAG: u64 = 1 << 17;
const CONTROL_FLAG: u64 = 1 << 18;
const OPTION_FLAG: u64 = 1 << 19;
const COMMAND_FLAG: u64 = 1 << 20;
const JIS_EISU: u16 = 102;
const JIS_KANA: u16 = 104;
const SYNTHETIC_MARKER: i64 = 0x0043_4d44_4b41_4e41;
const LOCK_EX: c_int = 2;
const LOCK_NB: c_int = 4;

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn CGEventTapCreate(
        tap: u32,
        place: u32,
        options: u32,
        events: u64,
        callback: extern "C" fn(CGEventTapProxy, u32, CGEventRef, *mut c_void) -> CGEventRef,
        user_info: *mut c_void,
    ) -> CFMachPortRef;
    fn CGEventGetIntegerValueField(event: CGEventRef, field: u32) -> i64;
    fn CGEventSetIntegerValueField(event: CGEventRef, field: u32, value: i64);
    fn CGEventGetFlags(event: CGEventRef) -> u64;
    fn CGEventSetFlags(event: CGEventRef, flags: u64);
    fn CGEventCreateKeyboardEvent(source: CFTypeRef, keycode: u16, down: bool) -> CGEventRef;
    fn CGEventPost(tap: u32, event: CGEventRef);
    fn CGEventTapEnable(tap: CFMachPortRef, enable: bool);
    fn CGRequestListenEventAccess() -> bool;
    fn CGRequestPostEventAccess() -> bool;
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    static kCFRunLoopCommonModes: CFStringRef;
    fn CFMachPortCreateRunLoopSource(
        allocator: CFTypeRef,
        port: CFMachPortRef,
        order: CFIndex,
    ) -> CFRunLoopSourceRef;
    fn CFRunLoopGetCurrent() -> CFRunLoopRef;
    fn CFRunLoopAddSource(run_loop: CFRunLoopRef, source: CFRunLoopSourceRef, mode: CFStringRef);
    fn CFRelease(value: CFTypeRef);
}

extern "C" {
    fn run_menu_bar_app();
    fn replace_previous_text(length: CFIndex, replacement_utf8: *const c_char) -> bool;
    fn uses_native_english_conversion() -> bool;
    fn getuid() -> u32;
    fn kill(pid: c_int, signal: c_int) -> c_int;
    fn flock(fd: c_int, operation: c_int) -> c_int;
}

struct State {
    input: InputState,
    tap: Option<CFMachPortRef>,
}

// CoreFoundation references are process-global and used by the app run loop.
unsafe impl Send for State {}
static STATE: OnceLock<Mutex<State>> = OnceLock::new();
static STARTED_AT: OnceLock<Instant> = OnceLock::new();
static INSTANCE_LOCK: OnceLock<File> = OnceLock::new();

fn wait_for_previous_instance() {
    let mut args = std::env::args();
    while let Some(arg) = args.next() {
        if arg == "--relaunch-after-pid" {
            if let Some(pid) = args.next().and_then(|value| value.parse::<c_int>().ok()) {
                let deadline = Instant::now() + Duration::from_secs(5);
                while unsafe { kill(pid, 0) } == 0 && Instant::now() < deadline {
                    thread::sleep(Duration::from_millis(25));
                }
            }
            break;
        }
    }
}

fn acquire_instance_lock() -> bool {
    let path: PathBuf = std::env::temp_dir().join(format!("cmd-kana-{}.lock", unsafe { getuid() }));
    let Ok(file) = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .truncate(false)
        .open(path)
    else {
        return false;
    };
    if unsafe { flock(file.as_raw_fd(), LOCK_EX | LOCK_NB) } != 0 {
        return false;
    }
    INSTANCE_LOCK.set(file).is_ok()
}

#[no_mangle]
pub extern "C" fn is_event_monitoring() -> bool {
    STATE
        .get()
        .is_some_and(|state| state.lock().unwrap().tap.is_some())
}

#[no_mangle]
pub extern "C" fn start_event_monitoring() -> bool {
    let Some(state) = STATE.get() else {
        return false;
    };
    let mut state = state.lock().unwrap();
    if state.tap.is_some() {
        return true;
    }
    let mask = (1u64 << FLAGS_CHANGED)
        | (1u64 << KEY_DOWN)
        | (1u64 << LEFT_MOUSE_DOWN)
        | (1u64 << RIGHT_MOUSE_DOWN);
    // Match cmd-eikana's proven architecture: session tap at head insert.
    let tap = unsafe { CGEventTapCreate(1, 0, 0, mask, event_callback, ptr::null_mut()) };
    if tap.is_null() {
        return false;
    }
    unsafe {
        let source = CFMachPortCreateRunLoopSource(ptr::null(), tap, 0);
        CFRunLoopAddSource(CFRunLoopGetCurrent(), source, kCFRunLoopCommonModes);
        CFRelease(source);
        CGEventTapEnable(tap, true);
    }
    state.input.clear();
    state.tap = Some(tap);
    eprintln!("cmd-kana: 入力監視を開始しました");
    true
}

unsafe fn post_key(keycode: u16, flags: u64) {
    for down in [true, false] {
        let event = CGEventCreateKeyboardEvent(ptr::null(), keycode, down);
        if event.is_null() {
            continue;
        }
        CGEventSetFlags(event, flags);
        CGEventSetIntegerValueField(event, 42, SYNTHETIC_MARKER);
        CGEventPost(0, event); // kCGHIDEventTap
        CFRelease(event);
    }
}

unsafe fn switch_to(side: Side) {
    post_key(
        match side {
            Side::English => JIS_EISU,
            Side::Japanese => JIS_KANA,
        },
        0,
    );
}

unsafe fn replace_with_english(keys: Vec<RecordedKey>) {
    let Some(rendered_length) = hiragana_length(&keys) else {
        return;
    };
    let Some(roman) = roman_text(&keys).and_then(|text| CString::new(text).ok()) else {
        return;
    };
    if replace_previous_text(rendered_length as CFIndex, roman.as_ptr()) {
        switch_to(Side::English);
        return;
    }
    // Legacy fallback for non-Google controls without AXSelectedText support.
    // Google/Mozc marked text must use NativeEnglishDoubleTap instead.
    for _ in 0..rendered_length {
        post_key(51, 0); // Delete
    }
    for key in keys {
        post_key(key.keycode, if key.shift { SHIFT_FLAG } else { 0 });
    }
}

extern "C" fn event_callback(
    _: CGEventTapProxy,
    event_type: u32,
    event: CGEventRef,
    _: *mut c_void,
) -> CGEventRef {
    unsafe {
        if event_type == TAP_DISABLED_TIMEOUT || event_type == TAP_DISABLED_USER_INPUT {
            if let Some(state) = STATE.get() {
                let mut state = state.lock().unwrap();
                // Releases and composition boundaries may have been missed.
                state.input = InputState::default();
                if let Some(tap) = state.tap {
                    CGEventTapEnable(tap, true);
                }
            }
            return event;
        }
        if CGEventGetIntegerValueField(event, 42) == SYNTHETIC_MARKER {
            return event;
        }

        let keycode = CGEventGetIntegerValueField(event, 9);
        let mut action = None;
        if let Some(mutex) = STATE.get() {
            let mut state = mutex.lock().unwrap();
            match event_type {
                FLAGS_CHANGED => {
                    let flags = CGEventGetFlags(event);
                    if keycode == input::LEFT_COMMAND || keycode == input::RIGHT_COMMAND {
                        state
                            .input
                            .set_native_english_conversion(uses_native_english_conversion());
                    }
                    let now_ms = STARTED_AT.get().unwrap().elapsed().as_millis() as u64;
                    action = state.input.flags_changed(
                        keycode,
                        now_ms,
                        flags & COMMAND_FLAG != 0,
                        flags & (SHIFT_FLAG | CONTROL_FLAG | OPTION_FLAG) != 0,
                    );
                }
                KEY_DOWN => {
                    let flags = CGEventGetFlags(event);
                    state.input.key_down(
                        keycode,
                        flags & SHIFT_FLAG != 0,
                        flags & (COMMAND_FLAG | CONTROL_FLAG | OPTION_FLAG) != 0,
                    );
                }
                LEFT_MOUSE_DOWN | RIGHT_MOUSE_DOWN => state.input.clear(),
                _ => {}
            }
        }

        match action {
            Some(Action::Switch(side)) => switch_to(side),
            Some(Action::ReplaceWithEnglish(keys)) => replace_with_english(keys),
            Some(Action::NativeEnglishDoubleTap) => {
                // Google's native double-tap window is 500 ms, while ours is
                // 550 ms. A fresh tight pair also handles the 500–550 ms edge.
                // Do not edit or delete IME-owned marked text through AX.
                switch_to(Side::English);
                switch_to(Side::English);
            }
            None => {}
        }
        event
    }
}

fn main() {
    wait_for_previous_instance();
    if !acquire_instance_lock() {
        eprintln!("cmd-kana: すでに起動しています");
        return;
    }
    unsafe {
        STATE
            .set(Mutex::new(State {
                input: InputState::default(),
                tap: None,
            }))
            .ok();
        STARTED_AT.set(Instant::now()).ok();
        CGRequestListenEventAccess();
        CGRequestPostEventAccess();
        if !start_event_monitoring() {
            eprintln!("cmd-kana: 入力監視を開始できません。入力監視とアクセシビリティを許可してください。");
        }
        eprintln!("cmd-kana: 起動しました（左⌘: 英数 / 右⌘: かな）");
        run_menu_bar_app();
    }
}
