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
    let romaji = roman_text(keys)?.to_ascii_lowercase();
    let mut rest = romaji.as_str();
    let mut length = 0;
    while !rest.is_empty() {
        let bytes = rest.as_bytes();
        if rest == "nn" {
            length += 1; // terminal nn commits a single ん
            break;
        }
        if bytes.len() >= 2
            && bytes[0] == bytes[1]
            && !matches!(bytes[0], b'a' | b'e' | b'i' | b'n' | b'o' | b'u')
        {
            length += 1; // small っ
            rest = &rest[1..];
            continue;
        }
        if bytes[0] == b'n'
            && (bytes.len() == 1
                || (bytes[1] != b'y' && !matches!(bytes[1], b'a' | b'e' | b'i' | b'o' | b'u')))
        {
            length += 1; // ん
            rest = &rest[1..];
            continue;
        }
        if let Some((roman, kana)) = ROMAJI.iter().find(|(roman, _)| rest.starts_with(roman)) {
            length += kana.chars().count();
            rest = &rest[roman.len()..];
        } else {
            // An uncommitted consonant is rendered as one full-width Latin
            // character by Japanese IMEs (for example kfa -> ｋふぁ).
            length += 1;
            rest = &rest[1..];
        }
    }
    Some(length)
}

const ROMAJI: &[(&str, &str)] = &[
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
}

#[derive(Default)]
pub struct InputState {
    held: Option<i64>,
    used_with_other_key: bool,
    last_left_tap_ms: Option<u64>,
    typed: Vec<RecordedKey>,
}

impl InputState {
    pub fn command_changed(&mut self, keycode: i64, now_ms: u64) -> Option<Action> {
        if keycode != LEFT_COMMAND && keycode != RIGHT_COMMAND {
            return None;
        }
        if self.held == Some(keycode) {
            self.held = None;
            if self.used_with_other_key {
                self.last_left_tap_ms = None;
                return None;
            }
            if keycode == RIGHT_COMMAND {
                self.last_left_tap_ms = None;
                return Some(Action::Switch(Side::Japanese));
            }
            let double = self
                .last_left_tap_ms
                .is_some_and(|last| now_ms.saturating_sub(last) <= DOUBLE_TAP.as_millis() as u64);
            if double && !self.typed.is_empty() {
                self.last_left_tap_ms = None;
                return Some(Action::ReplaceWithEnglish(std::mem::take(&mut self.typed)));
            }
            self.last_left_tap_ms = Some(now_ms);
            return Some(Action::Switch(Side::English));
        }
        if self.held.is_none() {
            self.held = Some(keycode);
            self.used_with_other_key = false;
        } else {
            self.used_with_other_key = true;
        }
        None
    }

    pub fn key_down(&mut self, keycode: i64, shift: bool, shortcut: bool) {
        if self.held.is_some() {
            self.used_with_other_key = true;
            self.typed.clear();
            self.last_left_tap_ms = None;
            return;
        }
        if shortcut {
            self.clear();
            return;
        }
        match keycode {
            51 => {
                self.typed.pop();
            }
            36 | 48 | 49 | 53 | 76 | 123..=126 => self.clear(),
            0..=127 => {
                if self.typed.len() == 128 {
                    self.typed.remove(0);
                }
                self.typed.push(RecordedKey {
                    keycode: keycode as u16,
                    shift,
                });
            }
            _ => self.clear(),
        }
    }

    pub fn clear(&mut self) {
        self.typed.clear();
        self.last_left_tap_ms = None;
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
        assert_eq!(state.command_changed(key, at), None);
        state.command_changed(key, at + 1)
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
            tap(&mut state, RIGHT_COMMAND, 200),
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
    fn right_double_tap_is_only_two_japanese_switches() {
        let mut state = InputState::default();
        assert_eq!(
            tap(&mut state, RIGHT_COMMAND, 0),
            Some(Action::Switch(Side::Japanese))
        );
        assert_eq!(
            tap(&mut state, RIGHT_COMMAND, 100),
            Some(Action::Switch(Side::Japanese))
        );
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
    fn delete_updates_the_recorded_physical_input() {
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
            Some(Action::ReplaceWithEnglish(vec![RecordedKey {
                keycode: 0,
                shift: false
            }]))
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
    fn history_is_bounded_to_the_latest_128_keys() {
        let mut state = InputState::default();
        for _ in 0..130 {
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
        assert_eq!(hiragana_length(&keys("kanna")), Some(3)); // かんな
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
    fn unsupported_physical_input_fails_safely() {
        assert_eq!(
            roman_text(&[RecordedKey {
                keycode: 49,
                shift: false
            }]),
            None
        );
    }
}
