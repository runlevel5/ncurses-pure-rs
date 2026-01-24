//! Wide character support for ncurses-rs.
//!
//! This module provides wide character support, allowing Unicode characters
//! and combining characters to be displayed. This matches the X/Open XSI
//! Curses wide character extensions.

use crate::attr::{A_CHARTEXT, A_NORMAL};
use crate::types::{AttrT, CCHARW_MAX};
use std::fmt;

/// Complex character type for wide character support.
///
/// A `cchar_t` stores an array of wide characters (up to `CCHARW_MAX`).
/// The first character is normally a spacing character, and the rest are
/// combining (non-spacing) characters.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CCharT {
    /// Attributes for this character cell.
    pub attr: AttrT,
    /// Array of wide characters.
    ///
    /// - `chars[0]` is the spacing character
    /// - `chars[1..]` are combining characters
    /// - A null character terminates the sequence
    pub chars: [char; CCHARW_MAX],
    /// Extended color pair (when ext-colors feature is enabled).
    #[cfg(feature = "ext-colors")]
    pub ext_color: i32,
}

impl CCharT {
    /// Create an empty (null) complex character.
    pub const fn new() -> Self {
        Self {
            attr: A_NORMAL,
            chars: ['\0'; CCHARW_MAX],
            #[cfg(feature = "ext-colors")]
            ext_color: 0,
        }
    }

    /// Create a complex character from a single character and attributes.
    pub fn from_char_attr(ch: char, attr: AttrT) -> Self {
        let mut chars = ['\0'; CCHARW_MAX];
        chars[0] = ch;
        Self {
            attr,
            chars,
            #[cfg(feature = "ext-colors")]
            ext_color: 0,
        }
    }

    /// Create a complex character from a single character with default attributes.
    pub fn from_char(ch: char) -> Self {
        Self::from_char_attr(ch, A_NORMAL)
    }

    /// Create a space character with default attributes.
    pub fn space() -> Self {
        Self::from_char(' ')
    }

    /// Get the spacing (primary) character.
    #[inline]
    pub fn spacing_char(&self) -> char {
        self.chars[0]
    }

    /// Check if this is a null (empty) character.
    #[inline]
    pub fn is_null(&self) -> bool {
        self.chars[0] == '\0'
    }

    /// Get the number of characters (including combining).
    pub fn char_count(&self) -> usize {
        self.chars.iter().take_while(|&&c| c != '\0').count()
    }

    /// Add a combining character.
    ///
    /// Returns `true` if the character was added, `false` if there's no room.
    pub fn add_combining(&mut self, ch: char) -> bool {
        for i in 1..CCHARW_MAX {
            if self.chars[i] == '\0' {
                self.chars[i] = ch;
                return true;
            }
        }
        false
    }

    /// Get the attributes.
    #[inline]
    pub fn attrs(&self) -> AttrT {
        self.attr & !A_CHARTEXT
    }

    /// Set the attributes.
    #[inline]
    pub fn set_attrs(&mut self, attr: AttrT) {
        self.attr = attr;
    }

    /// Get the display width of this character.
    ///
    /// Uses Unicode width calculations to determine how many columns
    /// the character occupies on the terminal.
    pub fn width(&self) -> usize {
        if self.chars[0] == '\0' {
            0
        } else {
            unicode_width::UnicodeWidthChar::width(self.chars[0]).unwrap_or(1)
        }
    }

    /// Check if this is a wide character (takes 2 columns).
    pub fn is_wide(&self) -> bool {
        self.width() > 1
    }

    /// Convert to a string representation.
    fn as_string(&self) -> String {
        self.chars.iter().take_while(|&&c| c != '\0').collect()
    }
}

impl fmt::Display for CCharT {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

impl From<char> for CCharT {
    fn from(ch: char) -> Self {
        Self::from_char(ch)
    }
}

impl From<(char, AttrT)> for CCharT {
    fn from((ch, attr): (char, AttrT)) -> Self {
        Self::from_char_attr(ch, attr)
    }
}

/// Wide character result from input operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WideInput {
    /// A wide character was read.
    Char(char),
    /// A key code was read.
    Key(i32),
    /// No input available (non-blocking).
    None,
    /// End of file.
    Eof,
    /// An error occurred.
    Error,
}

/// Convert a string to a vector of CCharT.
pub fn str_to_cchars(s: &str, attr: AttrT) -> Vec<CCharT> {
    s.chars().map(|c| CCharT::from_char_attr(c, attr)).collect()
}

/// Convert a vector of CCharT to a string.
pub fn cchars_to_string(chars: &[CCharT]) -> String {
    chars.iter().map(|c| c.spacing_char()).collect()
}

/// Get the display width of a string.
pub fn string_width(s: &str) -> usize {
    use unicode_width::UnicodeWidthStr;
    s.width()
}

/// Get the display width of a character.
pub fn char_width(ch: char) -> usize {
    unicode_width::UnicodeWidthChar::width(ch).unwrap_or(1)
}

/// Check if a character is a combining character.
pub fn is_combining(ch: char) -> bool {
    // Combining characters have a display width of 0
    unicode_width::UnicodeWidthChar::width(ch) == Some(0)
}

/// Wide character constants for line drawing.
///
/// These are the Unicode box-drawing characters, which provide
/// better-looking results than the ACS fallbacks on modern terminals.
pub mod wacs {
    use super::CCharT;
    use crate::attr::A_NORMAL;

    /// Helper to create line-drawing characters.
    const fn ld(ch: char) -> CCharT {
        let mut chars = ['\0'; crate::types::CCHARW_MAX];
        chars[0] = ch;
        CCharT {
            attr: A_NORMAL,
            chars,
            #[cfg(feature = "ext-colors")]
            ext_color: 0,
        }
    }

    /// Upper left corner (┌).
    pub const ULCORNER: CCharT = ld('┌');
    /// Lower left corner (└).
    pub const LLCORNER: CCharT = ld('└');
    /// Upper right corner (┐).
    pub const URCORNER: CCharT = ld('┐');
    /// Lower right corner (┘).
    pub const LRCORNER: CCharT = ld('┘');
    /// Tee pointing right (├).
    pub const LTEE: CCharT = ld('├');
    /// Tee pointing left (┤).
    pub const RTEE: CCharT = ld('┤');
    /// Tee pointing up (┴).
    pub const BTEE: CCharT = ld('┴');
    /// Tee pointing down (┬).
    pub const TTEE: CCharT = ld('┬');
    /// Horizontal line (─).
    pub const HLINE: CCharT = ld('─');
    /// Vertical line (│).
    pub const VLINE: CCharT = ld('│');
    /// Plus/crossover (┼).
    pub const PLUS: CCharT = ld('┼');
    /// Diamond (◆).
    pub const DIAMOND: CCharT = ld('◆');
    /// Checkerboard (░).
    pub const CKBOARD: CCharT = ld('░');
    /// Degree symbol (°).
    pub const DEGREE: CCharT = ld('°');
    /// Plus/minus (±).
    pub const PLMINUS: CCharT = ld('±');
    /// Bullet (·).
    pub const BULLET: CCharT = ld('·');
    /// Arrow pointing left (←).
    pub const LARROW: CCharT = ld('←');
    /// Arrow pointing right (→).
    pub const RARROW: CCharT = ld('→');
    /// Arrow pointing down (↓).
    pub const DARROW: CCharT = ld('↓');
    /// Arrow pointing up (↑).
    pub const UARROW: CCharT = ld('↑');
    /// Board of squares (▒).
    pub const BOARD: CCharT = ld('▒');
    /// Lantern symbol (§).
    pub const LANTERN: CCharT = ld('§');
    /// Solid square block (█).
    pub const BLOCK: CCharT = ld('█');
    /// Less than or equal (≤).
    pub const LEQUAL: CCharT = ld('≤');
    /// Greater than or equal (≥).
    pub const GEQUAL: CCharT = ld('≥');
    /// Pi (π).
    pub const PI: CCharT = ld('π');
    /// Not equal (≠).
    pub const NEQUAL: CCharT = ld('≠');
    /// UK pound sign (£).
    pub const STERLING: CCharT = ld('£');
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cchar_creation() {
        let ch = CCharT::from_char('A');
        assert_eq!(ch.spacing_char(), 'A');
        assert_eq!(ch.attrs(), A_NORMAL);
        assert_eq!(ch.char_count(), 1);
    }

    #[test]
    fn test_cchar_combining() {
        let mut ch = CCharT::from_char('e');
        assert!(ch.add_combining('\u{0301}')); // combining acute accent
        assert_eq!(ch.char_count(), 2);
        // The result is 'e' + combining acute (decomposed form), not precomposed 'é'
        assert_eq!(ch.to_string(), "e\u{0301}");
        // Verify it renders the same (both are canonically equivalent)
        assert_eq!(ch.spacing_char(), 'e');
    }

    #[test]
    fn test_cchar_width() {
        // ASCII character
        let ch = CCharT::from_char('A');
        assert_eq!(ch.width(), 1);
        assert!(!ch.is_wide());

        // Wide character (CJK)
        let ch = CCharT::from_char('漢');
        assert_eq!(ch.width(), 2);
        assert!(ch.is_wide());
    }

    #[test]
    fn test_str_conversion() {
        let s = "Hello";
        let cchars = str_to_cchars(s, A_NORMAL);
        assert_eq!(cchars.len(), 5);
        assert_eq!(cchars_to_string(&cchars), s);
    }

    #[test]
    fn test_string_width() {
        assert_eq!(string_width("Hello"), 5);
        assert_eq!(string_width("漢字"), 4); // 2 wide characters
        assert_eq!(string_width("Héllo"), 5); // with combining character
    }
}
