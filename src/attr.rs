//! Video attributes for ncurses-pure.
//!
//! This module defines text attributes like bold, underline, reverse video, etc.
//! These match the X/Open XSI Curses standard attributes.

use crate::types::{AttrT, ChType};

/// Attribute shift - characters occupy bits 0-7.
pub const NCURSES_ATTR_SHIFT: u32 = 8;

/// Helper function for attribute bit positioning.
#[inline]
pub const fn ncurses_bits(mask: u32, shift: u32) -> ChType {
    (mask as ChType) << (shift + NCURSES_ATTR_SHIFT)
}

// ============================================================================
// Standard X/Open Curses Attributes
// ============================================================================

/// Normal display (no attributes).
pub const A_NORMAL: AttrT = 0;

/// Mask for extracting the character portion of a chtype.
pub const A_CHARTEXT: AttrT = (1 << NCURSES_ATTR_SHIFT) - 1;

/// Mask for extracting the color pair portion of a chtype.
pub const A_COLOR: AttrT = ((1 << 8) - 1) << NCURSES_ATTR_SHIFT;

/// Mask for extracting all attributes (everything except the character).
pub const A_ATTRIBUTES: AttrT = !A_CHARTEXT;

/// Standout mode (typically reverse video).
pub const A_STANDOUT: AttrT = ncurses_bits(1, 8);

/// Underline mode.
pub const A_UNDERLINE: AttrT = ncurses_bits(1, 9);

/// Reverse video mode.
pub const A_REVERSE: AttrT = ncurses_bits(1, 10);

/// Blinking text.
pub const A_BLINK: AttrT = ncurses_bits(1, 11);

/// Half-bright or dim text.
pub const A_DIM: AttrT = ncurses_bits(1, 12);

/// Bold or extra-bright text.
pub const A_BOLD: AttrT = ncurses_bits(1, 13);

/// Alternate character set (line drawing characters).
pub const A_ALTCHARSET: AttrT = ncurses_bits(1, 14);

/// Invisible text.
pub const A_INVIS: AttrT = ncurses_bits(1, 15);

/// Protected text (cannot be modified).
pub const A_PROTECT: AttrT = ncurses_bits(1, 16);

// X/Open features not found in SVr4 curses

/// Horizontal highlight (rarely supported).
pub const A_HORIZONTAL: AttrT = ncurses_bits(1, 17);

/// Left highlight (rarely supported).
pub const A_LEFT: AttrT = ncurses_bits(1, 18);

/// Low highlight (rarely supported).
pub const A_LOW: AttrT = ncurses_bits(1, 19);

/// Right highlight (rarely supported).
pub const A_RIGHT: AttrT = ncurses_bits(1, 20);

/// Top highlight (rarely supported).
pub const A_TOP: AttrT = ncurses_bits(1, 21);

/// Vertical highlight (rarely supported).
pub const A_VERTICAL: AttrT = ncurses_bits(1, 22);

/// Italic text (ncurses extension, widely supported).
pub const A_ITALIC: AttrT = ncurses_bits(1, 23);

// ============================================================================
// X/Open Wide-Character Attributes (WA_* aliases)
// ============================================================================

/// Normal display (wide-char alias).
pub const WA_NORMAL: AttrT = A_NORMAL;

/// All attributes mask (wide-char alias).
pub const WA_ATTRIBUTES: AttrT = A_ATTRIBUTES;

/// Standout mode (wide-char alias).
pub const WA_STANDOUT: AttrT = A_STANDOUT;

/// Underline mode (wide-char alias).
pub const WA_UNDERLINE: AttrT = A_UNDERLINE;

/// Reverse video (wide-char alias).
pub const WA_REVERSE: AttrT = A_REVERSE;

/// Blinking (wide-char alias).
pub const WA_BLINK: AttrT = A_BLINK;

/// Dim (wide-char alias).
pub const WA_DIM: AttrT = A_DIM;

/// Bold (wide-char alias).
pub const WA_BOLD: AttrT = A_BOLD;

/// Alternate character set (wide-char alias).
pub const WA_ALTCHARSET: AttrT = A_ALTCHARSET;

/// Invisible (wide-char alias).
pub const WA_INVIS: AttrT = A_INVIS;

/// Protected (wide-char alias).
pub const WA_PROTECT: AttrT = A_PROTECT;

/// Horizontal highlight (wide-char alias).
pub const WA_HORIZONTAL: AttrT = A_HORIZONTAL;

/// Left highlight (wide-char alias).
pub const WA_LEFT: AttrT = A_LEFT;

/// Low highlight (wide-char alias).
pub const WA_LOW: AttrT = A_LOW;

/// Right highlight (wide-char alias).
pub const WA_RIGHT: AttrT = A_RIGHT;

/// Top highlight (wide-char alias).
pub const WA_TOP: AttrT = A_TOP;

/// Vertical highlight (wide-char alias).
pub const WA_VERTICAL: AttrT = A_VERTICAL;

/// Italic (wide-char alias).
pub const WA_ITALIC: AttrT = A_ITALIC;

// ============================================================================
// Color pair helpers
// ============================================================================

/// Create a color attribute from a color pair number.
///
/// This function encodes a color pair number into the attribute bits
/// that can be OR'd with other attributes.
///
/// # Arguments
///
/// * `n` - The color pair number (1-255 for standard, higher with ext-colors)
///
/// # Example
///
/// ```rust
/// use ncurses::attr::{color_pair, A_BOLD};
///
/// let attr = color_pair(1) | A_BOLD;
/// ```
#[inline]
pub const fn color_pair(n: i16) -> AttrT {
    ncurses_bits(n as u32, 0) & A_COLOR
}

/// Extract the color pair number from an attribute value.
///
/// # Arguments
///
/// * `attr` - The attribute value containing a color pair
///
/// # Returns
///
/// The color pair number (0-255 for standard colors)
#[inline]
pub const fn pair_number(attr: AttrT) -> i16 {
    ((attr & A_COLOR) >> NCURSES_ATTR_SHIFT) as i16
}

// ============================================================================
// Attribute type for idiomatic Rust usage
// ============================================================================

bitflags::bitflags! {
    /// Video attributes as a bitflags type.
    ///
    /// This provides a more Rust-idiomatic interface for working with
    /// video attributes compared to raw `AttrT` values.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    pub struct Attribute: AttrT {
        /// Normal display (no attributes).
        const NORMAL = A_NORMAL;
        /// Standout mode.
        const STANDOUT = A_STANDOUT;
        /// Underline mode.
        const UNDERLINE = A_UNDERLINE;
        /// Reverse video.
        const REVERSE = A_REVERSE;
        /// Blinking text.
        const BLINK = A_BLINK;
        /// Dim or half-bright.
        const DIM = A_DIM;
        /// Bold or extra-bright.
        const BOLD = A_BOLD;
        /// Alternate character set.
        const ALTCHARSET = A_ALTCHARSET;
        /// Invisible text.
        const INVIS = A_INVIS;
        /// Protected text.
        const PROTECT = A_PROTECT;
        /// Horizontal highlight.
        const HORIZONTAL = A_HORIZONTAL;
        /// Left highlight.
        const LEFT = A_LEFT;
        /// Low highlight.
        const LOW = A_LOW;
        /// Right highlight.
        const RIGHT = A_RIGHT;
        /// Top highlight.
        const TOP = A_TOP;
        /// Vertical highlight.
        const VERTICAL = A_VERTICAL;
        /// Italic text.
        const ITALIC = A_ITALIC;
    }
}

impl Attribute {
    /// Create an attribute with a color pair.
    pub fn with_color_pair(self, pair: i16) -> AttrT {
        self.bits() | color_pair(pair)
    }
}

impl From<AttrT> for Attribute {
    fn from(attr: AttrT) -> Self {
        // Extract only the attribute bits, ignoring color and character
        Attribute::from_bits_truncate(attr & !A_COLOR & !A_CHARTEXT)
    }
}

impl From<Attribute> for AttrT {
    fn from(attr: Attribute) -> Self {
        attr.bits()
    }
}

// ============================================================================
// Character extraction helpers
// ============================================================================

/// Extract the character portion from a chtype.
#[inline]
pub const fn chtype_char(ch: ChType) -> u8 {
    (ch & A_CHARTEXT) as u8
}

/// Extract the attribute portion from a chtype (excluding character).
#[inline]
pub const fn chtype_attr(ch: ChType) -> AttrT {
    ch & A_ATTRIBUTES
}

/// Create a chtype from a character and attributes.
#[inline]
pub const fn make_chtype(ch: u8, attr: AttrT) -> ChType {
    (ch as ChType) | attr
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attribute_bits() {
        assert_eq!(A_NORMAL, 0);
        assert_eq!(A_CHARTEXT, 0xFF);
        assert!(A_STANDOUT > A_CHARTEXT);
        assert!(A_BOLD > A_STANDOUT);
    }

    #[test]
    fn test_color_pair() {
        let pair1 = color_pair(1);
        assert_ne!(pair1, 0);
        assert_eq!(pair_number(pair1), 1);

        let pair255 = color_pair(255);
        assert_eq!(pair_number(pair255), 255);
    }

    #[test]
    fn test_chtype_helpers() {
        let ch = make_chtype(b'A', A_BOLD | color_pair(1));
        assert_eq!(chtype_char(ch), b'A');
        assert_ne!(chtype_attr(ch), 0);
    }

    #[test]
    fn test_attribute_bitflags() {
        let attr = Attribute::BOLD | Attribute::UNDERLINE;
        assert!(attr.contains(Attribute::BOLD));
        assert!(attr.contains(Attribute::UNDERLINE));
        assert!(!attr.contains(Attribute::REVERSE));
    }
}
