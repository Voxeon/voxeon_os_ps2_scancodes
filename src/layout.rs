use super::{Key, ScanType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyModifierState {
    pub left_shift: bool,
    pub left_alt: bool,
    pub left_ctrl: bool,
    pub left_gui: bool,
    pub right_shift: bool,
    pub right_alt: bool,
    pub right_ctrl: bool,
    pub right_gui: bool,
    pub caps_lock: bool,
    pub num_lock: bool,
    pub scroll_lock: bool,
}

impl KeyModifierState {
    pub fn new() -> Self {
        return KeyModifierState {
            left_shift: false,
            left_alt: false,
            left_ctrl: false,
            left_gui: false,
            right_shift: false,
            right_alt: false,
            right_ctrl: false,
            right_gui: false,
            caps_lock: false,
            num_lock: false,
            scroll_lock: false,
        };
    }

    pub fn shift_down(&self) -> bool {
        return self.left_shift || self.right_shift;
    }

    pub fn ctrl_down(&self) -> bool {
        return self.left_ctrl || self.right_ctrl;
    }

    pub fn alt_down(&self) -> bool {
        return self.left_alt || self.right_alt;
    }

    pub fn gui_down(&self) -> bool {
        return self.left_gui || self.right_gui;
    }
}

// These only specify a way to create a character from a key only.
pub trait Layout {
    fn key_into_char(&self, modifiers: &KeyModifierState, key: Key) -> Option<char>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct USStandardLayout;
impl Layout for USStandardLayout {
    fn key_into_char(&self, modifiers: &KeyModifierState, key: Key) -> Option<char> {
        use ScanType::*;
        let ch;

        if modifiers.shift_down() {
            // Shift is being held down

            ch = match key.scan_type() {
                // Check if the character is on the keypad
                CharA => 'A',
                CharB => 'B',
                CharC => 'C',
                CharD => 'D',
                CharE => 'E',
                CharF => 'F',
                CharG => 'G',
                CharH => 'H',
                CharI => 'I',
                CharJ => 'J',
                CharK => 'K',
                CharL => 'L',
                CharM => 'M',
                CharN => 'N',
                CharO => 'O',
                CharP => 'P',
                CharQ => 'Q',
                CharR => 'R',
                CharS => 'S',
                CharT => 'T',
                CharU => 'U',
                CharV => 'V',
                CharW => 'W',
                CharX => 'X',
                CharY => 'Y',
                CharZ => 'Z',
                SymbolPlus => '+', // This is a symbol on the keypad
                SymbolMinus => '_',
                SymbolEquals => '+',
                SymbolOpenSquareBracket => '{',
                SymbolCloseSquareBracket => '}',
                SymbolSemicolon => ':',
                SymbolSingleQuote => '\"',
                SymbolBacktick => '~',
                SymbolBackslash => '|',
                SymbolComma => '<',
                SymbolPeriod => '>',
                SymbolForwardSlash => '?',
                SymbolAsterisk => '*', // This is a symbol on the keypad
                Space => ' ',
                Tab => '\t',
                _ => {
                    if key.keypad() {
                        match key.scan_type() {
                            // Ignore numbers when shifted on the keypad
                            _ => return None,
                        }
                    } else {
                        match key.scan_type() {
                            Num0 => ')',
                            Num1 => '!',
                            Num2 => '@',
                            Num3 => '#',
                            Num4 => '$',
                            Num5 => '%',
                            Num6 => '^',
                            Num7 => '&',
                            Num8 => '*',
                            Num9 => '(',
                            _ => return None,
                        }
                    }
                }
            };
        } else {
            ch = match key.scan_type() {
                Num0 => '0',
                Num1 => '1',
                Num2 => '2',
                Num3 => '3',
                Num4 => '4',
                Num5 => '5',
                Num6 => '6',
                Num7 => '7',
                Num8 => '8',
                Num9 => '9',
                CharA => 'a',
                CharB => 'b',
                CharC => 'c',
                CharD => 'd',
                CharE => 'e',
                CharF => 'f',
                CharG => 'g',
                CharH => 'h',
                CharI => 'i',
                CharJ => 'j',
                CharK => 'k',
                CharL => 'l',
                CharM => 'm',
                CharN => 'n',
                CharO => 'o',
                CharP => 'p',
                CharQ => 'q',
                CharR => 'r',
                CharS => 's',
                CharT => 't',
                CharU => 'u',
                CharV => 'v',
                CharW => 'w',
                CharX => 'x',
                CharY => 'y',
                CharZ => 'z',
                SymbolPlus => '+', // This is a symbol on the keypad
                SymbolMinus => '-',
                SymbolEquals => '=',
                SymbolOpenSquareBracket => '[',
                SymbolCloseSquareBracket => ']',
                SymbolSemicolon => ';',
                SymbolSingleQuote => '\'',
                SymbolBacktick => '`',
                SymbolBackslash => '\\',
                SymbolComma => ',',
                SymbolPeriod => '.',
                SymbolForwardSlash => '/',
                SymbolAsterisk => '*', // This is a symbol on the keypad
                Space => ' ',
                Tab => '\t',
                _ => return None,
            };
        }

        if modifiers.caps_lock && ch.is_alphabetic() {
            // Invert case
            if ch.is_ascii_lowercase() {
                return Some(ch.to_ascii_uppercase());
            } else if ch.is_ascii_uppercase() {
                // If for some reason ch is not ASCII we ignore it.
                return Some(ch.to_ascii_lowercase());
            }
        }

        return Some(ch);
    }
}
