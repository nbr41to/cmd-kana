use std::time::Duration;

pub const LEFT_COMMAND: i64 = 55;
pub const RIGHT_COMMAND: i64 = 54;
pub const DOUBLE_TAP: Duration = Duration::from_millis(550);

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Side {
    English,
    Japanese,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RecordedKey {
    pub keycode: u16,
    pub shift: bool,
}

impl RecordedKey {
    fn us_letter(self) -> Option<char> {
        Some(match self.keycode {
            0 => 'a',
            11 => 'b',
            8 => 'c',
            2 => 'd',
            14 => 'e',
            3 => 'f',
            5 => 'g',
            4 => 'h',
            34 => 'i',
            38 => 'j',
            40 => 'k',
            37 => 'l',
            46 => 'm',
            45 => 'n',
            31 => 'o',
            35 => 'p',
            12 => 'q',
            15 => 'r',
            1 => 's',
            17 => 't',
            32 => 'u',
            9 => 'v',
            13 => 'w',
            7 => 'x',
            16 => 'y',
            6 => 'z',
            _ => return None,
        })
    }
}

pub fn roman_text(keys: &[RecordedKey]) -> Option<String> {
    keys.iter()
        .copied()
        .map(|key| {
            let letter = key.us_letter()?;
            Some(if key.shift {
                letter.to_ascii_uppercase()
            } else {
                letter
            })
        })
        .collect()
}

pub fn hiragana_length(keys: &[RecordedKey]) -> Option<usize> {
    if keys.iter().any(|key| key.shift) {
        // Shift can select ASCII or Katakana according to the IME settings.
        // A lowercase romaji estimate cannot safely describe that text.
        return None;
    }
    // This is the legacy non-Google estimate. Google/Mozc must use its own
    // native raw-text conversion, whose composition rules differ by mode.
    let romaji = roman_text(keys)?;
    let mut rest = romaji.as_str();
    let mut length = 0;
    while !rest.is_empty() {
        let bytes = rest.as_bytes();
        if bytes.starts_with(b"nn")
            && (bytes.len() == 2 || !matches!(bytes[2], b'a' | b'e' | b'i' | b'o' | b'u' | b'y'))
        {
            length += 1;
            rest = &rest[2..];
            continue;
        }
        if bytes.len() >= 2
            && bytes[0] == bytes[1]
            && !matches!(bytes[0], b'a' | b'e' | b'i' | b'n' | b'o' | b'u')
        {
            length += 1;
            rest = &rest[1..];
            continue;
        }
        if bytes[0] == b'n'
            && (bytes.len() == 1
                || (bytes[1] != b'y' && !matches!(bytes[1], b'a' | b'e' | b'i' | b'o' | b'u')))
        {
            length += 1;
            rest = &rest[1..];
            continue;
        }
        if let Some((roman, kana)) = ROMAJI
            .iter()
            .filter(|(roman, _)| rest.starts_with(roman))
            .max_by_key(|(roman, _)| roman.len())
        {
            length += kana.chars().count();
            rest = &rest[roman.len()..];
        } else {
            length += 1;
            rest = &rest[1..];
        }
    }
    Some(length)
}

const ROMAJI: &[(&str, &str)] = &[
    ("zh", "←"),
    ("zj", "↓"),
    ("zk", "↑"),
    ("zl", "→"),
    ("xtsu", "っ"),
    ("ltsu", "っ"),
    ("xtu", "っ"),
    ("ltu", "っ"),
    ("xa", "ぁ"),
    ("xi", "ぃ"),
    ("xu", "ぅ"),
    ("xe", "ぇ"),
    ("xo", "ぉ"),
    ("la", "ぁ"),
    ("li", "ぃ"),
    ("lu", "ぅ"),
    ("le", "ぇ"),
    ("lo", "ぉ"),
    ("xya", "ゃ"),
    ("xyu", "ゅ"),
    ("xyo", "ょ"),
    ("lya", "ゃ"),
    ("lyu", "ゅ"),
    ("lyo", "ょ"),
    ("xyi", "ぃ"),
    ("xye", "ぇ"),
    ("lyi", "ぃ"),
    ("lye", "ぇ"),
    ("xwa", "ゎ"),
    ("lwa", "ゎ"),
    ("xka", "ヵ"),
    ("xke", "ヶ"),
    ("lka", "ヵ"),
    ("lke", "ヶ"),
    ("xn", "ん"),
    ("thi", "てぃ"),
    ("tha", "てゃ"),
    ("thu", "てゅ"),
    ("the", "てぇ"),
    ("tho", "てょ"),
    ("dhi", "でぃ"),
    ("dha", "でゃ"),
    ("dhu", "でゅ"),
    ("dhe", "でぇ"),
    ("dho", "でょ"),
    ("dya", "ぢゃ"),
    ("dyi", "ぢぃ"),
    ("dyu", "ぢゅ"),
    ("dye", "ぢぇ"),
    ("dyo", "ぢょ"),
    ("tsa", "つぁ"),
    ("tsi", "つぃ"),
    ("tse", "つぇ"),
    ("tso", "つぉ"),
    ("twa", "とぁ"),
    ("twi", "とぃ"),
    ("twu", "とぅ"),
    ("twe", "とぇ"),
    ("two", "とぉ"),
    ("dwa", "どぁ"),
    ("dwi", "どぃ"),
    ("dwu", "どぅ"),
    ("dwe", "どぇ"),
    ("dwo", "どぉ"),
    ("wi", "うぃ"),
    ("we", "うぇ"),
    ("wyi", "ゐ"),
    ("wye", "ゑ"),
    ("wha", "うぁ"),
    ("whi", "うぃ"),
    ("whu", "う"),
    ("whe", "うぇ"),
    ("who", "うぉ"),
    ("wu", "う"),
    ("ye", "いぇ"),
    ("ca", "か"),
    ("ci", "し"),
    ("cu", "く"),
    ("ce", "せ"),
    ("co", "こ"),
    ("qa", "くぁ"),
    ("qi", "くぃ"),
    ("qu", "く"),
    ("qe", "くぇ"),
    ("qo", "くぉ"),
    ("kwa", "くぁ"),
    ("kwi", "くぃ"),
    ("kwu", "くぅ"),
    ("kwe", "くぇ"),
    ("kwo", "くぉ"),
    ("gwa", "ぐぁ"),
    ("gwi", "ぐぃ"),
    ("gwu", "ぐぅ"),
    ("gwe", "ぐぇ"),
    ("gwo", "ぐぉ"),
    ("swa", "すぁ"),
    ("swi", "すぃ"),
    ("swu", "すぅ"),
    ("swe", "すぇ"),
    ("swo", "すぉ"),
    ("zwa", "ずぁ"),
    ("zwi", "ずぃ"),
    ("zwu", "ずぅ"),
    ("zwe", "ずぇ"),
    ("zwo", "ずぉ"),
    ("fya", "ふゃ"),
    ("fyu", "ふゅ"),
    ("fyo", "ふょ"),
    ("hwa", "ふぁ"),
    ("hwi", "ふぃ"),
    ("hwe", "ふぇ"),
    ("hwo", "ふぉ"),
    ("hwyu", "ふゅ"),
    ("vya", "ゔゃ"),
    ("vyi", "ゔぃ"),
    ("vyu", "ゔゅ"),
    ("vye", "ゔぇ"),
    ("vyo", "ゔょ"),
    ("kyi", "きぃ"),
    ("kye", "きぇ"),
    ("gyi", "ぎぃ"),
    ("gye", "ぎぇ"),
    ("syi", "しぃ"),
    ("sye", "しぇ"),
    ("zya", "じゃ"),
    ("zyi", "じぃ"),
    ("zyu", "じゅ"),
    ("zye", "じぇ"),
    ("zyo", "じょ"),
    ("tyi", "ちぃ"),
    ("tye", "ちぇ"),
    ("cyi", "ちぃ"),
    ("cye", "ちぇ"),
    ("nyi", "にぃ"),
    ("nye", "にぇ"),
    ("hyi", "ひぃ"),
    ("hye", "ひぇ"),
    ("byi", "びぃ"),
    ("bye", "びぇ"),
    ("pyi", "ぴぃ"),
    ("pye", "ぴぇ"),
    ("myi", "みぃ"),
    ("mye", "みぇ"),
    ("ryi", "りぃ"),
    ("rye", "りぇ"),
    ("jyi", "じぃ"),
    ("jye", "じぇ"),
    ("kya", "きゃ"),
    ("kyu", "きゅ"),
    ("kyo", "きょ"),
    ("sha", "しゃ"),
    ("shu", "しゅ"),
    ("sho", "しょ"),
    ("sya", "しゃ"),
    ("syu", "しゅ"),
    ("syo", "しょ"),
    ("cha", "ちゃ"),
    ("chu", "ちゅ"),
    ("cho", "ちょ"),
    ("cya", "ちゃ"),
    ("cyu", "ちゅ"),
    ("cyo", "ちょ"),
    ("tya", "ちゃ"),
    ("tyu", "ちゅ"),
    ("tyo", "ちょ"),
    ("nya", "にゃ"),
    ("nyu", "にゅ"),
    ("nyo", "にょ"),
    ("hya", "ひゃ"),
    ("hyu", "ひゅ"),
    ("hyo", "ひょ"),
    ("mya", "みゃ"),
    ("myu", "みゅ"),
    ("myo", "みょ"),
    ("rya", "りゃ"),
    ("ryu", "りゅ"),
    ("ryo", "りょ"),
    ("gya", "ぎゃ"),
    ("gyu", "ぎゅ"),
    ("gyo", "ぎょ"),
    ("bya", "びゃ"),
    ("byu", "びゅ"),
    ("byo", "びょ"),
    ("pya", "ぴゃ"),
    ("pyu", "ぴゅ"),
    ("pyo", "ぴょ"),
    ("jya", "じゃ"),
    ("jyu", "じゅ"),
    ("jyo", "じょ"),
    ("shi", "し"),
    ("chi", "ち"),
    ("tsu", "つ"),
    ("she", "しぇ"),
    ("che", "ちぇ"),
    ("je", "じぇ"),
    ("ja", "じゃ"),
    ("ju", "じゅ"),
    ("jo", "じょ"),
    ("fa", "ふぁ"),
    ("fi", "ふぃ"),
    ("fe", "ふぇ"),
    ("fo", "ふぉ"),
    ("va", "ゔぁ"),
    ("vi", "ゔぃ"),
    ("vu", "ゔ"),
    ("ve", "ゔぇ"),
    ("vo", "ゔぉ"),
    ("ka", "か"),
    ("ki", "き"),
    ("ku", "く"),
    ("ke", "け"),
    ("ko", "こ"),
    ("sa", "さ"),
    ("si", "し"),
    ("su", "す"),
    ("se", "せ"),
    ("so", "そ"),
    ("ta", "た"),
    ("ti", "ち"),
    ("tu", "つ"),
    ("te", "て"),
    ("to", "と"),
    ("na", "な"),
    ("ni", "に"),
    ("nu", "ぬ"),
    ("ne", "ね"),
    ("no", "の"),
    ("ha", "は"),
    ("hi", "ひ"),
    ("fu", "ふ"),
    ("hu", "ふ"),
    ("he", "へ"),
    ("ho", "ほ"),
    ("ma", "ま"),
    ("mi", "み"),
    ("mu", "む"),
    ("me", "め"),
    ("mo", "も"),
    ("ya", "や"),
    ("yu", "ゆ"),
    ("yo", "よ"),
    ("ra", "ら"),
    ("ri", "り"),
    ("ru", "る"),
    ("re", "れ"),
    ("ro", "ろ"),
    ("wa", "わ"),
    ("wo", "を"),
    ("ga", "が"),
    ("gi", "ぎ"),
    ("gu", "ぐ"),
    ("ge", "げ"),
    ("go", "ご"),
    ("za", "ざ"),
    ("zi", "じ"),
    ("ji", "じ"),
    ("zu", "ず"),
    ("ze", "ぜ"),
    ("zo", "ぞ"),
    ("da", "だ"),
    ("di", "ぢ"),
    ("du", "づ"),
    ("de", "で"),
    ("do", "ど"),
    ("ba", "ば"),
    ("bi", "び"),
    ("bu", "ぶ"),
    ("be", "べ"),
    ("bo", "ぼ"),
    ("pa", "ぱ"),
    ("pi", "ぴ"),
    ("pu", "ぷ"),
    ("pe", "ぺ"),
    ("po", "ぽ"),
    ("a", "あ"),
    ("i", "い"),
    ("u", "う"),
    ("e", "え"),
    ("o", "お"),
];

#[derive(Debug, PartialEq)]
pub enum Action {
    Switch(Side),
    ReplaceWithEnglish(Vec<RecordedKey>),
    NativeEnglishDoubleTap,
}

#[derive(Default)]
pub struct InputState {
    held: u8,
    used_with_other_key: bool,
    last_left_tap_ms: Option<u64>,
    last_right_tap_ms: Option<u64>,
    typed: Vec<RecordedKey>,
    recording_blocked: bool,
    english_selected: bool,
    native_english_conversion: bool,
    last_left_tap_native: bool,
}

impl InputState {
    pub fn set_native_english_conversion(&mut self, enabled: bool) {
        self.native_english_conversion = enabled;
    }

    pub fn flags_changed(
        &mut self,
        keycode: i64,
        now_ms: u64,
        command: bool,
        other_modifiers: bool,
    ) -> Option<Action> {
        if keycode != LEFT_COMMAND && keycode != RIGHT_COMMAND {
            if command || self.held != 0 {
                self.clear();
            }
            return None;
        }
        if other_modifiers {
            self.clear();
        }
        let action = self.command_changed(keycode, now_ms);
        if other_modifiers {
            // On key-down, suppress the newly held Command as well.
            self.clear();
            None
        } else {
            action
        }
    }

    pub fn command_changed(&mut self, keycode: i64, now_ms: u64) -> Option<Action> {
        let mask = match keycode {
            LEFT_COMMAND => 1,
            RIGHT_COMMAND => 2,
            _ => return None,
        };
        if self.held & mask != 0 {
            self.held &= !mask;
            if self.used_with_other_key {
                self.last_left_tap_ms = None;
                self.last_right_tap_ms = None;
                return None;
            }
            if keycode == RIGHT_COMMAND {
                self.last_left_tap_ms = None;
                self.typed.clear();
                self.recording_blocked = false;
                self.english_selected = false;
                let repeated = self.last_right_tap_ms.is_some_and(|last| {
                    now_ms.saturating_sub(last) <= DOUBLE_TAP.as_millis() as u64
                });
                self.last_right_tap_ms = Some(now_ms);
                if repeated {
                    return None;
                }
                return Some(Action::Switch(Side::Japanese));
            }
            self.last_right_tap_ms = None;
            self.english_selected = true;
            let double = self
                .last_left_tap_ms
                .is_some_and(|last| now_ms.saturating_sub(last) <= DOUBLE_TAP.as_millis() as u64);
            if double && self.last_left_tap_native {
                // Google Japanese Input owns the marked text and its raw key
                // history. Let its native Eisu double tap commit that history.
                self.last_left_tap_ms = None;
                self.last_left_tap_native = false;
                self.typed.clear();
                return Some(Action::NativeEnglishDoubleTap);
            }
            if double && !self.typed.is_empty() {
                self.last_left_tap_ms = None;
                return Some(Action::ReplaceWithEnglish(std::mem::take(&mut self.typed)));
            }
            if self.last_left_tap_ms.is_some() {
                // An earlier English switch has expired. Its composition must
                // never become part of a later double-tap replacement.
                self.typed.clear();
            }
            self.last_left_tap_ms = Some(now_ms);
            self.last_left_tap_native = self.native_english_conversion;
            return Some(Action::Switch(Side::English));
        }
        if self.held == 0 {
            self.used_with_other_key = false;
        } else {
            self.clear();
        }
        self.held |= mask;
        None
    }

    pub fn key_down(&mut self, keycode: i64, shift: bool, shortcut: bool) {
        if self.held != 0 {
            self.clear();
            return;
        }
        if shortcut {
            self.clear();
            return;
        }
        if self.last_left_tap_ms.take().is_some() {
            // Typing after the first left tap cancels its Japanese candidate.
            self.typed.clear();
        }
        self.last_right_tap_ms = None;
        match keycode {
            102 | 104 => {
                self.clear();
                self.english_selected = keycode == 102;
            }
            51 => {
                // IMEs delete rendered characters, not physical keys. Without
                // observing the composition, its remaining extent is unknown.
                self.typed.clear();
                self.recording_blocked = true;
            }
            36 | 48 | 49 | 53 | 76 | 123..=126 => self.clear(),
            0..=127 => {
                let key = RecordedKey {
                    keycode: keycode as u16,
                    shift,
                };
                if key.us_letter().is_none() {
                    self.clear();
                    return;
                }
                if self.recording_blocked || self.english_selected {
                    return;
                }
                if self.typed.len() == 128 {
                    // A sliding window can cut through a multi-key syllable.
                    self.typed.clear();
                    self.recording_blocked = true;
                    return;
                }
                self.typed.push(key);
            }
            _ => self.clear(),
        }
    }

    pub fn clear(&mut self) {
        if self.held != 0 {
            self.used_with_other_key = true;
        }
        self.typed.clear();
        self.recording_blocked = false;
        self.last_left_tap_ms = None;
        self.last_left_tap_native = false;
        self.last_right_tap_ms = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keycode(character: char) -> u16 {
        match character.to_ascii_lowercase() {
            'a' => 0,
            'b' => 11,
            'c' => 8,
            'd' => 2,
            'e' => 14,
            'f' => 3,
            'g' => 5,
            'h' => 4,
            'i' => 34,
            'j' => 38,
            'k' => 40,
            'l' => 37,
            'm' => 46,
            'n' => 45,
            'o' => 31,
            'p' => 35,
            'q' => 12,
            'r' => 15,
            's' => 1,
            't' => 17,
            'u' => 32,
            'v' => 9,
            'w' => 13,
            'x' => 7,
            'y' => 16,
            'z' => 6,
            _ => panic!("unsupported test character"),
        }
    }

    fn keys(text: &str) -> Vec<RecordedKey> {
        text.chars()
            .map(|character| RecordedKey {
                keycode: keycode(character),
                shift: character.is_ascii_uppercase(),
            })
            .collect()
    }

    fn tap(state: &mut InputState, key: i64, at: u64) -> Option<Action> {
        assert_eq!(state.flags_changed(key, at, true, false), None);
        state.flags_changed(key, at + 1, false, false)
    }

    #[test]
    fn single_commands_always_switch_to_the_expected_side() {
        let mut state = InputState::default();
        assert_eq!(
            tap(&mut state, LEFT_COMMAND, 0),
            Some(Action::Switch(Side::English))
        );
        assert_eq!(
            tap(&mut state, RIGHT_COMMAND, 100),
            Some(Action::Switch(Side::Japanese))
        );
        assert_eq!(
            tap(&mut state, RIGHT_COMMAND, 800),
            Some(Action::Switch(Side::Japanese))
        );
    }

    #[test]
    fn left_double_tap_replays_exact_physical_input() {
        let mut state = InputState::default();
        for keycode in [40, 31, 45, 45, 45, 34, 17, 4, 0] {
            state.key_down(keycode, false, false); // konnnitiha
        }
        assert_eq!(
            tap(&mut state, LEFT_COMMAND, 1000),
            Some(Action::Switch(Side::English))
        );
        assert_eq!(
            tap(&mut state, LEFT_COMMAND, 1200),
            Some(Action::ReplaceWithEnglish(vec![
                RecordedKey {
                    keycode: 40,
                    shift: false
                },
                RecordedKey {
                    keycode: 31,
                    shift: false
                },
                RecordedKey {
                    keycode: 45,
                    shift: false
                },
                RecordedKey {
                    keycode: 45,
                    shift: false
                },
                RecordedKey {
                    keycode: 45,
                    shift: false
                },
                RecordedKey {
                    keycode: 34,
                    shift: false
                },
                RecordedKey {
                    keycode: 17,
                    shift: false
                },
                RecordedKey {
                    keycode: 4,
                    shift: false
                },
                RecordedKey {
                    keycode: 0,
                    shift: false
                },
            ]))
        );
    }

    #[test]
    fn command_shortcuts_never_switch_or_convert() {
        let mut state = InputState::default();
        state.key_down(0, false, false);
        state.command_changed(LEFT_COMMAND, 0);
        state.key_down(8, false, true); // Command-C
        assert_eq!(state.command_changed(LEFT_COMMAND, 10), None);
        assert_eq!(
            tap(&mut state, LEFT_COMMAND, 100),
            Some(Action::Switch(Side::English))
        );
        assert_eq!(
            tap(&mut state, LEFT_COMMAND, 200),
            Some(Action::Switch(Side::English))
        );
    }

    #[test]
    fn rapid_right_taps_send_only_one_japanese_switch() {
        let mut state = InputState::default();
        assert_eq!(
            tap(&mut state, RIGHT_COMMAND, 0),
            Some(Action::Switch(Side::Japanese))
        );
        assert_eq!(tap(&mut state, RIGHT_COMMAND, 100), None);
        assert_eq!(
            tap(
                &mut state,
                RIGHT_COMMAND,
                100 + DOUBLE_TAP.as_millis() as u64 + 2
            ),
            Some(Action::Switch(Side::Japanese))
        );
    }

    #[test]
    fn right_switch_discards_the_previous_conversion_history() {
        let mut state = InputState::default();
        for key in keys("aka") {
            state.key_down(key.keycode.into(), key.shift, false);
        }
        assert_eq!(
            tap(&mut state, RIGHT_COMMAND, 0),
            Some(Action::Switch(Side::Japanese))
        );
        assert_eq!(
            tap(&mut state, LEFT_COMMAND, 1_000),
            Some(Action::Switch(Side::English))
        );
        assert_eq!(
            tap(&mut state, LEFT_COMMAND, 1_100),
            Some(Action::Switch(Side::English))
        );
    }

    #[test]
    fn typing_after_an_english_switch_does_not_reinterpret_latin_as_kana() {
        let mut state = InputState::default();
        for key in keys("aka") {
            state.key_down(key.keycode.into(), key.shift, false);
        }
        assert_eq!(
            tap(&mut state, LEFT_COMMAND, 0),
            Some(Action::Switch(Side::English))
        );
        for key in keys("kya") {
            state.key_down(key.keycode.into(), key.shift, false);
        }
        assert_eq!(
            tap(&mut state, LEFT_COMMAND, 1_000),
            Some(Action::Switch(Side::English))
        );
        assert_eq!(
            tap(&mut state, LEFT_COMMAND, 1_100),
            Some(Action::Switch(Side::English))
        );
    }

    #[test]
    fn confirmed_conversion_and_punctuation_discard_history() {
        for boundary in [36, 76, 43, 47] {
            let mut state = InputState::default();
            for key in keys("aka") {
                state.key_down(key.keycode.into(), key.shift, false);
            }
            state.key_down(boundary, false, false);
            assert_eq!(
                tap(&mut state, LEFT_COMMAND, 0),
                Some(Action::Switch(Side::English))
            );
            assert_eq!(
                tap(&mut state, LEFT_COMMAND, 100),
                Some(Action::Switch(Side::English))
            );
        }
    }

    #[test]
    fn boundaries_clear_history_so_unrelated_text_is_never_replayed() {
        let mut state = InputState::default();
        state.key_down(0, false, false);
        state.key_down(49, false, false); // Space
        assert_eq!(
            tap(&mut state, LEFT_COMMAND, 0),
            Some(Action::Switch(Side::English))
        );
        assert_eq!(
            tap(&mut state, LEFT_COMMAND, 100),
            Some(Action::Switch(Side::English))
        );
    }

    #[test]
    fn delete_cancels_replacement_even_with_pending_latin_input() {
        let mut state = InputState::default();
        state.key_down(0, false, false);
        state.key_down(11, true, false);
        state.key_down(51, false, false);
        assert_eq!(
            tap(&mut state, LEFT_COMMAND, 0),
            Some(Action::Switch(Side::English))
        );
        assert_eq!(
            tap(&mut state, LEFT_COMMAND, 100),
            Some(Action::Switch(Side::English))
        );
    }

    #[test]
    fn physical_romaji_has_the_expected_hiragana_length() {
        let keys = |codes: &[u16]| {
            codes
                .iter()
                .map(|&keycode| RecordedKey {
                    keycode,
                    shift: false,
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(hiragana_length(&keys(&[0, 40, 0])), Some(2)); // aka -> あか
        assert_eq!(
            hiragana_length(&keys(&[40, 31, 45, 45, 34, 17, 34, 4, 0])),
            Some(5)
        ); // konnitiha
        assert_eq!(hiragana_length(&keys(&[40, 16, 31, 32])), Some(3)); // kyou -> きょう
    }

    #[test]
    fn physical_keys_produce_the_exact_roman_replacement_text() {
        let keys = [
            RecordedKey {
                keycode: 0,
                shift: false,
            },
            RecordedKey {
                keycode: 40,
                shift: true,
            },
            RecordedKey {
                keycode: 0,
                shift: false,
            },
        ];
        assert_eq!(roman_text(&keys).as_deref(), Some("aKa"));
    }

    #[test]
    fn double_tap_boundary_is_inclusive_and_late_taps_are_single() {
        let mut state = InputState::default();
        state.key_down(0, false, false);
        assert_eq!(
            tap(&mut state, LEFT_COMMAND, 0),
            Some(Action::Switch(Side::English))
        );
        assert!(matches!(
            tap(&mut state, LEFT_COMMAND, DOUBLE_TAP.as_millis() as u64),
            Some(Action::ReplaceWithEnglish(_))
        ));

        tap(&mut state, RIGHT_COMMAND, 800);
        state.key_down(0, false, false);
        assert_eq!(
            tap(&mut state, LEFT_COMMAND, 1_000),
            Some(Action::Switch(Side::English))
        );
        assert_eq!(
            tap(
                &mut state,
                LEFT_COMMAND,
                1_000 + DOUBLE_TAP.as_millis() as u64 + 2
            ),
            Some(Action::Switch(Side::English))
        );
    }

    #[test]
    fn empty_history_never_creates_a_replacement() {
        let mut state = InputState::default();
        assert_eq!(
            tap(&mut state, LEFT_COMMAND, 0),
            Some(Action::Switch(Side::English))
        );
        assert_eq!(
            tap(&mut state, LEFT_COMMAND, 100),
            Some(Action::Switch(Side::English))
        );
    }

    #[test]
    fn exactly_128_keys_can_still_be_replaced() {
        let mut state = InputState::default();
        for _ in 0..128 {
            state.key_down(0, false, false);
        }
        assert_eq!(
            tap(&mut state, LEFT_COMMAND, 0),
            Some(Action::Switch(Side::English))
        );
        let Some(Action::ReplaceWithEnglish(keys)) = tap(&mut state, LEFT_COMMAND, 100) else {
            panic!("expected replacement");
        };
        assert_eq!(keys.len(), 128);
    }

    #[test]
    fn simultaneous_commands_do_not_trigger_a_switch() {
        let mut state = InputState::default();
        assert_eq!(state.command_changed(LEFT_COMMAND, 0), None);
        assert_eq!(state.command_changed(RIGHT_COMMAND, 1), None);
        assert_eq!(state.command_changed(RIGHT_COMMAND, 2), None);
        assert_eq!(state.command_changed(LEFT_COMMAND, 3), None);
    }

    #[test]
    fn romaji_length_handles_n_gemination_and_compound_kana() {
        assert_eq!(hiragana_length(&keys("kan")), Some(2)); // かん
        assert_eq!(hiragana_length(&keys("nn")), Some(1)); // ん
        assert_eq!(hiragana_length(&keys("konnnitiha")), Some(5)); // こんにちは
        assert_eq!(hiragana_length(&keys("kanna")), Some(3));
        assert_eq!(hiragana_length(&keys("kitte")), Some(3)); // きって
        assert_eq!(hiragana_length(&keys("gakkou")), Some(4)); // がっこう
        assert_eq!(hiragana_length(&keys("shashin")), Some(4)); // しゃしん
    }

    #[test]
    fn mixed_latin_and_kana_output_has_the_correct_length() {
        assert_eq!(hiragana_length(&keys("kfa")), Some(3)); // ｋふぁ
        assert_eq!(hiragana_length(&keys("q")), Some(1)); // ｑ
    }

    #[test]
    fn small_kana_and_extended_syllables_have_the_correct_length() {
        for (roman, kana) in [
            ("xa", "ぁ"),
            ("la", "ぁ"),
            ("xi", "ぃ"),
            ("li", "ぃ"),
            ("xu", "ぅ"),
            ("lu", "ぅ"),
            ("xe", "ぇ"),
            ("le", "ぇ"),
            ("xo", "ぉ"),
            ("lo", "ぉ"),
            ("xya", "ゃ"),
            ("lya", "ゃ"),
            ("xyu", "ゅ"),
            ("lyu", "ゅ"),
            ("xyo", "ょ"),
            ("lyo", "ょ"),
            ("xtu", "っ"),
            ("ltu", "っ"),
            ("xtsu", "っ"),
            ("ltsu", "っ"),
            ("xwa", "ゎ"),
            ("lwa", "ゎ"),
            ("xka", "ヵ"),
            ("lka", "ヵ"),
            ("xke", "ヶ"),
            ("lke", "ヶ"),
            ("xn", "ん"),
            ("thi", "てぃ"),
            ("dhi", "でぃ"),
            ("dhu", "でゅ"),
            ("tsa", "つぁ"),
            ("tsi", "つぃ"),
            ("tse", "つぇ"),
            ("tso", "つぉ"),
            ("wi", "うぃ"),
            ("we", "うぇ"),
            ("whu", "う"),
            ("qa", "くぁ"),
            ("qi", "くぃ"),
            ("qu", "く"),
            ("gwa", "ぐぁ"),
            ("she", "しぇ"),
            ("che", "ちぇ"),
            ("dya", "ぢゃ"),
            ("dyu", "ぢゅ"),
            ("dyo", "ぢょ"),
            ("fyu", "ふゅ"),
            ("vya", "ゔゃ"),
            ("hwyu", "ふゅ"),
            ("texisuto", "てぃすと"),
            ("maxtucha", "まっちゃ"),
        ] {
            assert_eq!(
                hiragana_length(&keys(roman)),
                Some(kana.chars().count()),
                "{roman} → {kana}"
            );
        }
    }

    #[test]
    fn delete_discards_composition_instead_of_popping_one_physical_key() {
        let mut state = InputState::default();
        for key in keys("ka") {
            state.key_down(key.keycode.into(), key.shift, false);
        }
        state.key_down(51, false, false); // Deletes か, not the physical a.
        assert_eq!(
            tap(&mut state, LEFT_COMMAND, 0),
            Some(Action::Switch(Side::English))
        );
        assert_eq!(
            tap(&mut state, LEFT_COMMAND, 100),
            Some(Action::Switch(Side::English))
        );
    }

    #[test]
    fn overlapping_commands_do_not_turn_the_final_release_into_a_new_press() {
        let mut state = InputState::default();
        for (key, time) in [
            (LEFT_COMMAND, 0),
            (RIGHT_COMMAND, 1),
            (LEFT_COMMAND, 2),
            (RIGHT_COMMAND, 3),
        ] {
            assert_eq!(state.command_changed(key, time), None);
        }
        assert_eq!(
            tap(&mut state, RIGHT_COMMAND, 100),
            Some(Action::Switch(Side::Japanese))
        );
        assert_eq!(
            tap(&mut state, LEFT_COMMAND, 200),
            Some(Action::Switch(Side::English))
        );
    }

    #[test]
    fn clicking_while_command_is_held_cancels_the_single_tap() {
        let mut state = InputState::default();
        assert_eq!(state.command_changed(LEFT_COMMAND, 0), None);
        state.clear(); // Mouse down, including Command-click.
        assert_eq!(state.command_changed(LEFT_COMMAND, 10), None);
    }

    #[test]
    fn overflowing_a_composition_disables_replacement_until_a_boundary() {
        let mut state = InputState::default();
        for key in keys(&"ka".repeat(65)) {
            state.key_down(key.keycode.into(), key.shift, false);
        }
        assert_eq!(
            tap(&mut state, LEFT_COMMAND, 0),
            Some(Action::Switch(Side::English))
        );
        assert_eq!(
            tap(&mut state, LEFT_COMMAND, 100),
            Some(Action::Switch(Side::English))
        );
    }

    #[test]
    fn unsupported_physical_input_fails_safely() {
        assert_eq!(
            roman_text(&[RecordedKey {
                keycode: 49,
                shift: false
            }]),
            None
        );
    }

    #[test]
    fn consecutive_n_pairs_commit_only_one_character_per_pair() {
        for (roman, kana) in [
            ("nnka", "んか"),
            ("nnn", "んん"),
            ("nnnn", "んん"),
            ("kannpai", "かんぱい"),
            ("konnnitiha", "こんにちは"),
            ("kanna", "かんな"),
            ("konnya", "こんにゃ"),
        ] {
            assert_eq!(
                hiragana_length(&keys(roman)),
                Some(kana.chars().count()),
                "{roman}"
            );
        }
    }

    #[test]
    fn alphabetic_symbol_shortcuts_count_as_one_rendered_character() {
        for roman in ["zh", "zj", "zk", "zl"] {
            assert_eq!(hiragana_length(&keys(roman)), Some(1));
        }
        assert_eq!(hiragana_length(&keys("kazh")), Some(2)); // か←
    }

    #[test]
    fn all_overlapping_command_orders_recover_for_the_next_single_tap() {
        for (first, second) in [(LEFT_COMMAND, RIGHT_COMMAND), (RIGHT_COMMAND, LEFT_COMMAND)] {
            for release_first in [first, second] {
                let release_second = if release_first == first {
                    second
                } else {
                    first
                };
                let mut state = InputState::default();
                for (time, key) in [first, second, release_first, release_second]
                    .into_iter()
                    .enumerate()
                {
                    assert_eq!(state.command_changed(key, time as u64), None);
                }
                assert_eq!(
                    tap(&mut state, LEFT_COMMAND, 100),
                    Some(Action::Switch(Side::English))
                );
                assert_eq!(
                    tap(&mut state, RIGHT_COMMAND, 200),
                    Some(Action::Switch(Side::Japanese))
                );
            }
        }
    }

    #[test]
    fn repeated_conversions_preserve_text_and_timing_boundaries() {
        // Reuse one state across 4,968 conversions, including left/right
        // switches, successful replacements, and expired double taps.
        let mut state = InputState::default();
        let mut now = 1_000;
        for delay in 1..=552 {
            for (roman, kana) in [
                ("aka", "あか"),
                ("konnnitiha", "こんにちは"),
                ("gakkou", "がっこう"),
                ("shashin", "しゃしん"),
                ("maxtucha", "まっちゃ"),
                ("thi", "てぃ"),
                ("kfa", "ｋふぁ"),
                ("nnka", "んか"),
                ("xya", "ゃ"),
            ] {
                assert_eq!(
                    tap(&mut state, RIGHT_COMMAND, now),
                    Some(Action::Switch(Side::Japanese))
                );
                let recorded = keys(roman);
                for key in &recorded {
                    state.key_down(key.keycode.into(), key.shift, false);
                }
                assert_eq!(
                    tap(&mut state, LEFT_COMMAND, now + 10),
                    Some(Action::Switch(Side::English))
                );
                let action = tap(&mut state, LEFT_COMMAND, now + 10 + delay);
                if delay <= 550 {
                    assert_eq!(
                        action,
                        Some(Action::ReplaceWithEnglish(recorded.clone())),
                        "{roman}, {delay}ms"
                    );
                    assert_eq!(hiragana_length(&recorded), Some(kana.chars().count()));
                    assert_eq!(roman_text(&recorded).as_deref(), Some(roman));
                } else {
                    assert_eq!(action, Some(Action::Switch(Side::English)));
                }
                now += 2_000;
            }
        }
    }

    #[test]
    fn deletion_and_overflow_stay_blocked_until_a_boundary() {
        for overflow in [false, true] {
            let mut state = InputState::default();
            for _ in 0..if overflow { 129 } else { 1 } {
                state.key_down(0, false, false);
            }
            if !overflow {
                state.key_down(51, false, false);
            }
            for key in keys("kya") {
                state.key_down(key.keycode.into(), key.shift, false);
            }
            assert!(state.typed.is_empty());
            state.key_down(49, false, false);
            for key in keys("aka") {
                state.key_down(key.keycode.into(), key.shift, false);
            }
            tap(&mut state, LEFT_COMMAND, 0);
            assert_eq!(
                tap(&mut state, LEFT_COMMAND, 100),
                Some(Action::ReplaceWithEnglish(keys("aka")))
            );
        }
    }

    #[test]
    fn japanese_switch_resumes_recording_after_english_and_blocked_input() {
        for japanese_key in [RIGHT_COMMAND, 104] {
            let mut state = InputState::default();
            tap(&mut state, LEFT_COMMAND, 0);
            state.key_down(51, false, false);
            if japanese_key == RIGHT_COMMAND {
                tap(&mut state, RIGHT_COMMAND, 100);
            } else {
                state.key_down(japanese_key, false, false);
            }
            for key in keys("aka") {
                state.key_down(key.keycode.into(), key.shift, false);
            }
            tap(&mut state, LEFT_COMMAND, 200);
            assert_eq!(
                tap(&mut state, LEFT_COMMAND, 300),
                Some(Action::ReplaceWithEnglish(keys("aka")))
            );
        }
    }

    #[test]
    fn command_modifier_chords_discard_existing_candidates() {
        for side in [LEFT_COMMAND, RIGHT_COMMAND] {
            for modifier in [56, 59, 58] {
                // Shift, Control, Option.
                for modifier_first in [false, true] {
                    let mut state = InputState::default();
                    for key in keys("aka") {
                        state.key_down(key.keycode.into(), key.shift, false);
                    }
                    if modifier_first {
                        assert_eq!(state.flags_changed(modifier, 0, false, true), None);
                        assert_eq!(state.flags_changed(side, 1, true, true), None);
                    } else {
                        assert_eq!(state.flags_changed(side, 0, true, false), None);
                        assert_eq!(state.flags_changed(modifier, 1, true, true), None);
                    }
                    assert_eq!(state.flags_changed(side, 2, false, true), None);
                    assert_eq!(state.flags_changed(modifier, 3, false, false), None);
                    assert_eq!(
                        tap(&mut state, LEFT_COMMAND, 100),
                        Some(Action::Switch(Side::English))
                    );
                    assert_eq!(
                        tap(&mut state, LEFT_COMMAND, 200),
                        Some(Action::Switch(Side::English))
                    );
                }
            }
        }
    }

    #[test]
    fn google_double_tap_uses_native_conversion_without_a_length_estimate() {
        for roman in ["", "konnnitiha", "nnya", "Kya", "gakkou", "lto"] {
            for delay in [1, 200, 499, 500, 550, 551] {
                let mut state = InputState::default();
                state.set_native_english_conversion(true);
                for key in keys(roman) {
                    state.key_down(key.keycode.into(), key.shift, false);
                }
                assert_eq!(
                    tap(&mut state, LEFT_COMMAND, 0),
                    Some(Action::Switch(Side::English))
                );
                // The first Eisu may select a different input source. Keep the
                // original provider attached to this double-tap candidate.
                state.set_native_english_conversion(false);
                let expected = if delay <= 550 {
                    Action::NativeEnglishDoubleTap
                } else {
                    Action::Switch(Side::English)
                };
                assert_eq!(
                    tap(&mut state, LEFT_COMMAND, delay),
                    Some(expected),
                    "{roman}, {delay}ms"
                );
            }
        }
    }

    #[test]
    fn google_native_conversion_does_not_depend_on_the_recorded_word() {
        let mut state = InputState::default();
        state.set_native_english_conversion(true);
        for _ in 0..129 {
            state.key_down(0, false, false);
        }
        state.key_down(51, false, false);
        assert!(state.typed.is_empty());
        tap(&mut state, LEFT_COMMAND, 0);
        assert_eq!(
            tap(&mut state, LEFT_COMMAND, 100),
            Some(Action::NativeEnglishDoubleTap)
        );
    }

    #[test]
    fn boundaries_cancel_a_pending_google_native_conversion() {
        for boundary in [36, 49, 51, 123] {
            let mut state = InputState::default();
            state.set_native_english_conversion(true);
            tap(&mut state, LEFT_COMMAND, 0);
            state.key_down(boundary, false, false);
            assert_eq!(
                tap(&mut state, LEFT_COMMAND, 100),
                Some(Action::Switch(Side::English))
            );
        }
        let mut state = InputState::default();
        state.set_native_english_conversion(true);
        tap(&mut state, LEFT_COMMAND, 0);
        state.clear(); // Focus moved by a click.
        assert_eq!(
            tap(&mut state, LEFT_COMMAND, 100),
            Some(Action::Switch(Side::English))
        );
    }

    #[test]
    fn shifted_input_does_not_use_a_lowercase_deletion_estimate() {
        for roman in ["Ka", "Kya", "KANJI", "aKa"] {
            assert_eq!(hiragana_length(&keys(roman)), None);
            assert_eq!(roman_text(&keys(roman)).as_deref(), Some(roman));
        }
    }
}
