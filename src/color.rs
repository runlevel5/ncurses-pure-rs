//! Color support for ncurses-pure.
//!
//! This module provides color handling functionality matching the X/Open
//! XSI Curses standard. Colors are organized into pairs (foreground/background)
//! that can be applied to text output.

use crate::error::{Error, Result};
use crate::types::ColorT;
use std::collections::BTreeMap;

/// How many colors have a slot of their own in a color manager's table.
/// Colors beyond these are kept only once they are defined.
const PALETTE_SIZE: i32 = 256;

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
    /// Foreground color index. Wider than `ColorT`, so that a pair can
    /// hold the color numbers of a terminal with more than 32767 colors.
    pub foreground: i32,
    /// Background color index.
    pub background: i32,
    /// Whether this pair has been initialized.
    pub initialized: bool,
}

impl ColorPair {
    /// Create a new color pair.
    pub const fn new(foreground: ColorT, background: ColorT) -> Self {
        Self::new_extended(foreground as i32, background as i32)
    }

    /// Create a new color pair from color numbers of any size.
    pub const fn new_extended(foreground: i32, background: i32) -> Self {
        Self {
            foreground,
            background,
            initialized: true,
        }
    }

    /// Create an uninitialized (default) color pair.
    pub const fn default_pair() -> Self {
        Self {
            foreground: COLOR_WHITE as i32,
            background: COLOR_BLACK as i32,
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
    /// Color definitions of the palette: the first `PALETTE_SIZE` colors,
    /// or all of them when the terminal has fewer.
    colors: Vec<ColorDef>,
    /// Definitions given to colors beyond the palette. A terminal with
    /// direct color has millions of them, almost all never defined.
    more_colors: BTreeMap<i32, ColorDef>,
    /// Color pairs.
    pairs: Vec<ColorPair>,
    /// Whether to use default colors (-1 for default).
    use_default_colors: bool,
}

impl ColorManager {
    /// Create a new color manager.
    pub fn new(num_colors: i32, num_pairs: i32, can_change: bool) -> Self {
        let palette = num_colors.clamp(0, PALETTE_SIZE) as usize;
        let mut colors = vec![ColorDef::default_color(); palette];
        let pairs = vec![ColorPair::default_pair(); num_pairs.max(0) as usize];

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
            more_colors: BTreeMap::new(),
            pairs,
            use_default_colors: false,
        }
    }

    /// The definition of `color`, which must be a valid color number.
    fn color_def(&self, color: i32) -> ColorDef {
        match self.colors.get(color as usize) {
            Some(def) => *def,
            None => self
                .more_colors
                .get(&color)
                .copied()
                .unwrap_or_else(ColorDef::default_color),
        }
    }

    /// Define `color`, which must be a valid color number.
    fn set_color_def(&mut self, color: i32, def: ColorDef) {
        match self.colors.get_mut(color as usize) {
            Some(slot) => *slot = def,
            None => {
                self.more_colors.insert(color, def);
            }
        }
    }

    /// The foreground and background of `pair` as the terminal is to show
    /// them, or `None` when there is no such pair or color is not started.
    pub(crate) fn pair_colors(&self, pair: i32) -> Option<(i32, i32)> {
        if !self.started {
            return None;
        }
        let pair = self.pairs.get(usize::try_from(pair).ok()?)?;
        Some((pair.foreground, pair.background))
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

        // A color number too large for a `ColorT` is reported as the largest
        // one; `extended_pair_content` returns it in full.
        let limit = |color: i32| color.min(i32::from(ColorT::MAX)) as ColorT;
        let cp = &self.pairs[pair as usize];
        Ok((limit(cp.foreground), limit(cp.background)))
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

        self.set_color_def(i32::from(color), ColorDef::new(r, g, b));
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

        let def = self.color_def(i32::from(color));
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

        self.pairs[pair as usize] = ColorPair::new_extended(fg, bg);
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
        Ok((cp.foreground, cp.background))
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

        self.set_color_def(color, ColorDef::new(r as i16, g as i16, b as i16));
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

        let def = self.color_def(color);
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
            if pair.initialized && pair.foreground == fg && pair.background == bg {
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

    /// A started color manager for a terminal with direct color.
    fn direct_color_manager() -> ColorManager {
        let mut colors = ColorManager::new(1 << 24, 32767, true);
        colors.start().unwrap();
        colors
    }

    #[test]
    fn test_color_table_stays_small_on_a_direct_color_terminal() {
        let colors = direct_color_manager();
        assert_eq!(colors.num_colors(), 1 << 24);
        assert_eq!(colors.colors.len(), PALETTE_SIZE as usize);
        assert!(colors.more_colors.is_empty());

        // A terminal with fewer colors than the palette holds gets a slot
        // for each, and one with none gets none.
        assert_eq!(ColorManager::new(8, 64, false).colors.len(), 8);
        assert_eq!(ColorManager::new(0, 0, false).colors.len(), 0);
        assert_eq!(ColorManager::new(-1, -1, false).colors.len(), 0);
    }

    #[test]
    fn test_colors_beyond_the_palette_can_be_defined_and_read() {
        let mut colors = direct_color_manager();
        // Not defined yet.
        assert_eq!(colors.color_content(300).unwrap(), (0, 0, 0));

        colors.init_color(300, 1000, 500, 0).unwrap();
        assert_eq!(colors.color_content(300).unwrap(), (1000, 500, 0));
        // The standard colors are untouched.
        assert_eq!(colors.color_content(1).unwrap(), (680, 0, 0));
        colors.init_color(1, 1, 2, 3).unwrap();
        assert_eq!(colors.color_content(1).unwrap(), (1, 2, 3));
        assert_eq!(colors.more_colors.len(), 1);
    }

    #[cfg(feature = "ext-colors")]
    #[test]
    fn test_extended_colors_beyond_the_palette() {
        let mut colors = direct_color_manager();
        colors.init_extended_color(1_000_000, 10, 20, 30).unwrap();
        assert_eq!(
            colors.extended_color_content(1_000_000).unwrap(),
            (10, 20, 30)
        );
        assert_eq!(colors.extended_color_content(1_000_001).unwrap(), (0, 0, 0));
        assert!(colors.init_extended_color(1 << 24, 0, 0, 0).is_err());
        assert!(colors.extended_color_content(1 << 24).is_err());
    }

    #[cfg(feature = "ext-colors")]
    #[test]
    fn test_extended_pair_keeps_large_color_numbers() {
        let mut colors = direct_color_manager();
        colors.init_extended_pair(1, 70_000, 0x12_34_56).unwrap();
        assert_eq!(
            colors.extended_pair_content(1).unwrap(),
            (70_000, 0x12_34_56)
        );
        assert_eq!(colors.pair_colors(1), Some((70_000, 0x12_34_56)));
        // The narrower call cannot say more than its type holds.
        assert_eq!(colors.pair_content(1).unwrap(), (i16::MAX, i16::MAX));

        assert_eq!(colors.find_pair(70_000, 0x12_34_56).unwrap(), 1);
        // 70000 cut down to 16 bits is 4464: not the same pair.
        assert!(colors.find_pair(4464, 0x12_34_56).is_err());
        assert_eq!(colors.alloc_pair(16_000_000, 1).unwrap(), 2);
        assert_eq!(colors.extended_pair_content(2).unwrap(), (16_000_000, 1));
    }

    #[cfg(feature = "ext-colors")]
    #[test]
    fn test_extended_pair_rejects_colors_the_terminal_does_not_have() {
        let mut colors = ColorManager::new(256, 32767, false);
        colors.start().unwrap();
        assert!(colors.init_extended_pair(1, 70_000, 0).is_err());
        assert!(colors.init_extended_pair(1, 0, 256).is_err());
        colors.init_extended_pair(1, 255, 0).unwrap();
        assert_eq!(colors.extended_pair_content(1).unwrap(), (255, 0));
        assert_eq!(colors.pair_content(1).unwrap(), (255, 0));
    }

    #[test]
    fn test_pair_colors_for_the_terminal() {
        let mut colors = ColorManager::new(8, 64, false);
        // Nothing before color is started.
        assert_eq!(colors.pair_colors(1), None);
        colors.start().unwrap();
        colors.init_pair(1, COLOR_RED, COLOR_BLUE).unwrap();
        assert_eq!(
            colors.pair_colors(1),
            Some((i32::from(COLOR_RED), i32::from(COLOR_BLUE)))
        );
        assert_eq!(colors.pair_colors(64), None);
        assert_eq!(colors.pair_colors(-1), None);
    }

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
