//! Key code definitions for ncurses-rs.
//!
//! This module defines the key codes returned by `getch()` for special keys
//! like function keys, arrow keys, etc. These match the X/Open XSI Curses
//! standard key definitions.

/// Indicates that a wchar_t contains a key code (not a character).
pub const KEY_CODE_YES: i32 = 0o400;

/// Minimum curses key value.
pub const KEY_MIN: i32 = 0o401;

/// Break key (unreliable).
pub const KEY_BREAK: i32 = 0o401;

/// Down arrow key.
pub const KEY_DOWN: i32 = 0o402;

/// Up arrow key.
pub const KEY_UP: i32 = 0o403;

/// Left arrow key.
pub const KEY_LEFT: i32 = 0o404;

/// Right arrow key.
pub const KEY_RIGHT: i32 = 0o405;

/// Home key.
pub const KEY_HOME: i32 = 0o406;

/// Backspace key.
pub const KEY_BACKSPACE: i32 = 0o407;

/// Function key F0.
pub const KEY_F0: i32 = 0o410;

/// Function key F(n) - use KEY_F(n) macro.
#[inline]
pub const fn key_f(n: i32) -> i32 {
    KEY_F0 + n
}

/// Delete line key.
pub const KEY_DL: i32 = 0o510;

/// Insert line key.
pub const KEY_IL: i32 = 0o511;

/// Delete character key.
pub const KEY_DC: i32 = 0o512;

/// Insert character key (enter insert mode).
pub const KEY_IC: i32 = 0o513;

/// Exit insert char mode key.
pub const KEY_EIC: i32 = 0o514;

/// Clear screen key.
pub const KEY_CLEAR: i32 = 0o515;

/// Clear to end of screen key.
pub const KEY_EOS: i32 = 0o516;

/// Clear to end of line key.
pub const KEY_EOL: i32 = 0o517;

/// Scroll forward key.
pub const KEY_SF: i32 = 0o520;

/// Scroll reverse key.
pub const KEY_SR: i32 = 0o521;

/// Next page key (Page Down).
pub const KEY_NPAGE: i32 = 0o522;

/// Previous page key (Page Up).
pub const KEY_PPAGE: i32 = 0o523;

/// Set tab key.
pub const KEY_STAB: i32 = 0o524;

/// Clear tab key.
pub const KEY_CTAB: i32 = 0o525;

/// Clear all tabs key.
pub const KEY_CATAB: i32 = 0o526;

/// Enter/send key.
pub const KEY_ENTER: i32 = 0o527;

/// Soft (partial) reset (unreliable).
pub const KEY_SRESET: i32 = 0o530;

/// Reset or hard reset (unreliable).
pub const KEY_RESET: i32 = 0o531;

/// Print key.
pub const KEY_PRINT: i32 = 0o532;

/// Lower-left key (home down).
pub const KEY_LL: i32 = 0o533;

/// Upper left of keypad.
pub const KEY_A1: i32 = 0o534;

/// Upper right of keypad.
pub const KEY_A3: i32 = 0o535;

/// Center of keypad.
pub const KEY_B2: i32 = 0o536;

/// Lower left of keypad.
pub const KEY_C1: i32 = 0o537;

/// Lower right of keypad.
pub const KEY_C3: i32 = 0o540;

/// Back tab key.
pub const KEY_BTAB: i32 = 0o541;

/// Beginning key.
pub const KEY_BEG: i32 = 0o542;

/// Cancel key.
pub const KEY_CANCEL: i32 = 0o543;

/// Close key.
pub const KEY_CLOSE: i32 = 0o544;

/// Command key.
pub const KEY_COMMAND: i32 = 0o545;

/// Copy key.
pub const KEY_COPY: i32 = 0o546;

/// Create key.
pub const KEY_CREATE: i32 = 0o547;

/// End key.
pub const KEY_END: i32 = 0o550;

/// Exit key.
pub const KEY_EXIT: i32 = 0o551;

/// Find key.
pub const KEY_FIND: i32 = 0o552;

/// Help key.
pub const KEY_HELP: i32 = 0o553;

/// Mark key.
pub const KEY_MARK: i32 = 0o554;

/// Message key.
pub const KEY_MESSAGE: i32 = 0o555;

/// Move key.
pub const KEY_MOVE: i32 = 0o556;

/// Next key.
pub const KEY_NEXT: i32 = 0o557;

/// Open key.
pub const KEY_OPEN: i32 = 0o560;

/// Options key.
pub const KEY_OPTIONS: i32 = 0o561;

/// Previous key.
pub const KEY_PREVIOUS: i32 = 0o562;

/// Redo key.
pub const KEY_REDO: i32 = 0o563;

/// Reference key.
pub const KEY_REFERENCE: i32 = 0o564;

/// Refresh key.
pub const KEY_REFRESH: i32 = 0o565;

/// Replace key.
pub const KEY_REPLACE: i32 = 0o566;

/// Restart key.
pub const KEY_RESTART: i32 = 0o567;

/// Resume key.
pub const KEY_RESUME: i32 = 0o570;

/// Save key.
pub const KEY_SAVE: i32 = 0o571;

/// Shifted beginning key.
pub const KEY_SBEG: i32 = 0o572;

/// Shifted cancel key.
pub const KEY_SCANCEL: i32 = 0o573;

/// Shifted command key.
pub const KEY_SCOMMAND: i32 = 0o574;

/// Shifted copy key.
pub const KEY_SCOPY: i32 = 0o575;

/// Shifted create key.
pub const KEY_SCREATE: i32 = 0o576;

/// Shifted delete character key.
pub const KEY_SDC: i32 = 0o577;

/// Shifted delete line key.
pub const KEY_SDL: i32 = 0o600;

/// Select key.
pub const KEY_SELECT: i32 = 0o601;

/// Shifted end key.
pub const KEY_SEND: i32 = 0o602;

/// Shifted clear-to-end-of-line key.
pub const KEY_SEOL: i32 = 0o603;

/// Shifted exit key.
pub const KEY_SEXIT: i32 = 0o604;

/// Shifted find key.
pub const KEY_SFIND: i32 = 0o605;

/// Shifted help key.
pub const KEY_SHELP: i32 = 0o606;

/// Shifted home key.
pub const KEY_SHOME: i32 = 0o607;

/// Shifted insert character key.
pub const KEY_SIC: i32 = 0o610;

/// Shifted left arrow key.
pub const KEY_SLEFT: i32 = 0o611;

/// Shifted message key.
pub const KEY_SMESSAGE: i32 = 0o612;

/// Shifted move key.
pub const KEY_SMOVE: i32 = 0o613;

/// Shifted next key.
pub const KEY_SNEXT: i32 = 0o614;

/// Shifted options key.
pub const KEY_SOPTIONS: i32 = 0o615;

/// Shifted previous key.
pub const KEY_SPREVIOUS: i32 = 0o616;

/// Shifted print key.
pub const KEY_SPRINT: i32 = 0o617;

/// Shifted redo key.
pub const KEY_SREDO: i32 = 0o620;

/// Shifted replace key.
pub const KEY_SREPLACE: i32 = 0o621;

/// Shifted right arrow key.
pub const KEY_SRIGHT: i32 = 0o622;

/// Shifted resume key.
pub const KEY_SRSUME: i32 = 0o623;

/// Shifted save key.
pub const KEY_SSAVE: i32 = 0o624;

/// Shifted suspend key.
pub const KEY_SSUSPEND: i32 = 0o625;

/// Shifted undo key.
pub const KEY_SUNDO: i32 = 0o626;

/// Suspend key.
pub const KEY_SUSPEND: i32 = 0o627;

/// Undo key.
pub const KEY_UNDO: i32 = 0o630;

/// Mouse event (ncurses extension).
pub const KEY_MOUSE: i32 = 0o631;

/// Terminal resize event (ncurses extension).
pub const KEY_RESIZE: i32 = 0o632;

/// Maximum key value.
pub const KEY_MAX: i32 = 0o777;

/// Key enumeration for type-safe key handling.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Key {
    /// A regular character.
    Char(char),
    /// Down arrow.
    Down,
    /// Up arrow.
    Up,
    /// Left arrow.
    Left,
    /// Right arrow.
    Right,
    /// Home key.
    Home,
    /// End key.
    End,
    /// Backspace.
    Backspace,
    /// Delete character.
    Delete,
    /// Insert character.
    Insert,
    /// Page up.
    PageUp,
    /// Page down.
    PageDown,
    /// Enter/Return.
    Enter,
    /// Tab.
    Tab,
    /// Back-tab (Shift+Tab).
    BackTab,
    /// Escape.
    Escape,
    /// Function key (1-64).
    F(u8),
    /// Mouse event.
    Mouse,
    /// Terminal resize.
    Resize,
    /// Unknown key code.
    Unknown(i32),
}

impl Key {
    /// Convert from a raw key code.
    pub fn from_code(code: i32) -> Self {
        match code {
            // Regular ASCII characters
            0..=31 if code == 9 => Key::Tab,
            0..=31 if code == 10 || code == 13 => Key::Enter,
            0..=31 if code == 27 => Key::Escape,
            32..=126 => Key::Char(code as u8 as char),
            127 => Key::Backspace,

            // Extended characters (128-255) - treat as characters
            128..=255 => Key::Char(code as u8 as char),

            // Special keys
            KEY_DOWN => Key::Down,
            KEY_UP => Key::Up,
            KEY_LEFT => Key::Left,
            KEY_RIGHT => Key::Right,
            KEY_HOME => Key::Home,
            KEY_END => Key::End,
            KEY_BACKSPACE => Key::Backspace,
            KEY_DC => Key::Delete,
            KEY_IC => Key::Insert,
            KEY_PPAGE => Key::PageUp,
            KEY_NPAGE => Key::PageDown,
            KEY_ENTER => Key::Enter,
            KEY_BTAB => Key::BackTab,
            KEY_MOUSE => Key::Mouse,
            KEY_RESIZE => Key::Resize,

            // Function keys
            k if (KEY_F0..=KEY_F0 + 64).contains(&k) => Key::F((k - KEY_F0) as u8),

            // Unknown
            _ => Key::Unknown(code),
        }
    }

    /// Convert to a raw key code.
    pub fn to_code(self) -> i32 {
        match self {
            Key::Char(c) => c as i32,
            Key::Down => KEY_DOWN,
            Key::Up => KEY_UP,
            Key::Left => KEY_LEFT,
            Key::Right => KEY_RIGHT,
            Key::Home => KEY_HOME,
            Key::End => KEY_END,
            Key::Backspace => KEY_BACKSPACE,
            Key::Delete => KEY_DC,
            Key::Insert => KEY_IC,
            Key::PageUp => KEY_PPAGE,
            Key::PageDown => KEY_NPAGE,
            Key::Enter => KEY_ENTER,
            Key::Tab => 9,
            Key::BackTab => KEY_BTAB,
            Key::Escape => 27,
            Key::F(n) => KEY_F0 + n as i32,
            Key::Mouse => KEY_MOUSE,
            Key::Resize => KEY_RESIZE,
            Key::Unknown(code) => code,
        }
    }

    /// Check if this is a function key.
    pub fn is_function_key(self) -> bool {
        matches!(self, Key::F(_))
    }

    /// Check if this is a printable character.
    pub fn is_printable(self) -> bool {
        matches!(self, Key::Char(c) if c >= ' ')
    }
}

impl From<i32> for Key {
    fn from(code: i32) -> Self {
        Key::from_code(code)
    }
}

impl From<Key> for i32 {
    fn from(key: Key) -> Self {
        key.to_code()
    }
}

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Key::Char(c) if c.is_control() => write!(f, "^{}", (*c as u8 + b'@') as char),
            Key::Char(c) => write!(f, "{}", c),
            Key::Down => write!(f, "<Down>"),
            Key::Up => write!(f, "<Up>"),
            Key::Left => write!(f, "<Left>"),
            Key::Right => write!(f, "<Right>"),
            Key::Home => write!(f, "<Home>"),
            Key::End => write!(f, "<End>"),
            Key::Backspace => write!(f, "<Backspace>"),
            Key::Delete => write!(f, "<Delete>"),
            Key::Insert => write!(f, "<Insert>"),
            Key::PageUp => write!(f, "<PageUp>"),
            Key::PageDown => write!(f, "<PageDown>"),
            Key::Enter => write!(f, "<Enter>"),
            Key::Tab => write!(f, "<Tab>"),
            Key::BackTab => write!(f, "<BackTab>"),
            Key::Escape => write!(f, "<Escape>"),
            Key::F(n) => write!(f, "<F{}>", n),
            Key::Mouse => write!(f, "<Mouse>"),
            Key::Resize => write!(f, "<Resize>"),
            Key::Unknown(code) => write!(f, "<Unknown:{}>", code),
        }
    }
}

/// Get the name of a key code.
pub fn keyname(code: i32) -> &'static str {
    match code {
        0..=31 => match code {
            0 => "^@",
            1 => "^A",
            2 => "^B",
            3 => "^C",
            4 => "^D",
            5 => "^E",
            6 => "^F",
            7 => "^G",
            8 => "^H",
            9 => "^I",
            10 => "^J",
            11 => "^K",
            12 => "^L",
            13 => "^M",
            14 => "^N",
            15 => "^O",
            16 => "^P",
            17 => "^Q",
            18 => "^R",
            19 => "^S",
            20 => "^T",
            21 => "^U",
            22 => "^V",
            23 => "^W",
            24 => "^X",
            25 => "^Y",
            26 => "^Z",
            27 => "^[",
            28 => "^\\",
            29 => "^]",
            30 => "^^",
            31 => "^_",
            _ => "UNKNOWN",
        },
        32 => "SPACE",
        127 => "^?",
        KEY_DOWN => "KEY_DOWN",
        KEY_UP => "KEY_UP",
        KEY_LEFT => "KEY_LEFT",
        KEY_RIGHT => "KEY_RIGHT",
        KEY_HOME => "KEY_HOME",
        KEY_BACKSPACE => "KEY_BACKSPACE",
        KEY_DC => "KEY_DC",
        KEY_IC => "KEY_IC",
        KEY_NPAGE => "KEY_NPAGE",
        KEY_PPAGE => "KEY_PPAGE",
        KEY_ENTER => "KEY_ENTER",
        KEY_END => "KEY_END",
        KEY_BTAB => "KEY_BTAB",
        KEY_MOUSE => "KEY_MOUSE",
        KEY_RESIZE => "KEY_RESIZE",
        k if (KEY_F0..=KEY_F0 + 12).contains(&k) => match k - KEY_F0 {
            0 => "KEY_F(0)",
            1 => "KEY_F(1)",
            2 => "KEY_F(2)",
            3 => "KEY_F(3)",
            4 => "KEY_F(4)",
            5 => "KEY_F(5)",
            6 => "KEY_F(6)",
            7 => "KEY_F(7)",
            8 => "KEY_F(8)",
            9 => "KEY_F(9)",
            10 => "KEY_F(10)",
            11 => "KEY_F(11)",
            12 => "KEY_F(12)",
            _ => "KEY_F(?)",
        },
        _ => "UNKNOWN",
    }
}

/// Get a printable representation of a character.
///
/// This function returns a printable string representation of any character:
/// - Control characters (0-31) return "^X" notation (e.g., "^A" for 0x01)
/// - DEL (127) returns "^?"
/// - Printable ASCII (32-126) returns the character itself
/// - Characters >= 128 return "M-X" notation or the character if printable
///
/// This is the ncurses `unctrl()` function.
pub fn unctrl(ch: u32) -> String {
    let c = (ch & 0xFF) as u8;
    match c {
        0 => "^@".to_string(),
        1..=26 => format!("^{}", (b'A' + c - 1) as char),
        27 => "^[".to_string(),
        28 => "^\\".to_string(),
        29 => "^]".to_string(),
        30 => "^^".to_string(),
        31 => "^_".to_string(),
        32..=126 => (c as char).to_string(),
        127 => "^?".to_string(),
        128..=159 => format!("M-^{}", unctrl((c - 128) as u32)),
        160..=254 => format!("M-{}", (c - 128) as char),
        255 => "M-^?".to_string(),
    }
}

/// Get the name of a wide character key.
///
/// This is similar to `keyname` but handles wide characters.
/// For regular characters, returns the character as a string.
/// For key codes, returns the key name.
#[cfg(feature = "wide")]
pub fn key_name(wch: char) -> String {
    let code = wch as i32;
    if code > 255 {
        // It's a key code
        keyname(code).to_string()
    } else {
        // It's a character
        unctrl(code as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_f() {
        assert_eq!(key_f(1), KEY_F0 + 1);
        assert_eq!(key_f(12), KEY_F0 + 12);
    }

    #[test]
    fn test_key_enum() {
        assert_eq!(Key::from_code(KEY_UP), Key::Up);
        assert_eq!(Key::from_code(KEY_F0 + 1), Key::F(1));
        assert_eq!(Key::from_code(65), Key::Char('A'));

        assert_eq!(Key::Up.to_code(), KEY_UP);
        assert_eq!(Key::F(1).to_code(), KEY_F0 + 1);
    }

    #[test]
    fn test_keyname() {
        assert_eq!(keyname(KEY_UP), "KEY_UP");
        assert_eq!(keyname(KEY_F0 + 1), "KEY_F(1)");
        assert_eq!(keyname(27), "^[");
    }
}
