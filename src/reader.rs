use super::{Key, ScanType, KeyState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReaderMode {
    Set1,
    Set2,
    Set3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// The reader simply reads bytes and returns keys
pub struct Reader {
    mode: ReaderMode,
    history_scan_codes: [u8; 6],
}

// Special bytes https://wiki.osdev.org/Keyboard
// 0x00	            Key detection error or internal buffer overrun
// 0xAA	            Self test passed (sent after "0xFF (reset)" command or keyboard power up)
// 0xEE	            Response to "0xEE (echo)" command
// 0xFA	            Command acknowledged (ACK)
// 0xFC and 0xFD	Self test failed (sent after "0xFF (reset)" command or keyboard power up)
// 0xFE	            Resend (keyboard wants controller to repeat last command it sent)
// 0xFF	            Key detection error or internal buffer overrun

impl Reader {
    pub fn new(mode: ReaderMode) -> Reader {
        return Reader {
            mode,
            history_scan_codes: [0u8; 6],
        };
    }

    pub fn zero_scan_codes(&mut self) {
        self.history_scan_codes = [0u8; 6];
    }

    pub fn switch_scan_mode(&mut self, mode: ReaderMode) {
        self.zero_scan_codes();
        self.mode = mode;
    }

    pub fn input_scan_code(&mut self, code: u8) -> Result<Option<Key>, &'static str> {
        return match self.mode {
            ReaderMode::Set1 => self.input_scan_code_s1(code),
            ReaderMode::Set2 => self.input_scan_code_s1(code),
            ReaderMode::Set3 => self.input_scan_code_s1(code),
        };
    }

    fn input_scan_code_s1(&mut self, code: u8) -> Result<Option<Key>, &'static str> {
        let mut first_free = 6;

        for (i, b) in self.history_scan_codes.iter().enumerate() {
            if *b == 0x00 {
                first_free = i;
                break;
            }
        }

        if first_free == 6 {
            return Err("Scan code buffer full."); // This should never happen but just in case.
        }

        // This is the first code being interpreted for a key
        if first_free == 0 {
            if code <= 0xd8 {
                let k = Self::map_simple_scan_code_s1(code);

                self.zero_scan_codes();

                return Ok(k);
            } else if code == 0xe0 || code == 0xe1 {
                // Multimedia key so add it to the history
            
                self.history_scan_codes[0] = code;

                return Ok(None);
            } else {
                return Ok(None);
            }
            
        } else if first_free == 1 {
            // The first code should be 0xe0, we need to check this. Otherwise if it is 0xe1 we should store the next code

            if self.history_scan_codes[0] == 0xe1 {
                if code != 0x1d {
                    // Pause is the only code that starts with 0xe1 and 0x1d should follow
                    self.zero_scan_codes();
                    return Err("Invalid follow-up code for 0xE1.");
                }

                self.history_scan_codes[1] = code;
                return Ok(None);

            } else if self.history_scan_codes[0] != 0xe0 {
                self.zero_scan_codes();
                return Err("Invalid previous scan code.");
            }

            // The first code was 0xe0

            if code == 0x2a || code == 0xb7 {
                self.history_scan_codes[1] = code;

                return Ok(None);
            }

            let k = Self::map_media_scan_code_s1(code);
            self.zero_scan_codes();
            return Ok(k);
        } else if first_free == 2 { // If the buffer reaches this size we know we have only 3 possible characters
            let previous_code = self.history_scan_codes[1];

            if previous_code == 0x2a || previous_code == 0xb7 {
                if code != 0xe0 {
                    self.zero_scan_codes();

                    return Err("Invalid scan code history.");
                }
                self.history_scan_codes[2] = code;
                    
                return Ok(None);
            } else if previous_code == 0x1d {
                if code != 0x45 {
                    self.zero_scan_codes();

                    return Err("Invalid scan code history.");
                }

                self.history_scan_codes[2] = code;
                    
                return Ok(None);
            }else {
                self.zero_scan_codes();

                return Err("Invalid scan code history.")
            }
        } else if first_free == 3 {
            let previous_code = self.history_scan_codes[2];

            if previous_code == 0xe0 {
                self.zero_scan_codes();

                if code == 0x37 {
                    return Ok(Some(Key::new(ScanType::PrintScreen, KeyState::Pressed)));
                }else if code == 0xaa {
                    return Ok(Some(Key::new(ScanType::PrintScreen, KeyState::Released)));
                } else {
                    return Err("Invalid scan code expected 0x37 or 0xaa.");
                }
            } else if previous_code == 0x45 {
                if code != 0xe1 {
                    self.zero_scan_codes();

                    return Err("Invalid scan code expected 0xe1.");
                }

                self.history_scan_codes[3] = code;
                return Ok(None);
            } else {
                self.zero_scan_codes();

                return Err("Invalid scan code history.")
            }
        } else if first_free == 4 {
            if self.history_scan_codes[3] != 0xe1 {
                self.zero_scan_codes();

                return Err("Invalid scan code history.");
            }

            if code != 0x9d {
                self.zero_scan_codes();
                return Err("Invalid scan code expected 0x9d");
            }

            self.history_scan_codes[4] = code;
        } else if first_free == 5 {
            if self.history_scan_codes[4] != 0x9d {
                self.zero_scan_codes();

                return Err("Invalid scan code history.");
            }

            if code != 0xc5 {
                self.zero_scan_codes();
                return Err("Invalid scan code expected 0xc5");
            } 

            self.zero_scan_codes();
            return Ok(Some(Key::new(ScanType::Pause, KeyState::Pressed)));
        }

        return Ok(None);
    }

    fn map_simple_scan_code_s1(code: u8) -> Option<Key> {
        macro_rules! create_pressed_key {
            ($scan_type:expr) => {
                {
                    Some(Key::new($scan_type, KeyState::Pressed))
                }
            };
        }

        macro_rules! create_pressed_keypad_key {
            ($scan_type:expr) => {
                {
                    Some(Key::new_keypad($scan_type, KeyState::Pressed))
                }
            };
        }

        match code {
            0x01 => return create_pressed_key!(ScanType::Escape),
            0x02 => return create_pressed_key!(ScanType::Num1),
            0x03 => return create_pressed_key!(ScanType::Num2),
            0x04 => return create_pressed_key!(ScanType::Num3),
            0x05 => return create_pressed_key!(ScanType::Num4),
            0x06 => return create_pressed_key!(ScanType::Num5),
            0x07 => return create_pressed_key!(ScanType::Num6),
            0x08 => return create_pressed_key!(ScanType::Num7),
            0x09 => return create_pressed_key!(ScanType::Num8),
            0x0a => return create_pressed_key!(ScanType::Num9),
            0x0b => return create_pressed_key!(ScanType::Num0),
            0x0c => return create_pressed_key!(ScanType::SymbolMinus),
            0x0d => return create_pressed_key!(ScanType::SymbolEquals),
            0x0e => return create_pressed_key!(ScanType::Backspace),
            0x0f => return create_pressed_key!(ScanType::Tab),

            0x10 => return create_pressed_key!(ScanType::CharQ),
            0x11 => return create_pressed_key!(ScanType::CharW),
            0x12 => return create_pressed_key!(ScanType::CharE),
            0x13 => return create_pressed_key!(ScanType::CharR),
            0x14 => return create_pressed_key!(ScanType::CharT),
            0x15 => return create_pressed_key!(ScanType::CharY),
            0x16 => return create_pressed_key!(ScanType::CharU),
            0x17 => return create_pressed_key!(ScanType::CharI),
            0x18 => return create_pressed_key!(ScanType::CharO),
            0x19 => return create_pressed_key!(ScanType::CharP),

            0x1a => return create_pressed_key!(ScanType::SymbolOpenSquareBracket),
            0x1b => return create_pressed_key!(ScanType::SymbolOpenSquareBracket),
            0x1c => return create_pressed_key!(ScanType::Enter),
            0x1d => return create_pressed_key!(ScanType::LeftCtrl),

            0x1e => return create_pressed_key!(ScanType::CharA),
            0x1f => return create_pressed_key!(ScanType::CharS),
            0x20 => return create_pressed_key!(ScanType::CharD),
            0x21 => return create_pressed_key!(ScanType::CharF),
            0x22 => return create_pressed_key!(ScanType::CharG),
            0x23 => return create_pressed_key!(ScanType::CharH),
            0x24 => return create_pressed_key!(ScanType::CharJ),
            0x25 => return create_pressed_key!(ScanType::CharK),
            0x26 => return create_pressed_key!(ScanType::CharL),

            0x27 => return create_pressed_key!(ScanType::SymbolSemicolon),
            0x28 => return create_pressed_key!(ScanType::SymbolSingleQuote),
            0x29 => return create_pressed_key!(ScanType::SymbolBacktick),
            0x2a => return create_pressed_key!(ScanType::LeftShift),
            0x2b => return create_pressed_key!(ScanType::SymbolBackslash), // '\'
            
            0x2c => return create_pressed_key!(ScanType::CharZ),
            0x2d => return create_pressed_key!(ScanType::CharX),
            0x2e => return create_pressed_key!(ScanType::CharC),
            0x2f => return create_pressed_key!(ScanType::CharV),
            0x30 => return create_pressed_key!(ScanType::CharB),
            0x31 => return create_pressed_key!(ScanType::CharN),
            0x32 => return create_pressed_key!(ScanType::CharM),

            0x33 => return create_pressed_key!(ScanType::SymbolComma),
            0x34 => return create_pressed_key!(ScanType::SymbolPeriod),
            0x35 => return create_pressed_key!(ScanType::SymbolForwardSlash),
            0x36 => return create_pressed_key!(ScanType::RightShift),

            0x37 => return create_pressed_keypad_key!(ScanType::SymbolAsterisk),

            0x38 => return create_pressed_key!(ScanType::LeftAlt),
            0x39 => return create_pressed_key!(ScanType::Space),
            0x3a => return create_pressed_key!(ScanType::CapsLock),

            0x3b => return create_pressed_key!(ScanType::F1),
            0x3c => return create_pressed_key!(ScanType::F2),
            0x3d => return create_pressed_key!(ScanType::F3),
            0x3e => return create_pressed_key!(ScanType::F4),
            0x3f => return create_pressed_key!(ScanType::F5),
            0x40 => return create_pressed_key!(ScanType::F6),
            0x41 => return create_pressed_key!(ScanType::F7),
            0x42 => return create_pressed_key!(ScanType::F8),
            0x43 => return create_pressed_key!(ScanType::F9),
            0x44 => return create_pressed_key!(ScanType::F10),

            0x45 => return create_pressed_key!(ScanType::NumLock),
            0x46 => return create_pressed_key!(ScanType::ScrollLock),

            0x47 => return create_pressed_keypad_key!(ScanType::Num7),
            0x48 => return create_pressed_keypad_key!(ScanType::Num8),
            0x49 => return create_pressed_keypad_key!(ScanType::Num9),
            0x4a => return create_pressed_keypad_key!(ScanType::SymbolMinus),
            0x4b => return create_pressed_keypad_key!(ScanType::Num4),
            0x4c => return create_pressed_keypad_key!(ScanType::Num5),
            0x4d => return create_pressed_keypad_key!(ScanType::Num6),
            0x4e => return create_pressed_keypad_key!(ScanType::SymbolPlus),
            0x4f => return create_pressed_keypad_key!(ScanType::Num1),
            0x50 => return create_pressed_keypad_key!(ScanType::Num2),
            0x51 => return create_pressed_keypad_key!(ScanType::Num3),
            0x52 => return create_pressed_keypad_key!(ScanType::Num0),
            0x53 => return create_pressed_keypad_key!(ScanType::SymbolPeriod),
            // 0x54 ... 0x56
            0x57 => return create_pressed_key!(ScanType::F11),
            0x58 => return create_pressed_key!(ScanType::F12),


            // Released keys
            0x81 ..= 0xd3 => {
                match Self::map_simple_scan_code_s1(code - 0x80) {
                    Some(n) => return Some(n.inverted_state()),
                    None => return None,
                }
            },

            0xd7 => Some(Key::new(ScanType::F11, KeyState::Released)),
            0xd8 => Some(Key::new(ScanType::F12, KeyState::Released)),

            _ => return None,
        }
    }

    fn map_media_scan_code_s1(code: u8) -> Option<Key> {
        macro_rules! create_pressed_key {
            ($scan_type:expr) => {
                {
                    Some(Key::new($scan_type, KeyState::Pressed))
                }
            };
        }

        macro_rules! create_pressed_keypad_key {
            ($scan_type:expr) => {
                {
                    Some(Key::new_keypad($scan_type, KeyState::Pressed))
                }
            };
        }

        match code {
            0x10 => return create_pressed_key!(ScanType::PreviousTrack),
            0x19 => return create_pressed_key!(ScanType::NextTrack),
            0x1c => return create_pressed_keypad_key!(ScanType::Enter),
            0x1d => return create_pressed_key!(ScanType::RightCtrl),
            0x20 => return create_pressed_key!(ScanType::Mute),
            0x21 => return create_pressed_key!(ScanType::Calculator),
            0x22 => return create_pressed_key!(ScanType::Play),
            0x24 => return create_pressed_key!(ScanType::Stop),
            0x2e => return create_pressed_key!(ScanType::VolumeDown),
            0x30 => return create_pressed_key!(ScanType::VolumeUp),
            0x32 => return create_pressed_key!(ScanType::WWWHome),
            0x35 => return create_pressed_keypad_key!(ScanType::SymbolForwardSlash),
            0x38 => return create_pressed_key!(ScanType::RightAlt),
            0x48 => return create_pressed_key!(ScanType::CursorUp),
            0x49 => return create_pressed_key!(ScanType::PageUp),
            0x4b => return create_pressed_key!(ScanType::CursorLeft),
            0x4d => return create_pressed_key!(ScanType::CursorRight),
            0x4f => return create_pressed_key!(ScanType::End),
            0x50 => return create_pressed_key!(ScanType::CursorDown),
            0x51 => return create_pressed_key!(ScanType::PageDown),
            0x52 => return create_pressed_key!(ScanType::Insert),
            0x53 => return create_pressed_key!(ScanType::Delete),
            0x5b => return create_pressed_key!(ScanType::LeftGUI),
            0x5c => return create_pressed_key!(ScanType::RightGUI),
            0x5d => return create_pressed_key!(ScanType::Apps),
            0x5e => return create_pressed_key!(ScanType::ACPIPower),
            0x5f => return create_pressed_key!(ScanType::ACPISleep),
            0x63 => return create_pressed_key!(ScanType::ACPIWake),
            0x65 => return create_pressed_key!(ScanType::WWWSearch),
            0x66 => return create_pressed_key!(ScanType::WWWFavorites),
            0x67 => return create_pressed_key!(ScanType::WWWRefresh),
            0x68 => return create_pressed_key!(ScanType::WWWStop),
            0x69 => return create_pressed_key!(ScanType::WWWForward),
            0x6a => return create_pressed_key!(ScanType::WWWBack),
            0x6b => return create_pressed_key!(ScanType::MyComputer),
            0x6c => return create_pressed_key!(ScanType::Email),
            0x6d => return create_pressed_key!(ScanType::MediaSelect),
            0x90 ..= 0xed => {
                match Self::map_media_scan_code_s1(code - 0x80) {
                    Some(n) => return Some(n.inverted_state()),
                    None => return None,
                }
            },
            _ => return None,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    mod set_1 {
        use super::*;

        #[test]
        fn test_simple_scan_1() {
            let mut reader = Reader::new(ReaderMode::Set1);
    
            assert_eq!(reader.input_scan_code(0x22).unwrap().unwrap(), Key::new(ScanType::CharG, KeyState::Pressed));
        }

        #[test]
        fn test_simple_scan_2() {
            let mut reader = Reader::new(ReaderMode::Set1);
    
            assert_eq!(reader.input_scan_code(0x57).unwrap().unwrap(), Key::new(ScanType::F11, KeyState::Pressed));
        }

        #[test]
        fn test_simple_scan_3() {
            let mut reader = Reader::new(ReaderMode::Set1);
    
            assert_eq!(reader.input_scan_code(0x57 + 0x80).unwrap().unwrap(), Key::new(ScanType::F11, KeyState::Released));
        }

        #[test]
        fn test_simple_scan_4() {
            let mut reader = Reader::new(ReaderMode::Set1);
    
            assert_eq!(reader.input_scan_code(0xa0).unwrap().unwrap(), Key::new(ScanType::CharD, KeyState::Released));
        }

        #[test]
        fn test_simple_scan_5() {
            let mut reader = Reader::new(ReaderMode::Set1);
    
            assert!(reader.input_scan_code(0xe0).unwrap().is_none());
        }

        #[test]
        fn test_simple_scan_6() {
            let mut reader = Reader::new(ReaderMode::Set1);
    
            assert!(reader.input_scan_code(0xe1).unwrap().is_none());
        }

        #[test]
        fn test_failed_scan_1() {
            let mut reader = Reader::new(ReaderMode::Set1);
    
            assert!(reader.input_scan_code(0xde).unwrap().is_none());
        }

        #[test]
        fn test_failed_scan_2() {
            let mut reader = Reader::new(ReaderMode::Set1);
    
            assert!(reader.input_scan_code(0xe0).unwrap().is_none());
            assert!(reader.input_scan_code(0xe0).unwrap().is_none());
        }

        #[test]
        fn test_media_scan_1() {
            let mut reader = Reader::new(ReaderMode::Set1);
    
            assert!(reader.input_scan_code(0xe0).unwrap().is_none());
            assert_eq!(reader.input_scan_code(0xe8).unwrap().unwrap(), Key::new(ScanType::WWWStop, KeyState::Released));
        }

        #[test]
        fn test_media_scan_2() {
            let mut reader = Reader::new(ReaderMode::Set1);
    
            assert!(reader.input_scan_code(0xe0).unwrap().is_none());
            assert_eq!(reader.input_scan_code(0x68).unwrap().unwrap(), Key::new(ScanType::WWWStop, KeyState::Pressed));
        }

        #[test]
        fn test_media_scan_3() {
            let mut reader = Reader::new(ReaderMode::Set1);
    
            assert!(reader.input_scan_code(0xe0).unwrap().is_none());
            assert_eq!(reader.input_scan_code(0x49).unwrap().unwrap(), Key::new(ScanType::PageUp, KeyState::Pressed));
        }

        #[test]
        fn test_media_scan_4() {
            let mut reader = Reader::new(ReaderMode::Set1);
    
            assert!(reader.input_scan_code(0xe0).unwrap().is_none());
            assert_eq!(reader.input_scan_code(0xc9).unwrap().unwrap(), Key::new(ScanType::PageUp, KeyState::Released));
        }

        #[test]
        fn test_print_screen_pressed() {
            let mut reader = Reader::new(ReaderMode::Set1);
    
            assert!(reader.input_scan_code(0xe0).unwrap().is_none());
            assert!(reader.input_scan_code(0x2a).unwrap().is_none());
            assert!(reader.input_scan_code(0xe0).unwrap().is_none());
            assert_eq!(reader.input_scan_code(0x37).unwrap().unwrap(), Key::new(ScanType::PrintScreen, KeyState::Pressed));
        }

        #[test]
        fn test_print_screen_released() {
            let mut reader = Reader::new(ReaderMode::Set1);
    
            assert!(reader.input_scan_code(0xe0).unwrap().is_none());
            assert!(reader.input_scan_code(0xb7).unwrap().is_none());
            assert!(reader.input_scan_code(0xe0).unwrap().is_none());
            assert_eq!(reader.input_scan_code(0xaa).unwrap().unwrap(), Key::new(ScanType::PrintScreen, KeyState::Released));
        }

        #[test]
        fn test_pause_pressed() {
            let mut reader = Reader::new(ReaderMode::Set1);
    
            assert!(reader.input_scan_code(0xe1).unwrap().is_none());
            assert!(reader.input_scan_code(0x1d).unwrap().is_none());
            assert!(reader.input_scan_code(0x45).unwrap().is_none());
            assert!(reader.input_scan_code(0xe1).unwrap().is_none());
            assert!(reader.input_scan_code(0x9d).unwrap().is_none());
            assert_eq!(reader.input_scan_code(0xc5).unwrap().unwrap(), Key::new(ScanType::Pause, KeyState::Pressed));
        }

        #[test]
        fn test_combination_1() {
            let mut reader = Reader::new(ReaderMode::Set1);
    
            assert!(reader.input_scan_code(0xe1).unwrap().is_none());
            assert!(reader.input_scan_code(0x1d).unwrap().is_none());
            assert!(reader.input_scan_code(0x45).unwrap().is_none());
            assert!(reader.input_scan_code(0xe1).unwrap().is_none());
            assert!(reader.input_scan_code(0x9d).unwrap().is_none());
            assert_eq!(reader.input_scan_code(0xc5).unwrap().unwrap(), Key::new(ScanType::Pause, KeyState::Pressed));

            assert!(reader.input_scan_code(0xe0).unwrap().is_none());
            assert!(reader.input_scan_code(0xb7).unwrap().is_none());
            assert!(reader.input_scan_code(0xe0).unwrap().is_none());
            assert_eq!(reader.input_scan_code(0xaa).unwrap().unwrap(), Key::new(ScanType::PrintScreen, KeyState::Released));
        }


        #[test]
        fn test_combination_2() {
            let mut reader = Reader::new(ReaderMode::Set1);
    
            assert_eq!(reader.input_scan_code(0x14).unwrap().unwrap(), Key::new(ScanType::CharT, KeyState::Pressed));
            assert_eq!(reader.input_scan_code(0x94).unwrap().unwrap(), Key::new(ScanType::CharT, KeyState::Released));

            assert_eq!(reader.input_scan_code(0x12).unwrap().unwrap(), Key::new(ScanType::CharE, KeyState::Pressed));
            assert_eq!(reader.input_scan_code(0x92).unwrap().unwrap(), Key::new(ScanType::CharE, KeyState::Released));

            assert_eq!(reader.input_scan_code(0x1f).unwrap().unwrap(), Key::new(ScanType::CharS, KeyState::Pressed));
            assert_eq!(reader.input_scan_code(0x9f).unwrap().unwrap(), Key::new(ScanType::CharS, KeyState::Released));

            assert_eq!(reader.input_scan_code(0x14).unwrap().unwrap(), Key::new(ScanType::CharT, KeyState::Pressed));
            assert_eq!(reader.input_scan_code(0x94).unwrap().unwrap(), Key::new(ScanType::CharT, KeyState::Released));
        }
    }
}