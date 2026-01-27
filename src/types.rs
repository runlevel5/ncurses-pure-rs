//! Core type definitions for ncurses-pure.
//!
//! This module defines the fundamental types used throughout the library,
//! matching the X/Open XSI Curses standard.

/// Character type with embedded attributes.
///
/// In ncurses, `chtype` is a 32-bit value where:
/// - Bits 0-7: The character (or character index)
/// - Bits 8-31: Attributes and color pair
///
/// This allows efficient storage of both the character and its display
/// attributes in a single value.
pub type ChType = u32;

/// Attribute type. Must be at least as wide as `ChType`.
///
/// Used for storing video attributes like bold, underline, reverse, etc.
pub type AttrT = ChType;

/// Mouse event mask type.
pub type MmaskT = u32;

/// Size type for window dimensions.
///
/// X/Open specifies this as `short` (16-bit signed integer).
pub type NcursesSize = i16;

/// Color value type.
///
/// X/Open uses `short` for color values, which are indices into
/// the terminal's color palette.
pub type ColorT = i16;

/// Color pair index type.
///
/// X/Open uses `short` for color pairs, allowing up to 32767 pairs
/// in the standard configuration. With extended colors, this limit
/// can be much higher.
pub type PairT = i16;

/// Window coordinate type.
///
/// Used for cursor positions and window coordinates.
pub type Coord = i32;

/// Boolean type for ncurses.
///
/// X/Open requires curses to define `bool`, but in Rust we just use
/// the native `bool` type.
pub type NcursesBool = bool;

/// OK return value (success).
pub const OK: i32 = 0;

/// ERR return value (failure).
pub const ERR: i32 = -1;

/// TRUE constant for compatibility.
pub const TRUE: NcursesBool = true;

/// FALSE constant for compatibility.
pub const FALSE: NcursesBool = false;

/// Marker for unchanged lines in the change tracking.
pub const NOCHANGE: NcursesSize = -1;

/// Marker for newly inserted lines.
pub const NEWINDEX: NcursesSize = -1;

bitflags::bitflags! {
    /// Window state flags indicating window properties and state.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    pub struct WindowFlags: u16 {
        /// This is a sub-window.
        const SUBWIN    = 0x01;
        /// The window is flush with the right edge of the screen.
        const ENDLINE   = 0x02;
        /// The window is full-screen.
        const FULLWIN   = 0x04;
        /// The bottom edge is at screen bottom.
        const SCROLLWIN = 0x08;
        /// This is a pad, not a regular window.
        const ISPAD     = 0x10;
        /// The cursor has moved since the last refresh.
        const HASMOVED  = 0x20;
        /// The cursor was just wrapped to the next line.
        const WRAPPED   = 0x40;
    }
}

/// The maximum number of wide characters in a `cchar_t`.
///
/// This includes one spacing character and up to `CCHARW_MAX - 1`
/// combining characters.
#[cfg(feature = "wide")]
pub const CCHARW_MAX: usize = 5;

/// Timeout values for input operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Delay {
    /// No delay - non-blocking input.
    NoDelay,
    /// Block indefinitely until input is available.
    #[default]
    Blocking,
    /// Wait for specified milliseconds.
    Timeout(i32),
}

impl Delay {
    /// Convert from the raw delay value used internally.
    #[must_use]
    pub fn from_raw(value: i32) -> Self {
        if value == 0 {
            Delay::NoDelay
        } else if value < 0 {
            Delay::Blocking
        } else {
            Delay::Timeout(value)
        }
    }

    /// Convert to the raw delay value used internally.
    #[must_use]
    pub fn to_raw(self) -> i32 {
        match self {
            Delay::NoDelay => 0,
            Delay::Blocking => -1,
            Delay::Timeout(ms) => ms,
        }
    }
}

/// Cursor visibility states.
///
/// Controls how the cursor is displayed on the terminal.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum CursorVisibility {
    /// Cursor is invisible (hidden).
    Invisible = 0,
    /// Normal cursor visibility (default).
    #[default]
    Normal = 1,
    /// Very visible cursor (e.g., block cursor).
    VeryVisible = 2,
}

impl CursorVisibility {
    /// Create from raw i32 value.
    ///
    /// Returns `None` if the value is not a valid cursor visibility.
    #[must_use]
    pub fn from_raw(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Invisible),
            1 => Some(Self::Normal),
            2 => Some(Self::VeryVisible),
            _ => None,
        }
    }

    /// Convert to raw i32 value.
    #[must_use]
    pub fn to_raw(self) -> i32 {
        self as i32
    }
}

impl TryFrom<i32> for CursorVisibility {
    type Error = crate::error::Error;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Self::from_raw(value).ok_or_else(|| {
            crate::error::Error::InvalidArgument(format!(
                "cursor visibility must be 0, 1, or 2, got {}",
                value
            ))
        })
    }
}

impl From<CursorVisibility> for i32 {
    fn from(v: CursorVisibility) -> Self {
        v.to_raw()
    }
}

/// Border characters for drawing window borders.
///
/// This struct provides a more ergonomic way to specify border characters
/// compared to passing 8 separate parameters.
///
/// # Example
///
/// ```rust
/// use ncurses::types::BorderChars;
///
/// // Use default box-drawing characters
/// let border = BorderChars::default();
///
/// // Create a simple border with same chars for sides
/// let border = BorderChars::simple('|' as u32, '-' as u32);
///
/// // Customize specific characters
/// let border = BorderChars::default()
///     .with_corners('+' as u32, '+' as u32, '+' as u32, '+' as u32);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BorderChars {
    /// Left side character.
    pub left: ChType,
    /// Right side character.
    pub right: ChType,
    /// Top side character.
    pub top: ChType,
    /// Bottom side character.
    pub bottom: ChType,
    /// Top-left corner character.
    pub top_left: ChType,
    /// Top-right corner character.
    pub top_right: ChType,
    /// Bottom-left corner character.
    pub bottom_left: ChType,
    /// Bottom-right corner character.
    pub bottom_right: ChType,
}

impl Default for BorderChars {
    /// Creates border with all characters set to 0, which means
    /// the window will use default ACS line-drawing characters.
    fn default() -> Self {
        Self {
            left: 0,
            right: 0,
            top: 0,
            bottom: 0,
            top_left: 0,
            top_right: 0,
            bottom_left: 0,
            bottom_right: 0,
        }
    }
}

impl BorderChars {
    /// Create a border with the same character for vertical sides
    /// and horizontal sides.
    #[must_use]
    pub const fn simple(vertical: ChType, horizontal: ChType) -> Self {
        Self {
            left: vertical,
            right: vertical,
            top: horizontal,
            bottom: horizontal,
            top_left: 0,
            top_right: 0,
            bottom_left: 0,
            bottom_right: 0,
        }
    }

    /// Create a border with the same character for all sides.
    #[must_use]
    pub const fn uniform(ch: ChType) -> Self {
        Self {
            left: ch,
            right: ch,
            top: ch,
            bottom: ch,
            top_left: ch,
            top_right: ch,
            bottom_left: ch,
            bottom_right: ch,
        }
    }

    /// Set corner characters.
    #[must_use]
    pub const fn with_corners(
        mut self,
        top_left: ChType,
        top_right: ChType,
        bottom_left: ChType,
        bottom_right: ChType,
    ) -> Self {
        self.top_left = top_left;
        self.top_right = top_right;
        self.bottom_left = bottom_left;
        self.bottom_right = bottom_right;
        self
    }

    /// Set side characters.
    #[must_use]
    pub const fn with_sides(
        mut self,
        left: ChType,
        right: ChType,
        top: ChType,
        bottom: ChType,
    ) -> Self {
        self.left = left;
        self.right = right;
        self.top = top;
        self.bottom = bottom;
        self
    }
}

/// Position in a 2D coordinate system.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Position {
    /// Y coordinate (row).
    pub y: Coord,
    /// X coordinate (column).
    pub x: Coord,
}

impl Position {
    /// Create a new position.
    #[must_use]
    pub const fn new(y: Coord, x: Coord) -> Self {
        Self { y, x }
    }
}

impl From<(Coord, Coord)> for Position {
    fn from((y, x): (Coord, Coord)) -> Self {
        Self { y, x }
    }
}

impl From<Position> for (Coord, Coord) {
    fn from(pos: Position) -> Self {
        (pos.y, pos.x)
    }
}

/// Size dimensions.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Size {
    /// Height (number of rows).
    pub height: Coord,
    /// Width (number of columns).
    pub width: Coord,
}

impl Size {
    /// Create a new size.
    #[must_use]
    pub const fn new(height: Coord, width: Coord) -> Self {
        Self { height, width }
    }
}

impl From<(Coord, Coord)> for Size {
    fn from((height, width): (Coord, Coord)) -> Self {
        Self { height, width }
    }
}

impl From<Size> for (Coord, Coord) {
    fn from(size: Size) -> Self {
        (size.height, size.width)
    }
}

/// A rectangular region on the screen.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Rect {
    /// Top-left position.
    pub origin: Position,
    /// Size of the rectangle.
    pub size: Size,
}

impl Rect {
    /// Create a new rectangle.
    #[must_use]
    pub const fn new(y: Coord, x: Coord, height: Coord, width: Coord) -> Self {
        Self {
            origin: Position::new(y, x),
            size: Size::new(height, width),
        }
    }

    /// Get the top Y coordinate.
    #[must_use]
    pub const fn top(&self) -> Coord {
        self.origin.y
    }

    /// Get the left X coordinate.
    #[must_use]
    pub const fn left(&self) -> Coord {
        self.origin.x
    }

    /// Get the bottom Y coordinate (exclusive).
    #[must_use]
    pub const fn bottom(&self) -> Coord {
        self.origin.y + self.size.height
    }

    /// Get the right X coordinate (exclusive).
    #[must_use]
    pub const fn right(&self) -> Coord {
        self.origin.x + self.size.width
    }

    /// Check if a position is within this rectangle.
    #[must_use]
    pub const fn contains(&self, pos: Position) -> bool {
        pos.y >= self.origin.y
            && pos.y < self.origin.y + self.size.height
            && pos.x >= self.origin.x
            && pos.x < self.origin.x + self.size.width
    }
}
