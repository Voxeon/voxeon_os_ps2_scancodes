#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum ScanType {
    // Numbers
    Num0 = 0, // 0-9
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,

    // Letters
    CharA,
    CharB,
    CharC,
    CharD,
    CharE,
    CharF,
    CharG,
    CharH,
    CharI,
    CharJ,
    CharK,
    CharL,
    CharM,
    CharN,
    CharO,
    CharP,
    CharQ,
    CharR,
    CharS,
    CharT,
    CharU,
    CharV,
    CharW,
    CharX,
    CharY,
    CharZ,

    //Symbols
    SymbolPlus,
    SymbolMinus,
    SymbolEquals,
    SymbolOpenSquareBracket,
    SymbolCloseSquareBracket,
    SymbolSemicolon,
    SymbolSingleQuote,
    SymbolBacktick,
    SymbolBackslash,
    SymbolComma,
    SymbolPeriod,
    SymbolForwardSlash,
    SymbolAsterisk,

    // Control keys
    Escape,
    Backspace,
    Tab,
    Enter,
    LeftCtrl,
    RightCtrl,
    LeftShift,
    RightShift,
    LeftAlt,
    RightAlt,
    LeftGUI,
    RightGUI,
    Space,

    // Function Keys
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,

    // Locks
    NumLock,
    ScrollLock,
    CapsLock,

    Home,

    // Paging
    PageUp,
    PageDown,

    // Arrow Keys
    CursorUp,
    CursorLeft,
    CursorRight,
    CursorDown,

    Insert,
    Delete,
    End,

    // Utility keys
    ACPIPower,
    ACPISleep,
    ACPIWake,

    // Multimedia keys
    PreviousTrack,
    NextTrack,
    Mute,
    Calculator,
    Stop,
    Play,
    WWWHome,
    VolumeUp,
    VolumeDown,
    Apps,
    WWWSearch,
    WWWFavorites,
    WWWRefresh,
    WWWStop,
    WWWForward,
    WWWBack,
    MyComputer,
    Email,
    MediaSelect,
    PrintScreen,
    Pause,

    // Reserved key for any unknown key
    Unknown = 0xFF,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum KeyState {
    Pressed,
    Released,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Key {
    scan_type: ScanType,
    state: KeyState,
    keypad: bool,
}

impl Key {
    pub fn new(scan_type: ScanType, state: KeyState) -> Self {
        return Self {
            scan_type,
            state,
            keypad: false,
        };
    }

    pub fn new_keypad(scan_type: ScanType, state: KeyState) -> Self {
        return Self {
            scan_type,
            state,
            keypad: true,
        };
    }

    #[inline]
    pub fn scan_type(&self) -> ScanType {
        return self.scan_type;
    }

    #[inline]
    pub fn state(&self) -> KeyState {
        return self.state;
    }

    #[inline]
    pub fn keypad(&self) -> bool {
        return self.keypad;
    }

    #[inline]
    pub fn inverted_state(mut self) -> Self {
        if self.state == KeyState::Pressed {
            self.state = KeyState::Released;
        } else {
            self.state = KeyState::Pressed;
        }

        return self;
    }

    #[inline]
    pub fn is_pressed(&self) -> bool {
        return self.state == KeyState::Pressed;
    }
}

impl ScanType {
    #[inline]
    pub fn as_u8(self) -> u8 {
        return self as u8;
    }

    #[inline]
    pub fn is_letter(&self) -> bool {
        return *self >= ScanType::CharA && *self <= ScanType::CharZ;
    }

    #[inline]
    pub fn is_num(&self) -> bool {
        return *self >= ScanType::Num0 && *self <= ScanType::Num9;
    }
}
