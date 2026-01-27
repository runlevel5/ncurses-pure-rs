//! Color support for ncurses-pure.
//!
//! This module provides color handling functionality matching the X/Open
//! XSI Curses standard. Colors are organized into pairs (foreground/background)
//! that can be applied to text output.

use crate::error::{Error, Result};
use crate::types::ColorT;

// ============================================================================
// Standard Colors
// ============================================================================

/// Black color.
pub const COLOR_BLACK: ColorT = 0;
/// Red color.
pub const COLOR_RED: ColorT = 1;
/// Green color.
pub const COLOR_GREEN: ColorT = 2;
/// Yellow color.
pub const COLOR_YELLOW: ColorT = 3;
/// Blue color.
pub const COLOR_BLUE: ColorT = 4;
/// Magenta color.
pub const COLOR_MAGENTA: ColorT = 5;
/// Cyan color.
pub const COLOR_CYAN: ColorT = 6;
/// White color.
pub const COLOR_WHITE: ColorT = 7;

/// Standard color enumeration.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(i16)]
pub enum Color {
    /// Black (color 0).
    Black = COLOR_BLACK,
    /// Red (color 1).
    Red = COLOR_RED,
    /// Green (color 2).
    Green = COLOR_GREEN,
    /// Yellow (color 3).
    Yellow = COLOR_YELLOW,
    /// Blue (color 4).
    Blue = COLOR_BLUE,
    /// Magenta (color 5).
    Magenta = COLOR_MAGENTA,
    /// Cyan (color 6).
    Cyan = COLOR_CYAN,
    /// White (color 7).
    White = COLOR_WHITE,
}

impl Color {
    /// Convert from a color index.
    pub fn from_index(index: ColorT) -> Option<Self> {
        match index {
            COLOR_BLACK => Some(Color::Black),
            COLOR_RED => Some(Color::Red),
            COLOR_GREEN => Some(Color::Green),
            COLOR_YELLOW => Some(Color::Yellow),
            COLOR_BLUE => Some(Color::Blue),
            COLOR_MAGENTA => Some(Color::Magenta),
            COLOR_CYAN => Some(Color::Cyan),
            COLOR_WHITE => Some(Color::White),
            _ => None,
        }
    }

    /// Convert to color index.
    pub const fn to_index(self) -> ColorT {
        self as ColorT
    }
}

impl From<Color> for ColorT {
    fn from(color: Color) -> Self {
        color as ColorT
    }
}

impl TryFrom<ColorT> for Color {
    type Error = Error;

    fn try_from(value: ColorT) -> Result<Self> {
        Color::from_index(value).ok_or(Error::InvalidColor(value))
    }
}

/// Default number of colors in a standard terminal.
pub const DEFAULT_COLORS: i32 = 8;

/// Default number of color pairs.
pub const DEFAULT_COLOR_PAIRS: i32 = 64;

/// Maximum RGB value for color definition.
pub const RGB_MAX: i16 = 1000;

// ============================================================================
// Color definition storage
// ============================================================================

/// RGB color definition.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ColorDef {
    /// Red component (0-1000).
    pub red: i16,
    /// Green component (0-1000).
    pub green: i16,
    /// Blue component (0-1000).
    pub blue: i16,
    /// Whether this color has been initialized with custom values.
    pub initialized: bool,
}

impl ColorDef {
    /// Create a new color definition.
    pub const fn new(red: i16, green: i16, blue: i16) -> Self {
        Self {
            red,
            green,
            blue,
            initialized: true,
        }
    }

    /// Create an uninitialized (default) color.
    pub const fn default_color() -> Self {
        Self {
            red: 0,
            green: 0,
            blue: 0,
            initialized: false,
        }
    }
}

/// Color pair definition (foreground and background).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ColorPair {
    /// Foreground color index.
    pub foreground: ColorT,
    /// Background color index.
    pub background: ColorT,
    /// Whether this pair has been initialized.
    pub initialized: bool,
}

impl ColorPair {
    /// Create a new color pair.
    pub const fn new(foreground: ColorT, background: ColorT) -> Self {
        Self {
            foreground,
            background,
            initialized: true,
        }
    }

    /// Create an uninitialized (default) color pair.
    pub const fn default_pair() -> Self {
        Self {
            foreground: COLOR_WHITE,
            background: COLOR_BLACK,
            initialized: false,
        }
    }
}

// ============================================================================
// Color Manager
// ============================================================================

/// Manages the color palette and color pairs.
pub struct ColorManager {
    /// Whether color support has been started.
    started: bool,
    /// Whether the terminal can change color definitions.
    can_change: bool,
    /// Number of available colors.
    num_colors: i32,
    /// Number of available color pairs.
    num_pairs: i32,
    /// Color definitions.
    colors: Vec<ColorDef>,
    /// Color pairs.
    pairs: Vec<ColorPair>,
    /// Whether to use default colors (-1 for default).
    use_default_colors: bool,
}

impl ColorManager {
    /// Create a new color manager.
    pub fn new(num_colors: i32, num_pairs: i32, can_change: bool) -> Self {
        let mut colors = vec![ColorDef::default_color(); num_colors as usize];
        let pairs = vec![ColorPair::default_pair(); num_pairs as usize];

        // Initialize standard colors with typical values
        if num_colors >= 8 {
            colors[0] = ColorDef::new(0, 0, 0); // Black
            colors[1] = ColorDef::new(680, 0, 0); // Red
            colors[2] = ColorDef::new(0, 680, 0); // Green
            colors[3] = ColorDef::new(680, 680, 0); // Yellow
            colors[4] = ColorDef::new(0, 0, 680); // Blue
            colors[5] = ColorDef::new(680, 0, 680); // Magenta
            colors[6] = ColorDef::new(0, 680, 680); // Cyan
            colors[7] = ColorDef::new(680, 680, 680); // White
        }

        Self {
            started: false,
            can_change,
            num_colors,
            num_pairs,
            colors,
            pairs,
            use_default_colors: false,
        }
    }

    /// Start color mode.
    pub fn start(&mut self) -> Result<()> {
        if self.num_colors <= 0 || self.num_pairs <= 0 {
            return Err(Error::ColorNotAvailable);
        }
        self.started = true;
        Ok(())
    }

    /// Check if colors have been started.
    pub fn is_started(&self) -> bool {
        self.started
    }

    /// Check if the terminal has color support.
    pub fn has_colors(&self) -> bool {
        self.num_colors > 0 && self.num_pairs > 0
    }

    /// Check if the terminal can change color definitions.
    pub fn can_change_color(&self) -> bool {
        self.can_change
    }

    /// Get the number of available colors.
    pub fn num_colors(&self) -> i32 {
        self.num_colors
    }

    /// Get the number of available color pairs.
    pub fn num_pairs(&self) -> i32 {
        self.num_pairs
    }

    /// Enable the use of default colors (-1 represents terminal default).
    pub fn use_default_colors(&mut self) -> Result<()> {
        if !self.started {
            return Err(Error::ColorNotAvailable);
        }
        self.use_default_colors = true;
        Ok(())
    }

    /// Initialize a color pair.
    pub fn init_pair(&mut self, pair: i16, fg: ColorT, bg: ColorT) -> Result<()> {
        if !self.started {
            return Err(Error::ColorNotAvailable);
        }
        if pair < 0 || pair as i32 >= self.num_pairs {
            return Err(Error::InvalidColorPair(pair));
        }
        if pair == 0 {
            // Color pair 0 is typically reserved
            return Err(Error::InvalidColorPair(0));
        }

        // Validate colors (-1 is allowed if use_default_colors is enabled)
        // Use i32 comparison to avoid overflow when num_colors > i16::MAX (e.g., true color terminals)
        let min_color: i32 = if self.use_default_colors { -1 } else { 0 };
        if (fg as i32) < min_color || (fg as i32) >= self.num_colors {
            return Err(Error::InvalidColor(fg));
        }
        if (bg as i32) < min_color || (bg as i32) >= self.num_colors {
            return Err(Error::InvalidColor(bg));
        }

        self.pairs[pair as usize] = ColorPair::new(fg, bg);
        Ok(())
    }

    /// Get the definition of a color pair.
    pub fn pair_content(&self, pair: i16) -> Result<(ColorT, ColorT)> {
        if !self.started {
            return Err(Error::ColorNotAvailable);
        }
        if pair < 0 || pair as i32 >= self.num_pairs {
            return Err(Error::InvalidColorPair(pair));
        }

        let cp = &self.pairs[pair as usize];
        Ok((cp.foreground, cp.background))
    }

    /// Initialize a color with RGB values.
    pub fn init_color(&mut self, color: ColorT, r: i16, g: i16, b: i16) -> Result<()> {
        if !self.started {
            return Err(Error::ColorNotAvailable);
        }
        if !self.can_change {
            return Err(Error::NotSupported("terminal cannot change colors".into()));
        }
        if (color as i32) < 0 || (color as i32) >= self.num_colors {
            return Err(Error::InvalidColor(color));
        }
        if !(0..=RGB_MAX).contains(&r) || !(0..=RGB_MAX).contains(&g) || !(0..=RGB_MAX).contains(&b)
        {
            return Err(Error::InvalidArgument("RGB values must be 0-1000".into()));
        }

        self.colors[color as usize] = ColorDef::new(r, g, b);
        Ok(())
    }

    /// Get the RGB definition of a color.
    pub fn color_content(&self, color: ColorT) -> Result<(i16, i16, i16)> {
        if !self.started {
            return Err(Error::ColorNotAvailable);
        }
        if (color as i32) < 0 || (color as i32) >= self.num_colors {
            return Err(Error::InvalidColor(color));
        }

        let def = &self.colors[color as usize];
        Ok((def.red, def.green, def.blue))
    }

    /// Reset all color pairs.
    pub fn reset_color_pairs(&mut self) {
        for pair in self.pairs.iter_mut() {
            *pair = ColorPair::default_pair();
        }
    }

    /// Set default foreground and background colors for pair 0.
    pub fn assume_default_colors(&mut self, fg: ColorT, bg: ColorT) -> Result<()> {
        if !self.started {
            return Err(Error::ColorNotAvailable);
        }

        // Unlike init_pair, this can modify pair 0
        self.pairs[0] = ColorPair::new(fg, bg);
        Ok(())
    }

    // ========================================================================
    // Extended color support (for >256 colors)
    // ========================================================================

    /// Initialize an extended color pair.
    ///
    /// This is similar to `init_pair` but supports color values beyond the
    /// standard 256-color limit when using terminals with extended color support.
    ///
    /// # Arguments
    ///
    /// * `pair` - The color pair number (can be larger than 256)
    /// * `fg` - Foreground color (can be > 256 for extended colors)
    /// * `bg` - Background color (can be > 256 for extended colors)
    #[cfg(feature = "ext-colors")]
    pub fn init_extended_pair(&mut self, pair: i32, fg: i32, bg: i32) -> Result<()> {
        if !self.started {
            return Err(Error::ColorNotAvailable);
        }
        if pair < 0 || pair >= self.num_pairs {
            return Err(Error::InvalidColorPair(pair as i16));
        }
        if pair == 0 {
            return Err(Error::InvalidColorPair(0));
        }

        let min_color = if self.use_default_colors { -1 } else { 0 };
        if fg < min_color || fg >= self.num_colors {
            return Err(Error::InvalidColor(fg as i16));
        }
        if bg < min_color || bg >= self.num_colors {
            return Err(Error::InvalidColor(bg as i16));
        }

        self.pairs[pair as usize] = ColorPair::new(fg as ColorT, bg as ColorT);
        Ok(())
    }

    /// Get the definition of an extended color pair.
    ///
    /// Returns the foreground and background colors as i32 to support
    /// extended color values.
    #[cfg(feature = "ext-colors")]
    pub fn extended_pair_content(&self, pair: i32) -> Result<(i32, i32)> {
        if !self.started {
            return Err(Error::ColorNotAvailable);
        }
        if pair < 0 || pair >= self.num_pairs {
            return Err(Error::InvalidColorPair(pair as i16));
        }

        let cp = &self.pairs[pair as usize];
        Ok((cp.foreground as i32, cp.background as i32))
    }

    /// Initialize an extended color with RGB values.
    ///
    /// This is similar to `init_color` but supports color indices beyond 256.
    #[cfg(feature = "ext-colors")]
    pub fn init_extended_color(&mut self, color: i32, r: i32, g: i32, b: i32) -> Result<()> {
        if !self.started {
            return Err(Error::ColorNotAvailable);
        }
        if !self.can_change {
            return Err(Error::NotSupported("terminal cannot change colors".into()));
        }
        if color < 0 || color >= self.num_colors {
            return Err(Error::InvalidColor(color as i16));
        }
        if !(0..=RGB_MAX as i32).contains(&r)
            || !(0..=RGB_MAX as i32).contains(&g)
            || !(0..=RGB_MAX as i32).contains(&b)
        {
            return Err(Error::InvalidArgument("RGB values must be 0-1000".into()));
        }

        self.colors[color as usize] = ColorDef::new(r as i16, g as i16, b as i16);
        Ok(())
    }

    /// Get the RGB definition of an extended color.
    #[cfg(feature = "ext-colors")]
    pub fn extended_color_content(&self, color: i32) -> Result<(i32, i32, i32)> {
        if !self.started {
            return Err(Error::ColorNotAvailable);
        }
        if color < 0 || color >= self.num_colors {
            return Err(Error::InvalidColor(color as i16));
        }

        let def = &self.colors[color as usize];
        Ok((def.red as i32, def.green as i32, def.blue as i32))
    }

    /// Allocate a color pair dynamically.
    ///
    /// Finds an unused color pair slot and initializes it with the given colors.
    /// Returns the allocated pair number, or an error if no slots are available.
    #[cfg(feature = "ext-colors")]
    pub fn alloc_pair(&mut self, fg: i32, bg: i32) -> Result<i32> {
        if !self.started {
            return Err(Error::ColorNotAvailable);
        }

        // Find an uninitialized pair (skip pair 0)
        for (i, pair) in self.pairs.iter().enumerate().skip(1) {
            if !pair.initialized {
                self.init_extended_pair(i as i32, fg, bg)?;
                return Ok(i as i32);
            }
        }

        Err(Error::InvalidArgument(
            "no free color pairs available".into(),
        ))
    }

    /// Find a color pair with the given foreground and background colors.
    ///
    /// Returns the pair number if found, or an error if not found.
    #[cfg(feature = "ext-colors")]
    pub fn find_pair(&self, fg: i32, bg: i32) -> Result<i32> {
        if !self.started {
            return Err(Error::ColorNotAvailable);
        }

        for (i, pair) in self.pairs.iter().enumerate() {
            if pair.initialized
                && pair.foreground == fg as ColorT
                && pair.background == bg as ColorT
            {
                return Ok(i as i32);
            }
        }

        Err(Error::InvalidArgument("color pair not found".into()))
    }

    /// Free a color pair, making it available for reuse.
    #[cfg(feature = "ext-colors")]
    pub fn free_pair(&mut self, pair: i32) -> Result<()> {
        if !self.started {
            return Err(Error::ColorNotAvailable);
        }
        if pair <= 0 || pair >= self.num_pairs {
            return Err(Error::InvalidColorPair(pair as i16));
        }

        self.pairs[pair as usize] = ColorPair::default_pair();
        Ok(())
    }
}

impl Default for ColorManager {
    fn default() -> Self {
        Self::new(DEFAULT_COLORS, DEFAULT_COLOR_PAIRS, false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_enum() {
        assert_eq!(Color::Black.to_index(), 0);
        assert_eq!(Color::White.to_index(), 7);
        assert_eq!(Color::from_index(3), Some(Color::Yellow));
        assert_eq!(Color::from_index(100), None);
    }

    #[test]
    fn test_color_manager() {
        let mut cm = ColorManager::new(8, 64, true);
        assert!(!cm.is_started());
        assert!(cm.has_colors());
        assert!(cm.can_change_color());

        cm.start().unwrap();
        assert!(cm.is_started());

        // Test init_pair
        cm.init_pair(1, COLOR_RED, COLOR_BLACK).unwrap();
        let (fg, bg) = cm.pair_content(1).unwrap();
        assert_eq!(fg, COLOR_RED);
        assert_eq!(bg, COLOR_BLACK);

        // Test init_color
        cm.init_color(1, 500, 500, 500).unwrap();
        let (r, g, b) = cm.color_content(1).unwrap();
        assert_eq!((r, g, b), (500, 500, 500));
    }
}
