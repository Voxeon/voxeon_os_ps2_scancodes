use super::{Reader, ReaderMode, Key, ScanType};
use super::layout::{Layout, KeyModifierState};

pub struct Keyboard<T> where T: Layout{
    reader: Reader,
    modifiers: KeyModifierState,
    layout: T,
}

impl<T: Layout> Keyboard<T> {
    pub fn new(mode: ReaderMode, layout: T) -> Self {
        return Self {
            reader: Reader::new(mode),
            modifiers: KeyModifierState::new(),
            layout,
        };
    }

    pub fn current_state(&self) -> KeyModifierState {
        return self.modifiers;
    }

    pub fn input_byte(&mut self, byte: u8) -> Option<char> {
        let k = self.raw_input_byte(byte)?;

        return self.layout.key_into_char(&self.modifiers, k);
    }

    pub fn raw_input_byte(&mut self, byte: u8) -> Option<Key> {
        return match self.try_raw_input_byte(byte) {
            Ok(v) => v,
            Err(_) => None,
        };
    }

    pub fn try_raw_input_byte(&mut self, byte: u8) -> Result<Option<Key>, &str> {
        let res = self.reader.input_scan_code(byte);

        match res {
            Ok(Some(k)) => {
                self.check_apply_modifiers(&k);

                return Ok(Some(k));
            },
            _ => return res,
        }
    }

    fn check_apply_modifiers(&mut self, key: &Key) {
        match key.scan_type() {
            ScanType::LeftGUI => self.modifiers.left_gui = key.is_pressed(),
            ScanType::RightGUI => self.modifiers.right_gui = key.is_pressed(),
            ScanType::LeftAlt => self.modifiers.left_alt = key.is_pressed(),
            ScanType::RightAlt => self.modifiers.right_alt = key.is_pressed(),
            ScanType::LeftShift => self.modifiers.left_shift = key.is_pressed(),
            ScanType::RightShift => self.modifiers.right_shift = key.is_pressed(),
            ScanType::LeftCtrl => self.modifiers.left_ctrl = key.is_pressed(),
            ScanType::RightCtrl => self.modifiers.right_ctrl = key.is_pressed(),
            ScanType::NumLock => {
                // Toggle only when pressed
                if key.is_pressed() {
                    self.modifiers.num_lock =  !self.modifiers.num_lock;
                }
            },
            ScanType::CapsLock => {
                // Toggle only when pressed
                if key.is_pressed() {
                    self.modifiers.caps_lock = !self.modifiers.caps_lock;
                }
            },
            ScanType::ScrollLock => {
                // Toggle only when pressed
                if key.is_pressed() {
                    self.modifiers.scroll_lock =  !self.modifiers.scroll_lock;
                }
            },
            _ => (),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::layout::USStandardLayout;

    mod set1 {
        use super::*;

        #[test]
        fn test_single_character() {
            let mut key_board = Keyboard::new(ReaderMode::Set1, USStandardLayout);

            assert_eq!(key_board.input_byte(0x1e).unwrap(), 'a');
        }

        #[test]
        fn test_upper_character() {
            let mut key_board = Keyboard::new(ReaderMode::Set1, USStandardLayout);

            assert!(key_board.input_byte(0x36).is_none()); // Right shift

            assert_eq!(key_board.input_byte(0x1e).unwrap(), 'A');
        }

        #[test]
        fn test_upper_character_capslock() {
            let mut key_board = Keyboard::new(ReaderMode::Set1, USStandardLayout);

            assert!(key_board.input_byte(0x3a).is_none()); // Capslock pressed
            assert_eq!(key_board.input_byte(0x1e).unwrap(), 'A');

            assert!(key_board.input_byte(0xba).is_none()); // CapsLock released
            assert_eq!(key_board.input_byte(0x1e).unwrap(), 'A');
        }

        #[test]
        fn test_upper_character_capslock_toggle() {
            let mut key_board = Keyboard::new(ReaderMode::Set1, USStandardLayout);

            assert!(key_board.input_byte(0x3a).is_none()); // Capslock pressed
            assert_eq!(key_board.input_byte(0x1e).unwrap(), 'A');
            assert!(key_board.input_byte(0xba).is_none()); // CapsLock released
            assert!(key_board.input_byte(0x3a).is_none()); // Capslock pressed
            assert!(key_board.input_byte(0xba).is_none()); // CapsLock released
            assert_eq!(key_board.input_byte(0x1e).unwrap(), 'a');
        }
    }
}