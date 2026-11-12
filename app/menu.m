#import <Cocoa/Cocoa.h>
#import <ApplicationServices/ApplicationServices.h>
#import <CoreServices/CoreServices.h>
#import <ServiceManagement/ServiceManagement.h>
#import <Carbon/Carbon.h>

extern bool start_event_monitoring(void);
extern bool is_event_monitoring(void);

bool uses_native_english_conversion(void) {
    TISInputSourceRef source = TISCopyCurrentKeyboardInputSource();
    if (!source) return false;
    CFTypeRef identifier = TISGetInputSourceProperty(source, kTISPropertyInputSourceID);
    bool native = identifier && CFGetTypeID(identifier) == CFStringGetTypeID() &&
        (CFEqual(identifier, CFSTR("com.google.inputmethod.Japanese")) ||
         CFEqual(identifier, CFSTR("org.mozc.inputmethod.Japanese")) ||
         CFStringHasPrefix((CFStringRef)identifier, CFSTR("com.google.inputmethod.Japanese.")) ||
         CFStringHasPrefix((CFStringRef)identifier, CFSTR("org.mozc.inputmethod.Japanese.")));
    CFRelease(source);
    return native;
}

@interface CmdKanaDelegate : NSObject <NSApplicationDelegate>
@property(strong) NSStatusItem *statusItem;
@property(strong) NSWindow *settingsWindow;
@property(strong) NSButton *menuBarCheckbox;
@property(strong) NSButton *loginItemCheckbox;
@property(strong) NSTextField *loginItemStatusLabel;
@property(strong) NSTextField *listenStatusLabel;
@property(strong) NSTextField *postStatusLabel;
@property(strong) NSTimer *monitoringRetryTimer;
@end

bool replace_previous_text(CFIndex length, const char *replacementUTF8) {
    @autoreleasepool {
        if (length <= 0 || !replacementUTF8) return false;
        AXUIElementRef system = AXUIElementCreateSystemWide();
        AXUIElementRef focused = NULL;
        CFTypeRef rangeRef = NULL;
        bool replaced = false;

        if (AXUIElementCopyAttributeValue(system, kAXFocusedUIElementAttribute,
                (CFTypeRef *)&focused) == kAXErrorSuccess && focused &&
            AXUIElementCopyAttributeValue(focused, kAXSelectedTextRangeAttribute,
                &rangeRef) == kAXErrorSuccess && rangeRef &&
            CFGetTypeID(rangeRef) == AXValueGetTypeID()) {
            CFRange original;
            if (AXValueGetValue((AXValueRef)rangeRef, kAXValueCFRangeType, &original) &&
                original.length == 0 && original.location >= length) {
                CFRange target = CFRangeMake(original.location - length, length);
                AXValueRef targetRef = AXValueCreate(kAXValueCFRangeType, &target);
                NSString *replacement = [NSString stringWithUTF8String:replacementUTF8];
                if (replacement && AXUIElementSetAttributeValue(focused,
                        kAXSelectedTextRangeAttribute, targetRef) == kAXErrorSuccess) {
                    replaced = AXUIElementSetAttributeValue(focused,
                        kAXSelectedTextAttribute,
                        (__bridge CFStringRef)replacement) == kAXErrorSuccess;
                    if (!replaced) {
                        AXValueRef originalRef = AXValueCreate(kAXValueCFRangeType, &original);
                        AXUIElementSetAttributeValue(focused,
                            kAXSelectedTextRangeAttribute, originalRef);
                        CFRelease(originalRef);
                    }
                }
                CFRelease(targetRef);
            }
        }
        if (rangeRef) CFRelease(rangeRef);
        if (focused) CFRelease(focused);
        CFRelease(system);
        return replaced;
    }
}

@implementation CmdKanaDelegate

- (BOOL)wasLaunchedAsLoginItem {
    NSAppleEventDescriptor *event = NSAppleEventManager.sharedAppleEventManager.currentAppleEvent;
    return [event descriptorForKeyword:keyAELaunchedAsLogInItem] != nil;
}

- (void)applicationDidFinishLaunching:(NSNotification *)notification {
    [self createSettingsWindow];
    NSNumber *saved = [[NSUserDefaults standardUserDefaults] objectForKey:@"ShowMenuBar"];
    self.menuBarCheckbox.state = !saved || saved.boolValue ? NSControlStateValueOn : NSControlStateValueOff;
    [self applyMenuBarVisibility];
    [self updateLoginItemStatus];
    self.monitoringRetryTimer = [NSTimer scheduledTimerWithTimeInterval:2.0
        target:self selector:@selector(retryEventMonitoring:)
        userInfo:nil repeats:YES];
    if (![self wasLaunchedAsLoginItem]) [self showSettings:nil];
}

- (NSTextField *)labelWithText:(NSString *)text frame:(NSRect)frame {
    NSTextField *label = [[NSTextField alloc] initWithFrame:frame];
    label.stringValue = text;
    label.editable = NO;
    label.selectable = NO;
    label.bezeled = NO;
    label.drawsBackground = NO;
    return label;
}

- (NSButton *)buttonWithTitle:(NSString *)title action:(SEL)action frame:(NSRect)frame {
    NSButton *button = [[NSButton alloc] initWithFrame:frame];
    button.title = title;
    button.bezelStyle = NSBezelStyleRounded;
    button.target = self;
    button.action = action;
    return button;
}

- (void)createSettingsWindow {
    self.settingsWindow = [[NSWindow alloc]
        initWithContentRect:NSMakeRect(0, 0, 460, 340)
        styleMask:NSWindowStyleMaskTitled | NSWindowStyleMaskClosable
        backing:NSBackingStoreBuffered defer:NO];
    self.settingsWindow.title = @"cmd-kana 設定";
    self.settingsWindow.releasedWhenClosed = NO;
    [self.settingsWindow center];
    NSView *content = self.settingsWindow.contentView;

    NSTextField *title = [self labelWithText:@"cmd-kana" frame:NSMakeRect(24, 292, 300, 28)];
    title.font = [NSFont boldSystemFontOfSize:20];
    [content addSubview:title];

    self.menuBarCheckbox = [[NSButton alloc] initWithFrame:NSMakeRect(24, 250, 260, 24)];
    self.menuBarCheckbox.buttonType = NSSwitchButton;
    self.menuBarCheckbox.title = @"メニューバーに表示する";
    self.menuBarCheckbox.target = self;
    self.menuBarCheckbox.action = @selector(toggleMenuBar:);
    [content addSubview:self.menuBarCheckbox];

    self.loginItemCheckbox = [[NSButton alloc] initWithFrame:NSMakeRect(24, 216, 210, 24)];
    self.loginItemCheckbox.buttonType = NSSwitchButton;
    self.loginItemCheckbox.title = @"ログイン時に起動する";
    self.loginItemCheckbox.target = self;
    self.loginItemCheckbox.action = @selector(toggleLoginItem:);
    [content addSubview:self.loginItemCheckbox];

    self.loginItemStatusLabel = [self labelWithText:@"" frame:NSMakeRect(250, 216, 185, 24)];
    self.loginItemStatusLabel.alignment = NSTextAlignmentRight;
    self.loginItemStatusLabel.textColor = [NSColor secondaryLabelColor];
    [content addSubview:self.loginItemStatusLabel];

    NSTextField *permissions = [self labelWithText:@"アクセス許可" frame:NSMakeRect(24, 172, 200, 24)];
    permissions.font = [NSFont boldSystemFontOfSize:13];
    [content addSubview:permissions];

    self.listenStatusLabel = [self labelWithText:@"" frame:NSMakeRect(24, 137, 260, 24)];
    [content addSubview:self.listenStatusLabel];
    [content addSubview:[self buttonWithTitle:@"入力監視を開く"
        action:@selector(openInputMonitoring:) frame:NSMakeRect(300, 133, 135, 30)]];

    self.postStatusLabel = [self labelWithText:@"" frame:NSMakeRect(24, 100, 260, 24)];
    [content addSubview:self.postStatusLabel];
    [content addSubview:[self buttonWithTitle:@"アクセシビリティを開く"
        action:@selector(openAccessibility:) frame:NSMakeRect(280, 96, 155, 30)]];
    [content addSubview:[self buttonWithTitle:@"動かない場合"
        action:@selector(showPermissionHelp:) frame:NSMakeRect(24, 69, 115, 28)]];

    NSString *version = [[NSBundle mainBundle]
        objectForInfoDictionaryKey:@"CFBundleShortVersionString"] ?: @"不明";
    [content addSubview:[self labelWithText:
        [NSString stringWithFormat:@"バージョン %@", version]
        frame:NSMakeRect(24, 42, 220, 24)]];
    [content addSubview:[self buttonWithTitle:@"アプリを更新"
        action:@selector(updateApp:) frame:NSMakeRect(195, 32, 115, 32)]];
    [content addSubview:[self buttonWithTitle:@"アプリを終了"
        action:@selector(quitApp:) frame:NSMakeRect(320, 32, 115, 32)]];
    [self updatePermissionStatus];
}

- (void)createStatusItem {
    if (self.statusItem) return;
    self.statusItem = [[NSStatusBar systemStatusBar]
        statusItemWithLength:NSSquareStatusItemLength];

    NSImage *image = [NSApp applicationIconImage];
    image.size = NSMakeSize(19, 19);
    self.statusItem.button.image = image;
    self.statusItem.button.imageScaling = NSImageScaleProportionallyUpOrDown;
    self.statusItem.button.toolTip = @"cmd-kana（単押し: 切替 / 左⌘2回: 直前の語を英数化）";

    NSMenu *menu = [[NSMenu alloc] init];
    NSString *version = [[NSBundle mainBundle]
        objectForInfoDictionaryKey:@"CFBundleShortVersionString"] ?: @"不明";
    NSMenuItem *versionItem = [[NSMenuItem alloc]
        initWithTitle:[NSString stringWithFormat:@"バージョン %@", version]
        action:nil keyEquivalent:@""];
    versionItem.enabled = NO;
    [menu addItem:versionItem];


    [menu addItem:[NSMenuItem separatorItem]];

    NSMenuItem *settings = [[NSMenuItem alloc]
        initWithTitle:@"設定を開く" action:@selector(showSettings:) keyEquivalent:@","];
    settings.target = self;
    [menu addItem:settings];

    NSMenuItem *update = [[NSMenuItem alloc]
        initWithTitle:@"更新" action:@selector(updateApp:) keyEquivalent:@"r"];
    update.target = self;
    [menu addItem:update];

    NSMenuItem *quit = [[NSMenuItem alloc]
        initWithTitle:@"閉じる" action:@selector(quitApp:) keyEquivalent:@"q"];
    quit.target = self;
    [menu addItem:quit];
    self.statusItem.menu = menu;
}

- (void)applyMenuBarVisibility {
    if (self.menuBarCheckbox.state == NSControlStateValueOn) {
        [self createStatusItem];
    } else if (self.statusItem) {
        [[NSStatusBar systemStatusBar] removeStatusItem:self.statusItem];
        self.statusItem = nil;
    }
}

- (void)toggleMenuBar:(id)sender {
    BOOL visible = self.menuBarCheckbox.state == NSControlStateValueOn;
    [[NSUserDefaults standardUserDefaults] setBool:visible forKey:@"ShowMenuBar"];
    [self applyMenuBarVisibility];
}

- (void)updateLoginItemStatus {
    if (@available(macOS 13.0, *)) {
        SMAppServiceStatus status = SMAppService.mainAppService.status;
        self.loginItemCheckbox.enabled = YES;
        self.loginItemCheckbox.state = status == SMAppServiceStatusEnabled
            ? NSControlStateValueOn : NSControlStateValueOff;
        switch (status) {
            case SMAppServiceStatusEnabled:
                self.loginItemStatusLabel.stringValue = @"オン";
                break;
            case SMAppServiceStatusRequiresApproval:
                self.loginItemStatusLabel.stringValue = @"システム設定で許可が必要";
                break;
            case SMAppServiceStatusNotFound:
                self.loginItemStatusLabel.stringValue = @"利用できません";
                break;
            case SMAppServiceStatusNotRegistered:
            default:
                self.loginItemStatusLabel.stringValue = @"オフ";
                break;
        }
    } else {
        self.loginItemCheckbox.state = NSControlStateValueOff;
        self.loginItemCheckbox.enabled = NO;
        self.loginItemStatusLabel.stringValue = @"macOS 13以降で利用可能";
    }
}

- (void)showLoginItemError:(NSError *)error needsApproval:(BOOL)needsApproval {
    NSAlert *alert = [[NSAlert alloc] init];
    alert.alertStyle = NSAlertStyleWarning;
    alert.messageText = needsApproval
        ? @"ログイン項目の許可が必要です"
        : @"ログイン時の起動を変更できませんでした";
    alert.informativeText = needsApproval
        ? @"システム設定の「ログイン項目」で cmd-kana を許可してください。"
        : (error.localizedDescription ?: @"しばらくしてからもう一度お試しください。");
    [alert addButtonWithTitle:needsApproval ? @"システム設定を開く" : @"OK"];
    if (needsApproval) [alert addButtonWithTitle:@"あとで"];
    if ([alert runModal] == NSAlertFirstButtonReturn && needsApproval) {
        [SMAppService openSystemSettingsLoginItems];
    }
}

- (void)toggleLoginItem:(id)sender {
    if (@available(macOS 13.0, *)) {
        BOOL shouldEnable = self.loginItemCheckbox.state == NSControlStateValueOn;
        NSError *error = nil;
        BOOL succeeded = shouldEnable
            ? [SMAppService.mainAppService registerAndReturnError:&error]
            : [SMAppService.mainAppService unregisterAndReturnError:&error];
        [self updateLoginItemStatus];
        BOOL needsApproval = SMAppService.mainAppService.status == SMAppServiceStatusRequiresApproval;
        if (!succeeded || needsApproval) {
            [self showLoginItemError:error needsApproval:needsApproval];
        }
    }
}

- (void)updatePermissionStatus {
    NSString *listenStatus = CGPreflightListenEventAccess()
        ? (is_event_monitoring() ? @"許可済み（動作中）" : @"許可済み（開始待ち）")
        : @"未許可";
    self.listenStatusLabel.stringValue = [NSString stringWithFormat:@"入力監視: %@", listenStatus];
    self.postStatusLabel.stringValue = [NSString stringWithFormat:@"アクセシビリティ: %@",
        CGPreflightPostEventAccess() ? @"許可済み" : @"未許可"];
}

- (void)retryEventMonitoring:(NSTimer *)timer {
    if (!is_event_monitoring()) start_event_monitoring();
    [self updatePermissionStatus];
}

- (void)showPermissionHelp:(id)sender {
    NSAlert *alert = [[NSAlert alloc] init];
    alert.messageText = @"入力切り替えが動かない場合";
    alert.informativeText =
        @"1. 入力監視とアクセシビリティの両方を開きます。\n"
         "2. cmd-kana がオンでも動かない場合は、一度リストから削除します。\n"
         "3. /Applications/cmd-kana.app を追加し直してオンにします。\n\n"
         "許可が反映されると、cmd-kana は自動的に入力監視を再開します。";
    [alert addButtonWithTitle:@"入力監視を開く"];
    [alert addButtonWithTitle:@"アクセシビリティを開く"];
    [alert addButtonWithTitle:@"閉じる"];
    NSModalResponse response = [alert runModal];
    if (response == NSAlertFirstButtonReturn) {
        [self openInputMonitoring:nil];
    } else if (response == NSAlertSecondButtonReturn) {
        [self openAccessibility:nil];
    }
}

- (void)showSettings:(id)sender {
    [self updatePermissionStatus];
    [self updateLoginItemStatus];
    [self.settingsWindow makeKeyAndOrderFront:nil];
    [NSApp activateIgnoringOtherApps:YES];
}

- (BOOL)applicationShouldHandleReopen:(NSApplication *)sender hasVisibleWindows:(BOOL)flag {
    [self showSettings:nil];
    return YES;
}

- (void)applicationDidBecomeActive:(NSNotification *)notification {
    [self updatePermissionStatus];
    [self updateLoginItemStatus];
}

- (void)openInputMonitoring:(id)sender {
    [[NSWorkspace sharedWorkspace] openURL:[NSURL URLWithString:
        @"x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent"]];
}

- (void)openAccessibility:(id)sender {
    [[NSWorkspace sharedWorkspace] openURL:[NSURL URLWithString:
        @"x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility"]];
}

- (void)updateApp:(id)sender {
    NSString *executablePath = [NSBundle mainBundle].executablePath;
    if (executablePath) {
        NSTask *task = [[NSTask alloc] init];
        task.launchPath = executablePath;
        task.arguments = @[@"--relaunch-after-pid",
            [NSString stringWithFormat:@"%d", NSProcessInfo.processInfo.processIdentifier]];
        [task launch];
    }
    [NSApp terminate:nil];
}

- (void)quitApp:(id)sender {
    [NSApp terminate:nil];
}

@end

void run_menu_bar_app(void) {
    @autoreleasepool {
        NSApplication *app = [NSApplication sharedApplication];
        CmdKanaDelegate *delegate = [[CmdKanaDelegate alloc] init];
        app.delegate = delegate;
        [app setActivationPolicy:NSApplicationActivationPolicyAccessory];
        [app run];
    }
}
