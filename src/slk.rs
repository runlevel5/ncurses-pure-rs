//! Soft function key labels for ncurses-rs.
//!
//! This module provides soft label key (SLK) functionality, which displays
//! labels at the bottom of the screen for function keys. This feature must
//! be enabled with the `slk` feature flag.

use crate::error::{Error, Result};
use crate::types::AttrT;

/// Number of soft labels.
pub const SLK_LABELS: usize = 8;

/// Soft label format options.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlkFormat {
    /// 3-2-3 format (8 labels: 3 left, 2 center, 3 right).
    Format323,
    /// 4-4 format (8 labels: 4 left, 4 right).
    Format44,
    /// 4-4-4 format with index line (12 labels).
    Format444,
    /// 4-4-4 format with index line, PC style.
    Format444Pc,
}

impl SlkFormat {
    /// Convert from the integer format used by ncurses.
    pub fn from_int(fmt: i32) -> Option<Self> {
        match fmt {
            0 => Some(SlkFormat::Format323),
            1 => Some(SlkFormat::Format44),
            2 => Some(SlkFormat::Format444),
            3 => Some(SlkFormat::Format444Pc),
            _ => None,
        }
    }

    /// Get the number of labels for this format.
    pub fn num_labels(&self) -> usize {
        match self {
            SlkFormat::Format323 | SlkFormat::Format44 => 8,
            SlkFormat::Format444 | SlkFormat::Format444Pc => 12,
        }
    }

    /// Get the group sizes for this format.
    ///
    /// Returns a slice of group sizes (e.g., [3, 2, 3] for Format323).
    pub fn group_sizes(&self) -> &'static [usize] {
        match self {
            SlkFormat::Format323 => &[3, 2, 3],
            SlkFormat::Format44 => &[4, 4],
            SlkFormat::Format444 | SlkFormat::Format444Pc => &[4, 4, 4],
        }
    }
}

/// Soft label justification.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SlkJustify {
    /// Left-justify the label.
    #[default]
    Left,
    /// Center the label.
    Center,
    /// Right-justify the label.
    Right,
}

impl SlkJustify {
    /// Convert from integer justification value.
    pub fn from_int(j: i32) -> Option<Self> {
        match j {
            0 => Some(SlkJustify::Left),
            1 => Some(SlkJustify::Center),
            2 => Some(SlkJustify::Right),
            _ => None,
        }
    }
}

/// A single soft label.
#[derive(Clone, Debug, Default)]
pub struct SoftLabel {
    /// The label text.
    pub text: String,
    /// Label justification.
    pub justify: SlkJustify,
    /// Whether this label is visible.
    pub visible: bool,
    /// Whether this label has changed.
    pub dirty: bool,
}

impl SoftLabel {
    /// Create a new soft label.
    pub fn new() -> Self {
        Self {
            text: String::new(),
            justify: SlkJustify::Left,
            visible: true,
            dirty: true,
        }
    }

    /// Set the label text.
    pub fn set(&mut self, text: &str, justify: SlkJustify) {
        self.text = text.to_string();
        self.justify = justify;
        self.dirty = true;
    }

    /// Clear the label.
    pub fn clear(&mut self) {
        self.text.clear();
        self.dirty = true;
    }

    /// Format the label text with justification to the given width.
    pub fn format(&self, width: usize) -> String {
        let text_len = self.text.chars().count();
        if text_len >= width {
            self.text.chars().take(width).collect()
        } else {
            let padding = width - text_len;
            match self.justify {
                SlkJustify::Left => format!("{}{}", self.text, " ".repeat(padding)),
                SlkJustify::Right => format!("{}{}", " ".repeat(padding), self.text),
                SlkJustify::Center => {
                    let left_pad = padding / 2;
                    let right_pad = padding - left_pad;
                    format!(
                        "{}{}{}",
                        " ".repeat(left_pad),
                        self.text,
                        " ".repeat(right_pad)
                    )
                }
            }
        }
    }
}

/// Rendered label output for terminal display.
#[derive(Clone, Debug)]
pub struct RenderedLabel {
    /// The formatted text to display.
    pub text: String,
    /// Column position on screen.
    pub col: i32,
    /// Attributes to apply.
    pub attrs: AttrT,
}

/// Soft label key state and management.
pub struct SlkState {
    /// The format of the soft labels.
    format: SlkFormat,
    /// The soft labels.
    labels: Vec<SoftLabel>,
    /// Whether SLK has been initialized.
    initialized: bool,
    /// Whether labels are hidden.
    hidden: bool,
    /// Label attributes.
    attrs: AttrT,
    /// Label width (calculated based on screen width).
    label_width: i32,
    /// Screen columns (for positioning).
    screen_cols: i32,
    /// Screen rows (for positioning at bottom).
    screen_rows: i32,
    /// Whether we need to output to terminal (for noutrefresh vs refresh).
    needs_refresh: bool,
    /// Gap between label groups.
    group_gap: i32,
}

impl SlkState {
    /// Create a new SLK state with the specified format.
    pub fn new(format: SlkFormat) -> Self {
        let num = format.num_labels();
        Self {
            format,
            labels: (0..num).map(|_| SoftLabel::new()).collect(),
            initialized: false,
            hidden: false,
            attrs: 0,
            label_width: 8,
            screen_cols: 80,
            screen_rows: 24,
            needs_refresh: false,
            group_gap: 1,
        }
    }

    /// Initialize soft labels.
    pub fn init(&mut self, screen_cols: i32) -> Result<()> {
        self.init_with_size(screen_cols, 24)
    }

    /// Initialize soft labels with screen dimensions.
    pub fn init_with_size(&mut self, screen_cols: i32, screen_rows: i32) -> Result<()> {
        if self.initialized {
            return Ok(());
        }

        self.screen_cols = screen_cols;
        self.screen_rows = screen_rows;

        // Calculate label width based on screen columns
        // Account for gaps between groups
        let num_labels = self.format.num_labels() as i32;
        let num_groups = self.format.group_sizes().len() as i32;
        let total_gap_width = (num_groups - 1) * self.group_gap;
        let available_width = screen_cols - total_gap_width;
        self.label_width = available_width / num_labels;
        self.label_width = self.label_width.clamp(1, 16);

        self.initialized = true;
        self.needs_refresh = true;
        Ok(())
    }

    /// Update screen dimensions (call when terminal resizes).
    pub fn resize(&mut self, screen_cols: i32, screen_rows: i32) {
        self.screen_cols = screen_cols;
        self.screen_rows = screen_rows;

        // Recalculate label width
        let num_labels = self.format.num_labels() as i32;
        let num_groups = self.format.group_sizes().len() as i32;
        let total_gap_width = (num_groups - 1) * self.group_gap;
        let available_width = screen_cols - total_gap_width;
        self.label_width = available_width / num_labels;
        self.label_width = self.label_width.clamp(1, 16);

        // Mark all labels as dirty
        self.touch();
    }

    /// Set a soft label.
    pub fn set(&mut self, labnum: i32, label: &str, justify: i32) -> Result<()> {
        if !self.initialized {
            return Err(Error::NotInitialized);
        }

        let idx = labnum as usize;
        if idx >= self.labels.len() {
            return Err(Error::InvalidArgument("invalid label number".into()));
        }

        let j = SlkJustify::from_int(justify)
            .ok_or_else(|| Error::InvalidArgument("invalid justification".into()))?;

        // Truncate label to label_width
        let truncated: String = label.chars().take(self.label_width as usize).collect();
        self.labels[idx].set(&truncated, j);
        self.needs_refresh = true;

        Ok(())
    }

    /// Get a soft label.
    pub fn label(&self, labnum: i32) -> Option<&str> {
        self.labels.get(labnum as usize).map(|l| l.text.as_str())
    }

    /// Clear a soft label.
    pub fn clear(&mut self, labnum: i32) -> Result<()> {
        if let Some(label) = self.labels.get_mut(labnum as usize) {
            label.clear();
            self.needs_refresh = true;
            Ok(())
        } else {
            Err(Error::InvalidArgument("invalid label number".into()))
        }
    }

    /// Restore soft labels after shell escape.
    pub fn restore(&mut self) -> Result<()> {
        for label in &mut self.labels {
            label.dirty = true;
        }
        self.hidden = false;
        self.needs_refresh = true;
        Ok(())
    }

    /// Mark labels for refresh without outputting to screen yet.
    ///
    /// This is used in conjunction with `doupdate()` to batch updates.
    pub fn noutrefresh(&mut self) -> Result<()> {
        if self.hidden {
            return Ok(());
        }

        // Mark as needing update - actual rendering happens in refresh()
        // or when Screen calls doupdate()
        self.needs_refresh = true;
        Ok(())
    }

    /// Refresh the soft labels immediately.
    ///
    /// This marks all labels as clean. The actual terminal output should be
    /// performed by the Screen using `render()` to get the output data.
    pub fn refresh(&mut self) -> Result<()> {
        if self.hidden {
            return Ok(());
        }

        for label in &mut self.labels {
            label.dirty = false;
        }
        self.needs_refresh = false;

        Ok(())
    }

    /// Get the row where soft labels should be displayed (bottom of screen).
    pub fn display_row(&self) -> i32 {
        self.screen_rows - 1
    }

    /// Check if a refresh is needed.
    pub fn needs_refresh(&self) -> bool {
        self.needs_refresh
    }

    /// Render the soft labels to a list of positioned strings.
    ///
    /// Returns a vector of `RenderedLabel` structs that can be written to the terminal.
    /// The caller is responsible for positioning the cursor and writing the output.
    pub fn render(&self) -> Vec<RenderedLabel> {
        if self.hidden || !self.initialized {
            return Vec::new();
        }

        let mut result = Vec::new();
        let mut col: i32 = 0;
        let mut label_idx = 0;
        let width = self.label_width as usize;

        for (group_idx, &group_size) in self.format.group_sizes().iter().enumerate() {
            // Add gap before groups (except first)
            if group_idx > 0 {
                col += self.group_gap;
            }

            // Render labels in this group
            for _ in 0..group_size {
                if label_idx < self.labels.len() {
                    let label = &self.labels[label_idx];
                    if label.visible {
                        result.push(RenderedLabel {
                            text: label.format(width),
                            col,
                            attrs: self.attrs,
                        });
                    }
                    label_idx += 1;
                }
                col += self.label_width;
            }
        }

        result
    }

    /// Render to a single string (useful for simple output).
    ///
    /// Returns the full SLK line as a string, with spaces for gaps.
    pub fn render_line(&self) -> String {
        if self.hidden || !self.initialized {
            return String::new();
        }

        let mut result = String::new();
        let mut label_idx = 0;
        let width = self.label_width as usize;

        for (group_idx, &group_size) in self.format.group_sizes().iter().enumerate() {
            // Add gap before groups (except first)
            if group_idx > 0 {
                for _ in 0..self.group_gap {
                    result.push(' ');
                }
            }

            // Render labels in this group
            for _ in 0..group_size {
                if label_idx < self.labels.len() {
                    let label = &self.labels[label_idx];
                    if label.visible {
                        result.push_str(&label.format(width));
                    } else {
                        result.push_str(&" ".repeat(width));
                    }
                    label_idx += 1;
                }
            }
        }

        result
    }

    /// Generate ANSI escape sequence for rendering soft labels.
    ///
    /// This returns a string that can be written directly to the terminal
    /// to display the soft labels at the bottom of the screen.
    pub fn render_ansi(&self) -> String {
        if self.hidden || !self.initialized {
            return String::new();
        }

        let row = self.display_row();
        let line = self.render_line();

        // Move to bottom row, write content, reset attributes
        // CSI row;col H = move cursor
        // CSI 0m = reset attributes
        format!(
            "\x1b[{};1H\x1b[7m{}\x1b[0m",
            row + 1, // Terminal rows are 1-based
            line
        )
    }

    /// Set soft label attributes.
    pub fn attrset(&mut self, attrs: AttrT) -> Result<()> {
        self.attrs = attrs;
        for label in &mut self.labels {
            label.dirty = true;
        }
        self.needs_refresh = true;
        Ok(())
    }

    /// Turn on soft label attributes.
    pub fn attron(&mut self, attrs: AttrT) -> Result<()> {
        self.attrs |= attrs;
        for label in &mut self.labels {
            label.dirty = true;
        }
        self.needs_refresh = true;
        Ok(())
    }

    /// Turn off soft label attributes.
    pub fn attroff(&mut self, attrs: AttrT) -> Result<()> {
        self.attrs &= !attrs;
        for label in &mut self.labels {
            label.dirty = true;
        }
        self.needs_refresh = true;
        Ok(())
    }

    /// Get current soft label attributes.
    pub fn attr(&self) -> AttrT {
        self.attrs
    }

    /// Set soft label color pair.
    pub fn color(&mut self, pair: i16) -> Result<()> {
        self.attrs = (self.attrs & !crate::attr::A_COLOR) | crate::attr::color_pair(pair);
        for label in &mut self.labels {
            label.dirty = true;
        }
        self.needs_refresh = true;
        Ok(())
    }

    /// Hide the soft labels.
    pub fn clear_all(&mut self) -> Result<()> {
        self.hidden = true;
        self.needs_refresh = true;
        Ok(())
    }

    /// Touch all soft labels (mark as needing refresh).
    pub fn touch(&mut self) {
        for label in &mut self.labels {
            label.dirty = true;
        }
        self.needs_refresh = true;
    }

    /// Check if initialized.
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Check if hidden.
    pub fn is_hidden(&self) -> bool {
        self.hidden
    }

    /// Get the format.
    pub fn format(&self) -> SlkFormat {
        self.format
    }

    /// Get the label width.
    pub fn label_width(&self) -> i32 {
        self.label_width
    }

    /// Get the number of screen rows reserved for soft labels.
    ///
    /// This is typically 1 for formats 323/44, and 2 for formats 444/444pc
    /// (one for index, one for labels).
    pub fn reserved_rows(&self) -> i32 {
        match self.format {
            SlkFormat::Format323 | SlkFormat::Format44 => 1,
            SlkFormat::Format444 | SlkFormat::Format444Pc => 2,
        }
    }
}

impl Default for SlkState {
    fn default() -> Self {
        Self::new(SlkFormat::Format323)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slk_format() {
        assert_eq!(SlkFormat::from_int(0), Some(SlkFormat::Format323));
        assert_eq!(SlkFormat::Format323.num_labels(), 8);
        assert_eq!(SlkFormat::Format444.num_labels(), 12);
    }

    #[test]
    fn test_slk_format_groups() {
        assert_eq!(SlkFormat::Format323.group_sizes(), &[3, 2, 3]);
        assert_eq!(SlkFormat::Format44.group_sizes(), &[4, 4]);
        assert_eq!(SlkFormat::Format444.group_sizes(), &[4, 4, 4]);
    }

    #[test]
    fn test_soft_label() {
        let mut label = SoftLabel::new();
        label.set("Test", SlkJustify::Center);
        assert_eq!(label.text, "Test");
        assert_eq!(label.justify, SlkJustify::Center);
        assert!(label.dirty);
    }

    #[test]
    fn test_soft_label_format() {
        let mut label = SoftLabel::new();
        label.set("F1", SlkJustify::Left);
        assert_eq!(label.format(8), "F1      ");

        label.set("F1", SlkJustify::Right);
        assert_eq!(label.format(8), "      F1");

        label.set("F1", SlkJustify::Center);
        assert_eq!(label.format(8), "   F1   ");

        // Truncation
        label.set("VeryLongLabel", SlkJustify::Left);
        assert_eq!(label.format(8), "VeryLong");
    }

    #[test]
    fn test_slk_state() {
        let mut state = SlkState::new(SlkFormat::Format323);
        state.init(80).unwrap();

        state.set(0, "F1", 0).unwrap();
        assert_eq!(state.label(0), Some("F1"));

        state.clear(0).unwrap();
        assert_eq!(state.label(0), Some(""));
    }

    #[test]
    fn test_slk_render() {
        let mut state = SlkState::new(SlkFormat::Format323);
        state.init_with_size(80, 24).unwrap();

        state.set(0, "F1", 0).unwrap();
        state.set(1, "F2", 1).unwrap(); // Center
        state.set(2, "F3", 2).unwrap(); // Right

        let rendered = state.render();
        assert_eq!(rendered.len(), 8); // 8 labels in 3-2-3 format

        // Check first label
        assert_eq!(rendered[0].col, 0);
        assert!(rendered[0].text.starts_with("F1"));
    }

    #[test]
    fn test_slk_render_line() {
        let mut state = SlkState::new(SlkFormat::Format44);
        state.init_with_size(80, 24).unwrap();

        for i in 0..8 {
            state.set(i, &format!("F{}", i + 1), 0).unwrap();
        }

        let line = state.render_line();
        assert!(line.contains("F1"));
        assert!(line.contains("F8"));
    }

    #[test]
    fn test_slk_noutrefresh() {
        let mut state = SlkState::new(SlkFormat::Format323);
        state.init(80).unwrap();

        state.set(0, "Test", 0).unwrap();
        state.refresh().unwrap(); // Clear needs_refresh
        assert!(!state.needs_refresh());

        state.noutrefresh().unwrap();
        assert!(state.needs_refresh());
    }

    #[test]
    fn test_slk_display_row() {
        let mut state = SlkState::new(SlkFormat::Format323);
        state.init_with_size(80, 24).unwrap();
        assert_eq!(state.display_row(), 23); // Last row (0-indexed)

        state.resize(80, 40);
        assert_eq!(state.display_row(), 39);
    }
}
