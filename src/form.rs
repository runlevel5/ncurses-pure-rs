//! Form library for ncurses-pure.
//!
//! This module provides form functionality for creating data entry forms
//! with fields. This feature must be enabled with the `form` feature flag.

use crate::error::{Error, Result};
use crate::types::{AttrT, ChType};
use crate::window::Window;

use std::cell::RefCell;
use std::rc::Rc;

// ============================================================================
// Field option flags
// ============================================================================

bitflags::bitflags! {
    /// Field option flags.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    pub struct FieldOpts: u32 {
        /// Field is visible.
        const O_VISIBLE = 0x0001;
        /// Field is active (can be visited).
        const O_ACTIVE = 0x0002;
        /// Field contents are displayed as entered.
        const O_PUBLIC = 0x0004;
        /// Field can be edited.
        const O_EDIT = 0x0008;
        /// Field wraps at end of line.
        const O_WRAP = 0x0010;
        /// Field is cleared on new input.
        const O_BLANK = 0x0020;
        /// Field contents validated on exit.
        const O_AUTOSKIP = 0x0040;
        /// Field has null padding.
        const O_NULLOK = 0x0080;
        /// Field contents are static.
        const O_STATIC = 0x0100;
        /// Field has a passthrough character.
        const O_PASSOK = 0x0200;
        /// Field reflows after editing.
        const O_REFLOW = 0x0400;
    }
}

bitflags::bitflags! {
    /// Form option flags.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    pub struct FormOpts: u32 {
        /// Form must be completed before exit.
        const O_NL_OVERLOAD = 0x0001;
        /// Backspace overload.
        const O_BS_OVERLOAD = 0x0002;
    }
}

// ============================================================================
// Form request codes
// ============================================================================

/// Form request: move to next page.
pub const REQ_NEXT_PAGE: i32 = 0x300;
/// Form request: move to previous page.
pub const REQ_PREV_PAGE: i32 = 0x301;
/// Form request: move to first page.
pub const REQ_FIRST_PAGE: i32 = 0x302;
/// Form request: move to last page.
pub const REQ_LAST_PAGE: i32 = 0x303;
/// Form request: move to next field.
pub const REQ_NEXT_FIELD: i32 = 0x304;
/// Form request: move to previous field.
pub const REQ_PREV_FIELD: i32 = 0x305;
/// Form request: move to first field.
pub const REQ_FIRST_FIELD: i32 = 0x306;
/// Form request: move to last field.
pub const REQ_LAST_FIELD: i32 = 0x307;
/// Form request: move to sorted next field.
pub const REQ_SNEXT_FIELD: i32 = 0x308;
/// Form request: move to sorted previous field.
pub const REQ_SPREV_FIELD: i32 = 0x309;
/// Form request: move to sorted first field.
pub const REQ_SFIRST_FIELD: i32 = 0x30a;
/// Form request: move to sorted last field.
pub const REQ_SLAST_FIELD: i32 = 0x30b;
/// Form request: move left in field.
pub const REQ_LEFT_FIELD: i32 = 0x30c;
/// Form request: move right in field.
pub const REQ_RIGHT_FIELD: i32 = 0x30d;
/// Form request: move up in field.
pub const REQ_UP_FIELD: i32 = 0x30e;
/// Form request: move down in field.
pub const REQ_DOWN_FIELD: i32 = 0x30f;
/// Form request: move to next character.
pub const REQ_NEXT_CHAR: i32 = 0x310;
/// Form request: move to previous character.
pub const REQ_PREV_CHAR: i32 = 0x311;
/// Form request: move to next line.
pub const REQ_NEXT_LINE: i32 = 0x312;
/// Form request: move to previous line.
pub const REQ_PREV_LINE: i32 = 0x313;
/// Form request: move to next word.
pub const REQ_NEXT_WORD: i32 = 0x314;
/// Form request: move to previous word.
pub const REQ_PREV_WORD: i32 = 0x315;
/// Form request: move to beginning of field.
pub const REQ_BEG_FIELD: i32 = 0x316;
/// Form request: move to end of field.
pub const REQ_END_FIELD: i32 = 0x317;
/// Form request: move to beginning of line.
pub const REQ_BEG_LINE: i32 = 0x318;
/// Form request: move to end of line.
pub const REQ_END_LINE: i32 = 0x319;
/// Form request: move left in field.
pub const REQ_LEFT_CHAR: i32 = 0x31a;
/// Form request: move right in field.
pub const REQ_RIGHT_CHAR: i32 = 0x31b;
/// Form request: move up in field.
pub const REQ_UP_CHAR: i32 = 0x31c;
/// Form request: move down in field.
pub const REQ_DOWN_CHAR: i32 = 0x31d;
/// Form request: insert a new line.
pub const REQ_NEW_LINE: i32 = 0x31e;
/// Form request: insert a character.
pub const REQ_INS_CHAR: i32 = 0x31f;
/// Form request: insert a line.
pub const REQ_INS_LINE: i32 = 0x320;
/// Form request: delete a character.
pub const REQ_DEL_CHAR: i32 = 0x321;
/// Form request: delete previous character.
pub const REQ_DEL_PREV: i32 = 0x322;
/// Form request: delete a line.
pub const REQ_DEL_LINE: i32 = 0x323;
/// Form request: delete a word.
pub const REQ_DEL_WORD: i32 = 0x324;
/// Form request: clear end of line.
pub const REQ_CLR_EOL: i32 = 0x325;
/// Form request: clear end of field.
pub const REQ_CLR_EOF: i32 = 0x326;
/// Form request: clear field.
pub const REQ_CLR_FIELD: i32 = 0x327;
/// Form request: overlay mode.
pub const REQ_OVL_MODE: i32 = 0x328;
/// Form request: insert mode.
pub const REQ_INS_MODE: i32 = 0x329;
/// Form request: scroll field forward.
pub const REQ_SCR_FLINE: i32 = 0x32a;
/// Form request: scroll field backward.
pub const REQ_SCR_BLINE: i32 = 0x32b;
/// Form request: scroll field forward a page.
pub const REQ_SCR_FPAGE: i32 = 0x32c;
/// Form request: scroll field backward a page.
pub const REQ_SCR_BPAGE: i32 = 0x32d;
/// Form request: scroll field forward half page.
pub const REQ_SCR_FHPAGE: i32 = 0x32e;
/// Form request: scroll field backward half page.
pub const REQ_SCR_BHPAGE: i32 = 0x32f;
/// Form request: scroll field forward a character.
pub const REQ_SCR_FCHAR: i32 = 0x330;
/// Form request: scroll field backward a character.
pub const REQ_SCR_BCHAR: i32 = 0x331;
/// Form request: horizontal scroll forward.
pub const REQ_SCR_HFLINE: i32 = 0x332;
/// Form request: horizontal scroll backward.
pub const REQ_SCR_HBLINE: i32 = 0x333;
/// Form request: horizontal scroll forward a half line.
pub const REQ_SCR_HFHALF: i32 = 0x334;
/// Form request: horizontal scroll backward a half line.
pub const REQ_SCR_HBHALF: i32 = 0x335;
/// Form request: validate field.
pub const REQ_VALIDATION: i32 = 0x336;

// ============================================================================
// Field Type
// ============================================================================

/// Field type for validation.
pub trait FieldType {
    /// Check if the field value is valid.
    fn validate(&self, field: &Field) -> bool;

    /// Get the type name.
    fn name(&self) -> &str;
}

/// Alphanumeric field type.
pub struct TypeAlnum {
    /// Minimum width.
    pub min_width: i32,
}

impl FieldType for TypeAlnum {
    fn validate(&self, field: &Field) -> bool {
        let buffer = field.buffer();
        let trimmed = buffer.trim();
        trimmed.len() >= self.min_width as usize && trimmed.chars().all(|c| c.is_alphanumeric())
    }

    fn name(&self) -> &str {
        "TYPE_ALNUM"
    }
}

/// Alpha field type.
pub struct TypeAlpha {
    /// Minimum width.
    pub min_width: i32,
}

impl FieldType for TypeAlpha {
    fn validate(&self, field: &Field) -> bool {
        let buffer = field.buffer();
        let trimmed = buffer.trim();
        trimmed.len() >= self.min_width as usize && trimmed.chars().all(|c| c.is_alphabetic())
    }

    fn name(&self) -> &str {
        "TYPE_ALPHA"
    }
}

/// Integer field type.
pub struct TypeInteger {
    /// Padding width.
    pub padding: i32,
    /// Minimum value.
    pub min: i32,
    /// Maximum value.
    pub max: i32,
}

impl FieldType for TypeInteger {
    fn validate(&self, field: &Field) -> bool {
        let buffer = field.buffer();
        if let Ok(n) = buffer.trim().parse::<i32>() {
            n >= self.min && n <= self.max
        } else {
            false
        }
    }

    fn name(&self) -> &str {
        "TYPE_INTEGER"
    }
}

/// Regular expression field type.
///
/// When the `regex` feature is enabled, this uses the `regex` crate for proper
/// regular expression matching. Otherwise, it falls back to simple substring matching.
pub struct TypeRegexp {
    /// The regular expression pattern.
    pub pattern: String,
    /// Compiled regex (when regex feature is enabled).
    #[cfg(feature = "regex")]
    compiled: Option<regex::Regex>,
}

impl TypeRegexp {
    /// Create a new TypeRegexp with the given pattern.
    pub fn new(pattern: &str) -> Self {
        #[cfg(feature = "regex")]
        {
            let compiled = regex::Regex::new(pattern).ok();
            Self {
                pattern: pattern.to_string(),
                compiled,
            }
        }
        #[cfg(not(feature = "regex"))]
        {
            Self {
                pattern: pattern.to_string(),
            }
        }
    }

    /// Check if the pattern is valid.
    #[cfg(feature = "regex")]
    pub fn is_valid(&self) -> bool {
        self.compiled.is_some()
    }

    /// Check if the pattern is valid (always true without regex feature).
    #[cfg(not(feature = "regex"))]
    pub fn is_valid(&self) -> bool {
        true
    }
}

impl FieldType for TypeRegexp {
    fn validate(&self, field: &Field) -> bool {
        let buffer = field.buffer();

        #[cfg(feature = "regex")]
        {
            if let Some(ref re) = self.compiled {
                re.is_match(&buffer)
            } else {
                // Invalid pattern - fail validation
                false
            }
        }

        #[cfg(not(feature = "regex"))]
        {
            // Fallback: simple substring match
            buffer.contains(&self.pattern)
        }
    }

    fn name(&self) -> &str {
        "TYPE_REGEXP"
    }
}

// ============================================================================
// Field
// ============================================================================

/// A form field.
pub struct Field {
    /// Field position (row).
    row: i32,
    /// Field position (column).
    col: i32,
    /// Field height.
    height: i32,
    /// Field width.
    width: i32,
    /// Number of off-screen rows (for scrollable fields).
    offscreen: i32,
    /// Additional buffers (reserved for future use).
    #[allow(dead_code)]
    nbuffers: i32,
    /// Field options.
    opts: FieldOpts,
    /// Field buffer (content) - stored as lines for multi-line fields.
    buffer: Vec<String>,
    /// Foreground attribute.
    fore: AttrT,
    /// Background attribute.
    back: AttrT,
    /// Pad character.
    pad: char,
    /// Field type (validation).
    field_type: Option<Box<dyn FieldType>>,
    /// User data.
    user_data: Option<Box<dyn std::any::Any>>,
    /// Cursor row within field (for multi-line fields).
    cursor_row: usize,
    /// Cursor column within field.
    cursor_col: usize,
    /// First visible row (for scrolling).
    scroll_row: usize,
    /// First visible column (for horizontal scrolling).
    scroll_col: usize,
}

impl Field {
    /// Create a new field.
    pub fn new(height: i32, width: i32, row: i32, col: i32, offscreen: i32, nbuffers: i32) -> Self {
        // Initialize buffer with appropriate number of lines
        let total_rows = (height + offscreen) as usize;
        let mut buffer = Vec::with_capacity(total_rows.max(1));
        buffer.push(String::new());

        Self {
            row,
            col,
            height,
            width,
            offscreen,
            nbuffers,
            opts: FieldOpts::O_VISIBLE
                | FieldOpts::O_ACTIVE
                | FieldOpts::O_PUBLIC
                | FieldOpts::O_EDIT
                | FieldOpts::O_WRAP
                | FieldOpts::O_BLANK
                | FieldOpts::O_AUTOSKIP
                | FieldOpts::O_NULLOK
                | FieldOpts::O_STATIC,
            buffer,
            fore: 0,
            back: 0,
            pad: ' ',
            field_type: None,
            user_data: None,
            cursor_row: 0,
            cursor_col: 0,
            scroll_row: 0,
            scroll_col: 0,
        }
    }

    /// Get the field buffer content as a single string.
    pub fn buffer(&self) -> String {
        self.buffer.join("\n")
    }

    /// Get the buffer for a specific row.
    pub fn buffer_row(&self, row: usize) -> Option<&str> {
        self.buffer.get(row).map(|s| s.as_str())
    }

    /// Get the number of rows in the buffer.
    pub fn buffer_rows(&self) -> usize {
        self.buffer.len()
    }

    /// Set the field buffer content.
    pub fn set_buffer(&mut self, value: &str) {
        self.buffer = value.lines().map(String::from).collect();
        if self.buffer.is_empty() {
            self.buffer.push(String::new());
        }
        self.cursor_row = 0;
        self.cursor_col = 0;
    }

    /// Get field dimensions.
    pub fn dimensions(&self) -> (i32, i32, i32, i32) {
        (self.height, self.width, self.row, self.col)
    }

    /// Get field row position.
    pub fn row(&self) -> i32 {
        self.row
    }

    /// Get field column position.
    pub fn col(&self) -> i32 {
        self.col
    }

    /// Get field height.
    pub fn height(&self) -> i32 {
        self.height
    }

    /// Get field width.
    pub fn width(&self) -> i32 {
        self.width
    }

    /// Get offscreen rows (for scrollable fields).
    pub fn offscreen(&self) -> i32 {
        self.offscreen
    }

    /// Get total rows (height + offscreen).
    pub fn total_rows(&self) -> i32 {
        self.height + self.offscreen
    }

    /// Move the field to a new position.
    ///
    /// This changes the field's row and column position. In ncurses,
    /// this function requires the field to be disconnected from any form.
    /// In this implementation, the caller is responsible for ensuring
    /// the field is not currently part of a posted form.
    ///
    /// # Arguments
    ///
    /// * `row` - New row position (must be >= 0)
    /// * `col` - New column position (must be >= 0)
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the position is invalid.
    pub fn move_to(&mut self, row: i32, col: i32) -> crate::Result<()> {
        if row < 0 || col < 0 {
            return Err(crate::error::Error::InvalidArgument(
                "field position must be non-negative".into(),
            ));
        }
        self.row = row;
        self.col = col;
        Ok(())
    }

    /// Set field options.
    pub fn set_opts(&mut self, opts: FieldOpts) {
        self.opts = opts;
    }

    /// Get field options.
    pub fn opts(&self) -> FieldOpts {
        self.opts
    }

    /// Set foreground attribute.
    pub fn set_fore(&mut self, attr: AttrT) {
        self.fore = attr;
    }

    /// Get foreground attribute.
    pub fn fore(&self) -> AttrT {
        self.fore
    }

    /// Set background attribute.
    pub fn set_back(&mut self, attr: AttrT) {
        self.back = attr;
    }

    /// Get background attribute.
    pub fn back(&self) -> AttrT {
        self.back
    }

    /// Set pad character.
    pub fn set_pad(&mut self, pad: char) {
        self.pad = pad;
    }

    /// Get pad character.
    pub fn pad(&self) -> char {
        self.pad
    }

    /// Set field type for validation.
    pub fn set_type<T: FieldType + 'static>(&mut self, field_type: T) {
        self.field_type = Some(Box::new(field_type));
    }

    /// Validate the field.
    pub fn validate(&self) -> bool {
        if let Some(ft) = &self.field_type {
            ft.validate(self)
        } else {
            true // No validation required
        }
    }

    /// Set user data.
    pub fn set_userptr<T: 'static>(&mut self, data: T) {
        self.user_data = Some(Box::new(data));
    }

    /// Get user data.
    pub fn userptr<T: 'static>(&self) -> Option<&T> {
        self.user_data.as_ref()?.downcast_ref::<T>()
    }

    /// Get cursor row position within field.
    pub fn cursor_row(&self) -> usize {
        self.cursor_row
    }

    /// Get cursor column position within field.
    pub fn cursor_col(&self) -> usize {
        self.cursor_col
    }

    /// Get cursor position as (row, col).
    pub fn cursor_pos(&self) -> (usize, usize) {
        (self.cursor_row, self.cursor_col)
    }

    /// Get the scroll row offset.
    pub fn scroll_row(&self) -> usize {
        self.scroll_row
    }

    /// Get the scroll column offset.
    pub fn scroll_col(&self) -> usize {
        self.scroll_col
    }

    /// Ensure cursor is visible by adjusting scroll position.
    fn adjust_scroll(&mut self) {
        let visible_rows = self.height as usize;
        let visible_cols = self.width as usize;

        // Vertical scrolling
        if self.cursor_row < self.scroll_row {
            self.scroll_row = self.cursor_row;
        } else if self.cursor_row >= self.scroll_row + visible_rows {
            self.scroll_row = self.cursor_row - visible_rows + 1;
        }

        // Horizontal scrolling
        if self.cursor_col < self.scroll_col {
            self.scroll_col = self.cursor_col;
        } else if self.cursor_col >= self.scroll_col + visible_cols {
            self.scroll_col = self.cursor_col - visible_cols + 1;
        }
    }

    /// Get the current line being edited.
    fn current_line(&self) -> &str {
        self.buffer
            .get(self.cursor_row)
            .map(|s| s.as_str())
            .unwrap_or("")
    }

    /// Insert a character at the cursor position.
    pub fn insert_char(&mut self, ch: char) {
        // Ensure we have enough rows
        while self.buffer.len() <= self.cursor_row {
            self.buffer.push(String::new());
        }

        // Check if we need to limit the input (O_STATIC means fixed size)
        if self.opts.contains(FieldOpts::O_STATIC) {
            // For single-line fields, check width
            if self.height == 1 && self.buffer[self.cursor_row].len() >= self.width as usize {
                return;
            }
            // For multi-line fields, check total capacity
            if self.height > 1 {
                let total_chars: usize = self.buffer.iter().map(|s| s.len()).sum();
                let max_chars = (self.height + self.offscreen) as usize * self.width as usize;
                if total_chars >= max_chars {
                    return;
                }
            }
        }

        let cursor_col = self.cursor_col;
        let line = &mut self.buffer[self.cursor_row];
        if cursor_col <= line.len() {
            line.insert(cursor_col, ch);
            self.cursor_col += 1;
        }

        // Handle wrap for multi-line fields
        if self.opts.contains(FieldOpts::O_WRAP) && self.height > 1 {
            self.wrap_line_if_needed();
        }

        self.adjust_scroll();
    }

    /// Wrap the current line if it exceeds the field width.
    fn wrap_line_if_needed(&mut self) {
        let width = self.width as usize;
        if self.buffer[self.cursor_row].len() > width {
            // Find the last space before the width limit, or just break at width
            let line = &self.buffer[self.cursor_row];
            let break_pos = line[..width]
                .rfind(' ')
                .map(|p| p + 1) // Break after the space
                .unwrap_or(width);

            let rest = self.buffer[self.cursor_row].split_off(break_pos);

            // Move cursor to next line if it was past the break
            if self.cursor_col >= break_pos {
                // Insert a new line or append to next
                if self.cursor_row + 1 < self.buffer.len() {
                    let next_line = &mut self.buffer[self.cursor_row + 1];
                    let old_len = rest.len();
                    *next_line = rest + next_line;
                    self.cursor_row += 1;
                    self.cursor_col -= break_pos;
                    // If we just added content to an existing line, it might need wrapping too
                    if next_line.len() > width {
                        self.wrap_line_if_needed();
                    }
                    // Adjust cursor position
                    let _ = old_len; // The position should be relative to what we moved
                } else {
                    self.buffer.push(rest);
                    self.cursor_row += 1;
                    self.cursor_col -= break_pos;
                }
            }
        }
    }

    /// Delete the character before the cursor.
    pub fn delete_prev(&mut self) {
        let cursor_col = self.cursor_col;
        let cursor_row = self.cursor_row;
        if cursor_col > 0 {
            let line = &mut self.buffer[cursor_row];
            if cursor_col <= line.len() {
                self.cursor_col -= 1;
                line.remove(self.cursor_col);
            }
        } else if cursor_row > 0 {
            // At beginning of line - join with previous line
            let current_line = self.buffer.remove(cursor_row);
            self.cursor_row -= 1;
            self.cursor_col = self.buffer[self.cursor_row].len();
            self.buffer[self.cursor_row].push_str(&current_line);
        }
        self.adjust_scroll();
    }

    /// Delete the character at the cursor.
    pub fn delete_char(&mut self) {
        let cursor_col = self.cursor_col;
        let cursor_row = self.cursor_row;
        let line = &mut self.buffer[cursor_row];
        if cursor_col < line.len() {
            line.remove(cursor_col);
        } else if cursor_row + 1 < self.buffer.len() {
            // At end of line - join with next line
            let next_line = self.buffer.remove(cursor_row + 1);
            self.buffer[cursor_row].push_str(&next_line);
        }
    }

    /// Move cursor left.
    pub fn cursor_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_row > 0 {
            // Wrap to end of previous line
            self.cursor_row -= 1;
            self.cursor_col = self.buffer[self.cursor_row].len();
        }
        self.adjust_scroll();
    }

    /// Move cursor right.
    pub fn cursor_right(&mut self) {
        let line_len = self.current_line().len();
        if self.cursor_col < line_len {
            self.cursor_col += 1;
        } else if self.cursor_row + 1 < self.buffer.len() {
            // Wrap to beginning of next line
            self.cursor_row += 1;
            self.cursor_col = 0;
        }
        self.adjust_scroll();
    }

    /// Move cursor up.
    pub fn cursor_up(&mut self) {
        if self.cursor_row > 0 {
            self.cursor_row -= 1;
            // Clamp column to line length
            let line_len = self.current_line().len();
            if self.cursor_col > line_len {
                self.cursor_col = line_len;
            }
        }
        self.adjust_scroll();
    }

    /// Move cursor down.
    pub fn cursor_down(&mut self) {
        if self.cursor_row + 1 < self.buffer.len() {
            self.cursor_row += 1;
            // Clamp column to line length
            let line_len = self.current_line().len();
            if self.cursor_col > line_len {
                self.cursor_col = line_len;
            }
        }
        self.adjust_scroll();
    }

    /// Move cursor to beginning of field.
    pub fn cursor_home(&mut self) {
        self.cursor_row = 0;
        self.cursor_col = 0;
        self.adjust_scroll();
    }

    /// Move cursor to end of field.
    pub fn cursor_end(&mut self) {
        if !self.buffer.is_empty() {
            self.cursor_row = self.buffer.len() - 1;
            self.cursor_col = self.buffer[self.cursor_row].len();
        }
        self.adjust_scroll();
    }

    /// Move cursor to beginning of current line.
    pub fn cursor_line_home(&mut self) {
        self.cursor_col = 0;
        self.adjust_scroll();
    }

    /// Move cursor to end of current line.
    pub fn cursor_line_end(&mut self) {
        self.cursor_col = self.current_line().len();
        self.adjust_scroll();
    }

    /// Move cursor to next line.
    pub fn cursor_next_line(&mut self) {
        if self.cursor_row + 1 < self.buffer.len() {
            self.cursor_row += 1;
            self.cursor_col = 0;
        }
        self.adjust_scroll();
    }

    /// Move cursor to previous line.
    pub fn cursor_prev_line(&mut self) {
        if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = 0;
        }
        self.adjust_scroll();
    }

    /// Move cursor to next word.
    pub fn cursor_next_word(&mut self) {
        let line = self.current_line();
        let chars: Vec<char> = line.chars().collect();
        let line_len = chars.len();

        // Skip current word (non-space characters)
        let mut pos = self.cursor_col;
        while pos < line_len && !chars[pos].is_whitespace() {
            pos += 1;
        }

        // Skip spaces
        while pos < line_len && chars[pos].is_whitespace() {
            pos += 1;
        }

        if pos < line_len {
            self.cursor_col = pos;
        } else if self.cursor_row + 1 < self.buffer.len() {
            // Move to next line
            self.cursor_row += 1;
            self.cursor_col = 0;
            // Skip leading spaces on new line
            let new_chars: Vec<char> = self.current_line().chars().collect();
            let mut new_pos = 0;
            while new_pos < new_chars.len() && new_chars[new_pos].is_whitespace() {
                new_pos += 1;
            }
            self.cursor_col = new_pos;
        } else {
            // Move to end of current line
            self.cursor_col = line_len;
        }
        self.adjust_scroll();
    }

    /// Move cursor to previous word.
    pub fn cursor_prev_word(&mut self) {
        // If at start of line, go to previous line
        if self.cursor_col == 0 {
            if self.cursor_row > 0 {
                self.cursor_row -= 1;
                self.cursor_col = self.current_line().len();
            }
            self.adjust_scroll();
            return;
        }

        let chars: Vec<char> = self.buffer[self.cursor_row].chars().collect();
        let mut pos = self.cursor_col;

        // Skip spaces backward
        while pos > 0 && chars[pos - 1].is_whitespace() {
            pos -= 1;
        }

        // Skip word backward
        while pos > 0 && !chars[pos - 1].is_whitespace() {
            pos -= 1;
        }

        self.cursor_col = pos;
        self.adjust_scroll();
    }

    /// Insert a new line at the cursor position.
    pub fn insert_line(&mut self) {
        // Split the current line at cursor position
        let rest = self.buffer[self.cursor_row].split_off(self.cursor_col);
        self.cursor_row += 1;
        self.buffer.insert(self.cursor_row, rest);
        self.cursor_col = 0;
        self.adjust_scroll();
    }

    /// Delete the current line.
    pub fn delete_line(&mut self) {
        if self.buffer.len() > 1 {
            self.buffer.remove(self.cursor_row);
            if self.cursor_row >= self.buffer.len() {
                self.cursor_row = self.buffer.len() - 1;
            }
            let line_len = self.current_line().len();
            if self.cursor_col > line_len {
                self.cursor_col = line_len;
            }
        } else {
            // Clear the only line
            self.buffer[0].clear();
            self.cursor_col = 0;
        }
        self.adjust_scroll();
    }

    /// Delete from cursor to end of line.
    pub fn clear_to_eol(&mut self) {
        let cursor_col = self.cursor_col;
        let cursor_row = self.cursor_row;
        let line = &mut self.buffer[cursor_row];
        line.truncate(cursor_col);
    }

    /// Delete from cursor to end of field.
    pub fn clear_to_eof(&mut self) {
        // Clear rest of current line
        self.clear_to_eol();
        // Remove all following lines
        self.buffer.truncate(self.cursor_row + 1);
    }

    /// Delete word at cursor.
    pub fn delete_word(&mut self) {
        let cursor_col = self.cursor_col;
        let cursor_row = self.cursor_row;
        let chars: Vec<char> = self.buffer[cursor_row].chars().collect();
        let mut end = cursor_col;

        // Skip current word
        while end < chars.len() && !chars[end].is_whitespace() {
            end += 1;
        }

        // Skip trailing spaces
        while end < chars.len() && chars[end].is_whitespace() {
            end += 1;
        }

        // Remove the characters
        if end > cursor_col {
            self.buffer[cursor_row].replace_range(cursor_col..end, "");
        }
    }

    /// Clear the field.
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.buffer.push(String::new());
        self.cursor_row = 0;
        self.cursor_col = 0;
        self.scroll_row = 0;
        self.scroll_col = 0;
    }

    /// Scroll field forward by n lines.
    pub fn scroll_forward(&mut self, n: usize) {
        let max_scroll = self.buffer.len().saturating_sub(self.height as usize);
        self.scroll_row = (self.scroll_row + n).min(max_scroll);
    }

    /// Scroll field backward by n lines.
    pub fn scroll_backward(&mut self, n: usize) {
        self.scroll_row = self.scroll_row.saturating_sub(n);
    }

    /// Scroll field forward by a page.
    pub fn scroll_forward_page(&mut self) {
        self.scroll_forward(self.height as usize);
    }

    /// Scroll field backward by a page.
    pub fn scroll_backward_page(&mut self) {
        self.scroll_backward(self.height as usize);
    }

    /// Scroll field forward by half a page.
    pub fn scroll_forward_half(&mut self) {
        self.scroll_forward((self.height as usize) / 2);
    }

    /// Scroll field backward by half a page.
    pub fn scroll_backward_half(&mut self) {
        self.scroll_backward((self.height as usize) / 2);
    }

    /// Scroll field horizontally forward by n columns.
    pub fn scroll_hforward(&mut self, n: usize) {
        let max_col = self.buffer.iter().map(|s| s.len()).max().unwrap_or(0);
        let max_scroll = max_col.saturating_sub(self.width as usize);
        self.scroll_col = (self.scroll_col + n).min(max_scroll);
    }

    /// Scroll field horizontally backward by n columns.
    pub fn scroll_hbackward(&mut self, n: usize) {
        self.scroll_col = self.scroll_col.saturating_sub(n);
    }

    /// Render the field to a window at the field's position.
    pub fn render(&self, win: &mut Window, is_current: bool) -> Result<()> {
        let attr = if is_current { self.fore } else { self.back };
        let is_public = self.opts.contains(FieldOpts::O_PUBLIC);
        let is_visible = self.opts.contains(FieldOpts::O_VISIBLE);

        for row in 0..self.height as usize {
            let buffer_row = self.scroll_row + row;
            let y = self.row + row as i32;

            // Move to start of field row
            win.mv(y, self.col)?;

            // Get the content for this row
            let content = self
                .buffer
                .get(buffer_row)
                .map(|s| s.as_str())
                .unwrap_or("");

            // Pre-collect characters for efficient indexed access
            let chars: Vec<char> = content.chars().collect();
            let visible_start = self.scroll_col;

            // Write characters with attribute
            for col in 0..self.width as usize {
                let content_col = visible_start + col;
                let ch = if content_col < chars.len() {
                    chars[content_col]
                } else {
                    self.pad
                };

                // Don't display if O_PUBLIC is not set (password field)
                let display_ch = if is_public {
                    ch
                } else if ch != self.pad {
                    '*' // Mask non-pad characters
                } else {
                    self.pad
                };

                if is_visible {
                    win.addch(display_ch as ChType | attr)?;
                }
            }
        }

        // Position cursor if this is the current field
        if is_current {
            let cursor_y = self.row + (self.cursor_row - self.scroll_row) as i32;
            let cursor_x = self.col + (self.cursor_col - self.scroll_col) as i32;
            if cursor_y >= self.row
                && cursor_y < self.row + self.height
                && cursor_x >= self.col
                && cursor_x < self.col + self.width
            {
                win.mv(cursor_y, cursor_x)?;
            }
        }

        Ok(())
    }
}

impl std::fmt::Debug for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Field")
            .field("row", &self.row)
            .field("col", &self.col)
            .field("height", &self.height)
            .field("width", &self.width)
            .field("buffer", &self.buffer)
            .finish()
    }
}

// ============================================================================
// Form
// ============================================================================

/// Type alias for form hook callbacks.
///
/// These are called at specific points in form/field lifecycle:
/// - Form init: when form is posted and after page changes
/// - Form term: when form is unposted and before page changes
/// - Field init: when form is posted and after current field changes
/// - Field term: when form is unposted and before current field changes
pub type FormHook = Box<dyn Fn(&Form)>;

/// A form containing fields.
pub struct Form {
    /// The form fields.
    fields: Vec<Rc<RefCell<Field>>>,
    /// Current field index.
    current: usize,
    /// Form options.
    opts: FormOpts,
    /// Current page.
    page: i32,
    /// Number of pages.
    max_page: i32,
    /// The form window.
    window: Option<Rc<RefCell<Window>>>,
    /// The form sub-window.
    sub_window: Option<Rc<RefCell<Window>>>,
    /// User data.
    user_data: Option<Box<dyn std::any::Any>>,
    /// Whether the form is posted.
    posted: bool,
    /// Insert mode.
    insert_mode: bool,
    /// Form initialization hook (called when posted and after page change).
    form_init: Option<FormHook>,
    /// Form termination hook (called when unposted and before page change).
    form_term: Option<FormHook>,
    /// Field initialization hook (called when posted and after field change).
    field_init: Option<FormHook>,
    /// Field termination hook (called when unposted and before field change).
    field_term: Option<FormHook>,
}

impl Form {
    /// Create a new form.
    pub fn new(fields: Vec<Field>) -> Self {
        let fields: Vec<_> = fields
            .into_iter()
            .map(|f| Rc::new(RefCell::new(f)))
            .collect();

        Self {
            fields,
            current: 0,
            opts: FormOpts::empty(),
            page: 0,
            max_page: 1,
            window: None,
            sub_window: None,
            user_data: None,
            posted: false,
            insert_mode: true,
            form_init: None,
            form_term: None,
            field_init: None,
            field_term: None,
        }
    }

    /// Get the number of fields.
    pub fn field_count(&self) -> usize {
        self.fields.len()
    }

    /// Get all form fields.
    ///
    /// Returns a slice of all fields in the form.
    pub fn fields(&self) -> &[Rc<RefCell<Field>>] {
        &self.fields
    }

    /// Set the form fields.
    ///
    /// This replaces all fields in the form. The form must not be posted.
    /// After setting fields, the current selection is reset to the first field.
    ///
    /// # Arguments
    ///
    /// * `fields` - The new fields for the form
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the form is currently posted.
    pub fn set_fields(&mut self, fields: Vec<Field>) -> Result<()> {
        if self.posted {
            return Err(Error::InvalidArgument(
                "cannot change fields while form is posted".into(),
            ));
        }

        let fields: Vec<_> = fields
            .into_iter()
            .map(|f| Rc::new(RefCell::new(f)))
            .collect();

        self.fields = fields;
        self.current = 0;
        self.page = 0;

        Ok(())
    }

    /// Get a field by index.
    pub fn field(&self, index: usize) -> Option<Rc<RefCell<Field>>> {
        self.fields.get(index).cloned()
    }

    /// Get the current field.
    pub fn current_field(&self) -> Option<Rc<RefCell<Field>>> {
        self.fields.get(self.current).cloned()
    }

    /// Set the current field by index.
    ///
    /// This calls the field termination hook before changing fields,
    /// and the field initialization hook after changing fields.
    pub fn set_current_field(&mut self, index: usize) -> Result<()> {
        if index < self.fields.len() {
            if index != self.current && self.posted {
                // Call field term hook before changing
                self.call_field_term();
            }
            self.current = index;
            if self.posted {
                // Call field init hook after changing
                self.call_field_init();
            }
            Ok(())
        } else {
            Err(Error::InvalidArgument("invalid field index".into()))
        }
    }

    /// Get the current field index.
    pub fn current_field_index(&self) -> usize {
        self.current
    }

    /// Set form options.
    pub fn set_opts(&mut self, opts: FormOpts) {
        self.opts = opts;
    }

    /// Get form options.
    pub fn opts(&self) -> FormOpts {
        self.opts
    }

    /// Set the form window.
    pub fn set_window(&mut self, window: Window) {
        self.window = Some(Rc::new(RefCell::new(window)));
    }

    /// Set the form sub-window.
    pub fn set_sub(&mut self, window: Window) {
        self.sub_window = Some(Rc::new(RefCell::new(window)));
    }

    /// Get the current page.
    pub fn page(&self) -> i32 {
        self.page
    }

    /// Set the current page.
    pub fn set_page(&mut self, page: i32) -> Result<()> {
        if page >= 0 && page < self.max_page {
            self.page = page;
            Ok(())
        } else {
            Err(Error::InvalidArgument("invalid page number".into()))
        }
    }

    /// Post the form (display it).
    pub fn post(&mut self) -> Result<()> {
        if self.posted {
            return Err(Error::InvalidArgument("form already posted".into()));
        }
        self.posted = true;
        // Call form init hook
        self.call_form_init();
        // Call field init hook for the current field
        self.call_field_init();
        Ok(())
    }

    /// Unpost the form (hide it).
    pub fn unpost(&mut self) -> Result<()> {
        if !self.posted {
            return Err(Error::InvalidArgument("form not posted".into()));
        }
        // Call field term hook for the current field
        self.call_field_term();
        // Call form term hook
        self.call_form_term();
        self.posted = false;
        Ok(())
    }

    /// Check if the form is posted.
    pub fn is_posted(&self) -> bool {
        self.posted
    }

    /// Process a form request.
    pub fn driver(&mut self, req: i32) -> Result<()> {
        match req {
            // Page navigation
            REQ_NEXT_PAGE => {
                if self.page + 1 < self.max_page {
                    self.page += 1;
                }
            }
            REQ_PREV_PAGE => {
                if self.page > 0 {
                    self.page -= 1;
                }
            }
            REQ_FIRST_PAGE => {
                self.page = 0;
            }
            REQ_LAST_PAGE => {
                self.page = self.max_page - 1;
            }

            // Field navigation
            REQ_NEXT_FIELD => {
                if self.current + 1 < self.fields.len() {
                    self.current += 1;
                } else {
                    self.current = 0;
                }
            }
            REQ_PREV_FIELD => {
                if self.current > 0 {
                    self.current -= 1;
                } else {
                    self.current = self.fields.len().saturating_sub(1);
                }
            }
            REQ_FIRST_FIELD => {
                self.current = 0;
            }
            REQ_LAST_FIELD => {
                self.current = self.fields.len().saturating_sub(1);
            }
            REQ_SNEXT_FIELD => {
                // Sorted next - for now same as next
                if self.current + 1 < self.fields.len() {
                    self.current += 1;
                } else {
                    self.current = 0;
                }
            }
            REQ_SPREV_FIELD => {
                // Sorted prev - for now same as prev
                if self.current > 0 {
                    self.current -= 1;
                } else {
                    self.current = self.fields.len().saturating_sub(1);
                }
            }
            REQ_SFIRST_FIELD => {
                self.current = 0;
            }
            REQ_SLAST_FIELD => {
                self.current = self.fields.len().saturating_sub(1);
            }
            REQ_LEFT_FIELD | REQ_UP_FIELD => {
                // Spatial navigation - move to field above/left
                self.navigate_to_adjacent_field(-1, 0);
            }
            REQ_RIGHT_FIELD | REQ_DOWN_FIELD => {
                // Spatial navigation - move to field below/right
                self.navigate_to_adjacent_field(1, 0);
            }

            // Intra-field cursor movement (horizontal)
            REQ_LEFT_CHAR | REQ_PREV_CHAR => {
                if let Some(field) = self.fields.get(self.current) {
                    field.borrow_mut().cursor_left();
                }
            }
            REQ_RIGHT_CHAR | REQ_NEXT_CHAR => {
                if let Some(field) = self.fields.get(self.current) {
                    field.borrow_mut().cursor_right();
                }
            }

            // Intra-field cursor movement (vertical)
            REQ_UP_CHAR => {
                if let Some(field) = self.fields.get(self.current) {
                    field.borrow_mut().cursor_up();
                }
            }
            REQ_DOWN_CHAR => {
                if let Some(field) = self.fields.get(self.current) {
                    field.borrow_mut().cursor_down();
                }
            }
            REQ_NEXT_LINE => {
                if let Some(field) = self.fields.get(self.current) {
                    field.borrow_mut().cursor_next_line();
                }
            }
            REQ_PREV_LINE => {
                if let Some(field) = self.fields.get(self.current) {
                    field.borrow_mut().cursor_prev_line();
                }
            }

            // Word navigation
            REQ_NEXT_WORD => {
                if let Some(field) = self.fields.get(self.current) {
                    field.borrow_mut().cursor_next_word();
                }
            }
            REQ_PREV_WORD => {
                if let Some(field) = self.fields.get(self.current) {
                    field.borrow_mut().cursor_prev_word();
                }
            }

            // Field boundaries
            REQ_BEG_FIELD => {
                if let Some(field) = self.fields.get(self.current) {
                    field.borrow_mut().cursor_home();
                }
            }
            REQ_END_FIELD => {
                if let Some(field) = self.fields.get(self.current) {
                    field.borrow_mut().cursor_end();
                }
            }
            REQ_BEG_LINE => {
                if let Some(field) = self.fields.get(self.current) {
                    field.borrow_mut().cursor_line_home();
                }
            }
            REQ_END_LINE => {
                if let Some(field) = self.fields.get(self.current) {
                    field.borrow_mut().cursor_line_end();
                }
            }

            // Editing: insert/delete character
            REQ_INS_CHAR => {
                if let Some(field) = self.fields.get(self.current) {
                    field.borrow_mut().insert_char(' ');
                }
            }
            REQ_DEL_CHAR => {
                if let Some(field) = self.fields.get(self.current) {
                    field.borrow_mut().delete_char();
                }
            }
            REQ_DEL_PREV => {
                if let Some(field) = self.fields.get(self.current) {
                    field.borrow_mut().delete_prev();
                }
            }

            // Editing: line operations
            REQ_NEW_LINE => {
                if let Some(field) = self.fields.get(self.current) {
                    let mut f = field.borrow_mut();
                    if f.height() > 1 {
                        f.insert_line();
                    }
                }
            }
            REQ_INS_LINE => {
                if let Some(field) = self.fields.get(self.current) {
                    let mut f = field.borrow_mut();
                    if f.height() > 1 {
                        f.insert_line();
                    }
                }
            }
            REQ_DEL_LINE => {
                if let Some(field) = self.fields.get(self.current) {
                    field.borrow_mut().delete_line();
                }
            }
            REQ_DEL_WORD => {
                if let Some(field) = self.fields.get(self.current) {
                    field.borrow_mut().delete_word();
                }
            }

            // Clearing
            REQ_CLR_EOL => {
                if let Some(field) = self.fields.get(self.current) {
                    field.borrow_mut().clear_to_eol();
                }
            }
            REQ_CLR_EOF => {
                if let Some(field) = self.fields.get(self.current) {
                    field.borrow_mut().clear_to_eof();
                }
            }
            REQ_CLR_FIELD => {
                if let Some(field) = self.fields.get(self.current) {
                    field.borrow_mut().clear();
                }
            }

            // Mode changes
            REQ_INS_MODE => {
                self.insert_mode = true;
            }
            REQ_OVL_MODE => {
                self.insert_mode = false;
            }

            // Scrolling (vertical)
            REQ_SCR_FLINE => {
                if let Some(field) = self.fields.get(self.current) {
                    field.borrow_mut().scroll_forward(1);
                }
            }
            REQ_SCR_BLINE => {
                if let Some(field) = self.fields.get(self.current) {
                    field.borrow_mut().scroll_backward(1);
                }
            }
            REQ_SCR_FPAGE => {
                if let Some(field) = self.fields.get(self.current) {
                    field.borrow_mut().scroll_forward_page();
                }
            }
            REQ_SCR_BPAGE => {
                if let Some(field) = self.fields.get(self.current) {
                    field.borrow_mut().scroll_backward_page();
                }
            }
            REQ_SCR_FHPAGE => {
                if let Some(field) = self.fields.get(self.current) {
                    field.borrow_mut().scroll_forward_half();
                }
            }
            REQ_SCR_BHPAGE => {
                if let Some(field) = self.fields.get(self.current) {
                    field.borrow_mut().scroll_backward_half();
                }
            }

            // Horizontal scrolling
            REQ_SCR_FCHAR => {
                if let Some(field) = self.fields.get(self.current) {
                    field.borrow_mut().scroll_hforward(1);
                }
            }
            REQ_SCR_BCHAR => {
                if let Some(field) = self.fields.get(self.current) {
                    field.borrow_mut().scroll_hbackward(1);
                }
            }
            REQ_SCR_HFLINE => {
                if let Some(field) = self.fields.get(self.current) {
                    let width = field.borrow().width() as usize;
                    field.borrow_mut().scroll_hforward(width);
                }
            }
            REQ_SCR_HBLINE => {
                if let Some(field) = self.fields.get(self.current) {
                    let width = field.borrow().width() as usize;
                    field.borrow_mut().scroll_hbackward(width);
                }
            }
            REQ_SCR_HFHALF => {
                if let Some(field) = self.fields.get(self.current) {
                    let width = field.borrow().width() as usize;
                    field.borrow_mut().scroll_hforward(width / 2);
                }
            }
            REQ_SCR_HBHALF => {
                if let Some(field) = self.fields.get(self.current) {
                    let width = field.borrow().width() as usize;
                    field.borrow_mut().scroll_hbackward(width / 2);
                }
            }

            // Validation
            REQ_VALIDATION => {
                if let Some(field) = self.fields.get(self.current) {
                    if !field.borrow().validate() {
                        return Err(Error::InvalidArgument("field validation failed".into()));
                    }
                }
            }

            // Handle printable characters
            _ => {
                if (0x20..0x7f).contains(&req) {
                    if let Some(field) = self.fields.get(self.current) {
                        field.borrow_mut().insert_char(req as u8 as char);
                    }
                }
            }
        }
        Ok(())
    }

    /// Navigate to an adjacent field based on position.
    fn navigate_to_adjacent_field(&mut self, dy: i32, _dx: i32) {
        if self.fields.is_empty() {
            return;
        }

        let current_field = match self.fields.get(self.current) {
            Some(f) => f.borrow(),
            None => return,
        };
        let (_, _, curr_row, _) = current_field.dimensions();
        drop(current_field);

        // Find the field in the specified direction
        let mut best_idx = None;
        let mut best_distance = i32::MAX;

        for (idx, field) in self.fields.iter().enumerate() {
            if idx == self.current {
                continue;
            }
            let f = field.borrow();
            let (_, _, row, _) = f.dimensions();

            if (dy > 0 && row > curr_row) || (dy < 0 && row < curr_row) {
                let distance = (row - curr_row).abs();
                if distance < best_distance {
                    best_distance = distance;
                    best_idx = Some(idx);
                }
            }
        }

        if let Some(idx) = best_idx {
            self.current = idx;
        }
    }

    /// Set user data.
    pub fn set_userptr<T: 'static>(&mut self, data: T) {
        self.user_data = Some(Box::new(data));
    }

    /// Get user data.
    pub fn userptr<T: 'static>(&self) -> Option<&T> {
        self.user_data.as_ref()?.downcast_ref::<T>()
    }

    /// Validate all fields in the form.
    pub fn validate_all(&self) -> bool {
        self.fields.iter().all(|f| f.borrow().validate())
    }

    /// Get data from all fields.
    pub fn data(&self) -> Vec<String> {
        self.fields.iter().map(|f| f.borrow().buffer()).collect()
    }

    /// Render the form to a window.
    ///
    /// This draws all visible fields to the form's window or sub-window.
    pub fn render(&self) -> Result<()> {
        // Get the window to render to (prefer sub_window, fall back to window)
        let win_rc = self
            .sub_window
            .as_ref()
            .or(self.window.as_ref())
            .ok_or_else(|| Error::InvalidArgument("form has no window".into()))?;

        let mut win = win_rc.borrow_mut();

        for (idx, field) in self.fields.iter().enumerate() {
            let f = field.borrow();
            if f.opts().contains(FieldOpts::O_VISIBLE) {
                f.render(&mut win, idx == self.current)?;
            }
        }

        Ok(())
    }

    /// Render the form to a specific window.
    ///
    /// This is useful when you want to render the form to a different window
    /// than the one set via `set_window`.
    pub fn render_to(&self, win: &mut Window) -> Result<()> {
        for (idx, field) in self.fields.iter().enumerate() {
            let f = field.borrow();
            if f.opts().contains(FieldOpts::O_VISIBLE) {
                f.render(win, idx == self.current)?;
            }
        }
        Ok(())
    }

    /// Get the cursor position for the current field (in screen coordinates).
    pub fn cursor_position(&self) -> Option<(i32, i32)> {
        let field = self.fields.get(self.current)?;
        let f = field.borrow();
        let (cursor_row, cursor_col) = f.cursor_pos();
        let scroll_row = f.scroll_row();
        let scroll_col = f.scroll_col();

        let screen_row = f.row() + (cursor_row - scroll_row) as i32;
        let screen_col = f.col() + (cursor_col - scroll_col) as i32;

        Some((screen_row, screen_col))
    }

    /// Check if insert mode is active.
    pub fn is_insert_mode(&self) -> bool {
        self.insert_mode
    }

    // ========================================================================
    // Hook functions
    // ========================================================================

    /// Set the form initialization hook.
    ///
    /// This function is called when the form is posted and just after
    /// a page change occurs.
    pub fn set_form_init<F>(&mut self, hook: F)
    where
        F: Fn(&Form) + 'static,
    {
        self.form_init = Some(Box::new(hook));
    }

    /// Get a reference to the form initialization hook.
    ///
    /// Returns `true` if a form init hook is set.
    pub fn has_form_init(&self) -> bool {
        self.form_init.is_some()
    }

    /// Clear the form initialization hook.
    pub fn clear_form_init(&mut self) {
        self.form_init = None;
    }

    /// Set the form termination hook.
    ///
    /// This function is called when the form is unposted and just before
    /// a page change occurs.
    pub fn set_form_term<F>(&mut self, hook: F)
    where
        F: Fn(&Form) + 'static,
    {
        self.form_term = Some(Box::new(hook));
    }

    /// Get a reference to the form termination hook.
    ///
    /// Returns `true` if a form term hook is set.
    pub fn has_form_term(&self) -> bool {
        self.form_term.is_some()
    }

    /// Clear the form termination hook.
    pub fn clear_form_term(&mut self) {
        self.form_term = None;
    }

    /// Set the field initialization hook.
    ///
    /// This function is called when the form is posted and just after
    /// the current field changes.
    pub fn set_field_init<F>(&mut self, hook: F)
    where
        F: Fn(&Form) + 'static,
    {
        self.field_init = Some(Box::new(hook));
    }

    /// Get a reference to the field initialization hook.
    ///
    /// Returns `true` if a field init hook is set.
    pub fn has_field_init(&self) -> bool {
        self.field_init.is_some()
    }

    /// Clear the field initialization hook.
    pub fn clear_field_init(&mut self) {
        self.field_init = None;
    }

    /// Set the field termination hook.
    ///
    /// This function is called when the form is unposted and just before
    /// the current field changes.
    pub fn set_field_term<F>(&mut self, hook: F)
    where
        F: Fn(&Form) + 'static,
    {
        self.field_term = Some(Box::new(hook));
    }

    /// Get a reference to the field termination hook.
    ///
    /// Returns `true` if a field term hook is set.
    pub fn has_field_term(&self) -> bool {
        self.field_term.is_some()
    }

    /// Clear the field termination hook.
    pub fn clear_field_term(&mut self) {
        self.field_term = None;
    }

    /// Call the form init hook if set.
    fn call_form_init(&self) {
        if let Some(ref hook) = self.form_init {
            hook(self);
        }
    }

    /// Call the form term hook if set.
    fn call_form_term(&self) {
        if let Some(ref hook) = self.form_term {
            hook(self);
        }
    }

    /// Call the field init hook if set.
    fn call_field_init(&self) {
        if let Some(ref hook) = self.field_init {
            hook(self);
        }
    }

    /// Call the field term hook if set.
    fn call_field_term(&self) {
        if let Some(ref hook) = self.field_term {
            hook(self);
        }
    }
}

impl std::fmt::Debug for Form {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Form")
            .field("field_count", &self.fields.len())
            .field("current", &self.current)
            .field("page", &self.page)
            .field("posted", &self.posted)
            .finish()
    }
}

// ============================================================================
// Free functions for ncurses compatibility (Form)
// ============================================================================

/// Create a new form from an array of fields.
///
/// This is the ncurses `new_form()` function.
pub fn new_form(fields: Vec<Field>) -> Form {
    Form::new(fields)
}

/// Free a form.
///
/// This is the ncurses `free_form()` function.
/// In Rust, the form is automatically freed when dropped.
pub fn free_form(_form: Form) {
    // Form is dropped automatically
}

/// Post a form to its associated window.
///
/// This is the ncurses `post_form()` function.
pub fn post_form(form: &mut Form) -> Result<()> {
    form.post()
}

/// Unpost a form from its associated window.
///
/// This is the ncurses `unpost_form()` function.
pub fn unpost_form(form: &mut Form) -> Result<()> {
    form.unpost()
}

/// Process form input.
///
/// This is the ncurses `form_driver()` function.
pub fn form_driver(form: &mut Form, request: i32) -> Result<()> {
    form.driver(request)
}

/// Process wide character form input.
///
/// This is the ncurses `form_driver_w()` function.
pub fn form_driver_w(form: &mut Form, request: i32) -> Result<()> {
    form.driver(request)
}

/// Set the form window.
///
/// This is the ncurses `set_form_win()` function.
pub fn set_form_win(form: &mut Form, window: Window) {
    form.set_window(window);
}

/// Get the form window.
///
/// This is the ncurses `form_win()` function.
/// Note: Form doesn't expose a window getter, so this returns None.
pub fn form_win(_form: &Form) -> Option<std::cell::Ref<'_, Window>> {
    // Form struct doesn't expose window getter
    None
}

/// Set the form sub-window.
///
/// This is the ncurses `set_form_sub()` function.
pub fn set_form_sub(form: &mut Form, window: Window) {
    form.set_sub(window);
}

/// Get the form sub-window.
///
/// This is the ncurses `form_sub()` function.
/// Note: Form doesn't expose a sub getter, so this returns None.
pub fn form_sub(_form: &Form) -> Option<std::cell::Ref<'_, Window>> {
    // Form struct doesn't expose sub_window getter
    None
}

/// Set form options.
///
/// This is the ncurses `set_form_opts()` function.
pub fn set_form_opts(form: &mut Form, opts: FormOpts) {
    form.set_opts(opts);
}

/// Get form options.
///
/// This is the ncurses `form_opts()` function.
pub fn form_opts(form: &Form) -> FormOpts {
    form.opts()
}

/// Turn on form options.
///
/// This is the ncurses `form_opts_on()` function.
pub fn form_opts_on(form: &mut Form, opts: FormOpts) {
    let current = form.opts();
    form.set_opts(current | opts);
}

/// Turn off form options.
///
/// This is the ncurses `form_opts_off()` function.
pub fn form_opts_off(form: &mut Form, opts: FormOpts) {
    let current = form.opts();
    form.set_opts(current & !opts);
}

/// Get the current field.
///
/// This is the ncurses `current_field()` function.
pub fn current_field(form: &Form) -> Option<Rc<RefCell<Field>>> {
    form.current_field()
}

/// Set the current field.
///
/// This is the ncurses `set_current_field()` function.
pub fn set_current_field(form: &mut Form, index: usize) -> Result<()> {
    form.set_current_field(index)
}

/// Get the field count.
///
/// This is the ncurses `field_count()` function.
pub fn field_count(form: &Form) -> usize {
    form.field_count()
}

/// Get the form fields.
///
/// This is the ncurses `form_fields()` function.
/// Returns a slice of all fields in the form.
pub fn form_fields(form: &Form) -> &[Rc<RefCell<Field>>] {
    form.fields()
}

/// Set the form fields.
///
/// This is the ncurses `set_form_fields()` function.
/// Replaces all fields in the form. The form must not be posted.
///
/// # Arguments
///
/// * `form` - The form to modify
/// * `fields` - The new fields for the form
///
/// # Returns
///
/// `Ok(())` on success, or an error if the form is currently posted.
pub fn set_form_fields(form: &mut Form, fields: Vec<Field>) -> Result<()> {
    form.set_fields(fields)
}

/// Get the current page.
///
/// This is the ncurses `form_page()` function.
pub fn form_page(form: &Form) -> i32 {
    form.page()
}

/// Set the current page.
///
/// This is the ncurses `set_form_page()` function.
pub fn set_form_page(form: &mut Form, page: i32) -> Result<()> {
    form.set_page(page)
}

/// Set form user pointer.
///
/// This is the ncurses `set_form_userptr()` function.
pub fn set_form_userptr<T: 'static>(form: &mut Form, data: T) {
    form.set_userptr(data);
}

/// Get form user pointer.
///
/// This is the ncurses `form_userptr()` function.
pub fn form_userptr<T: 'static>(form: &Form) -> Option<&T> {
    form.userptr::<T>()
}

/// Calculate the scale (size) required for the form.
///
/// This is the ncurses `scale_form()` function.
/// Returns (rows, cols) needed to display the form.
pub fn scale_form(form: &Form) -> (i32, i32) {
    let mut max_row = 0;
    let mut max_col = 0;

    for i in 0..form.field_count() {
        if let Some(field_ref) = form.field(i) {
            let field = field_ref.borrow();
            let (h, w, r, c) = field.dimensions();
            max_row = max_row.max(r + h);
            max_col = max_col.max(c + w);
        }
    }

    (max_row, max_col)
}

/// Validate the entire form.
///
/// This is the ncurses `form_validate()` function.
pub fn form_validate(form: &Form) -> bool {
    form.validate_all()
}

/// Get form data as a vector of field values.
///
/// This is a convenience function for retrieving all form data.
pub fn form_data(form: &Form) -> Vec<String> {
    form.data()
}

// ============================================================================
// Free functions for ncurses compatibility (Field)
// ============================================================================

/// Create a new field.
///
/// This is the ncurses `new_field()` function.
pub fn new_field(
    height: i32,
    width: i32,
    row: i32,
    col: i32,
    offscreen: i32,
    nbuffers: i32,
) -> Field {
    Field::new(height, width, row, col, offscreen, nbuffers)
}

/// Free a field.
///
/// This is the ncurses `free_field()` function.
/// In Rust, the field is automatically freed when dropped.
pub fn free_field(_field: Field) {
    // Field is dropped automatically
}

/// Duplicate a field.
///
/// This is the ncurses `dup_field()` function.
pub fn dup_field(field: &Field, row: i32, col: i32) -> Field {
    let mut new_field = Field::new(
        field.height(),
        field.width(),
        row,
        col,
        field.offscreen(),
        0,
    );
    new_field.set_buffer(&field.buffer());
    new_field.set_opts(field.opts());
    new_field.set_fore(field.fore());
    new_field.set_back(field.back());
    new_field.set_pad(field.pad());
    new_field
}

/// Link a field (create a connected copy).
///
/// This is the ncurses `link_field()` function.
/// In this implementation, it's the same as dup_field.
pub fn link_field(field: &Field, row: i32, col: i32) -> Field {
    dup_field(field, row, col)
}

/// Get field buffer content.
///
/// This is the ncurses `field_buffer()` function.
pub fn field_buffer(field: &Field, _buffer: i32) -> String {
    field.buffer()
}

/// Set field buffer content.
///
/// This is the ncurses `set_field_buffer()` function.
pub fn set_field_buffer(field: &mut Field, _buffer: i32, value: &str) -> Result<()> {
    field.set_buffer(value);
    Ok(())
}

/// Get field status (modified flag).
///
/// This is the ncurses `field_status()` function.
/// Returns true if the field has been modified.
pub fn field_status(_field: &Field) -> bool {
    // In our implementation, we don't track modification status
    true
}

/// Set field status.
///
/// This is the ncurses `set_field_status()` function.
pub fn set_field_status(_field: &mut Field, _status: bool) -> Result<()> {
    // No-op in our implementation
    Ok(())
}

/// Get field dimensions.
///
/// This is the ncurses `field_info()` function.
/// Returns (rows, cols, frow, fcol, nrow, nbuf).
pub fn field_info(field: &Field) -> (i32, i32, i32, i32, i32, i32) {
    let (h, w, r, c) = field.dimensions();
    (h, w, r, c, field.offscreen(), 0)
}

/// Move a field to a new position.
///
/// This is the ncurses `move_field()` function.
///
/// In ncurses, the field must be disconnected from any form before moving.
/// In this implementation, the caller is responsible for ensuring the field
/// is not part of a posted form.
///
/// # Arguments
///
/// * `field` - The field to move
/// * `row` - New row position (must be >= 0)
/// * `col` - New column position (must be >= 0)
///
/// # Returns
///
/// `Ok(())` on success, or an error if the position is invalid.
pub fn move_field(field: &mut Field, row: i32, col: i32) -> Result<()> {
    field.move_to(row, col)
}

/// Set field options.
///
/// This is the ncurses `set_field_opts()` function.
pub fn set_field_opts(field: &mut Field, opts: FieldOpts) {
    field.set_opts(opts);
}

/// Get field options.
///
/// This is the ncurses `field_opts()` function.
pub fn field_opts(field: &Field) -> FieldOpts {
    field.opts()
}

/// Turn on field options.
///
/// This is the ncurses `field_opts_on()` function.
pub fn field_opts_on(field: &mut Field, opts: FieldOpts) {
    let current = field.opts();
    field.set_opts(current | opts);
}

/// Turn off field options.
///
/// This is the ncurses `field_opts_off()` function.
pub fn field_opts_off(field: &mut Field, opts: FieldOpts) {
    let current = field.opts();
    field.set_opts(current & !opts);
}

/// Set the foreground attribute.
///
/// This is the ncurses `set_field_fore()` function.
pub fn set_field_fore(field: &mut Field, attr: AttrT) {
    field.set_fore(attr);
}

/// Get the foreground attribute.
///
/// This is the ncurses `field_fore()` function.
pub fn field_fore(field: &Field) -> AttrT {
    field.fore()
}

/// Set the background attribute.
///
/// This is the ncurses `set_field_back()` function.
pub fn set_field_back(field: &mut Field, attr: AttrT) {
    field.set_back(attr);
}

/// Get the background attribute.
///
/// This is the ncurses `field_back()` function.
pub fn field_back(field: &Field) -> AttrT {
    field.back()
}

/// Set the pad character.
///
/// This is the ncurses `set_field_pad()` function.
pub fn set_field_pad(field: &mut Field, pad: char) {
    field.set_pad(pad);
}

/// Get the pad character.
///
/// This is the ncurses `field_pad()` function.
pub fn field_pad(field: &Field) -> char {
    field.pad()
}

/// Set field user pointer.
///
/// This is the ncurses `set_field_userptr()` function.
pub fn set_field_userptr<T: 'static>(field: &mut Field, data: T) {
    field.set_userptr(data);
}

/// Get field user pointer.
///
/// This is the ncurses `field_userptr()` function.
pub fn field_userptr<T: 'static>(field: &Field) -> Option<&T> {
    field.userptr::<T>()
}

/// Set field type for validation.
///
/// This is the ncurses `set_field_type()` function.
pub fn set_field_type<T: FieldType + 'static>(field: &mut Field, field_type: T) {
    field.set_type(field_type);
}

/// Validate a field.
///
/// This is derived from field validation.
pub fn field_validate(field: &Field) -> bool {
    field.validate()
}

/// Get dynamic field info.
///
/// This is the ncurses `dynamic_field_info()` function.
/// Returns (rows, cols, max) where max is the maximum growth.
pub fn dynamic_field_info(field: &Field) -> (i32, i32, i32) {
    (field.height(), field.width(), field.total_rows())
}

/// Set the maximum growth for a dynamic field.
///
/// This is the ncurses `set_max_field()` function.
pub fn set_max_field(_field: &mut Field, _max: i32) -> Result<()> {
    // In our implementation, max is determined by offscreen rows
    Ok(())
}

/// Get field index in form.
///
/// This is the ncurses `field_index()` function.
pub fn field_index(form: &Form, field: &Rc<RefCell<Field>>) -> Option<usize> {
    for i in 0..form.field_count() {
        if let Some(f) = form.field(i) {
            if Rc::ptr_eq(&f, field) {
                return Some(i);
            }
        }
    }
    None
}

/// Check if field is visible.
///
/// This is derived from field options.
pub fn field_visible(field: &Field) -> bool {
    field.opts().contains(FieldOpts::O_VISIBLE)
}

/// Position the form cursor.
///
/// This is the ncurses `pos_form_cursor()` function.
pub fn pos_form_cursor(_form: &Form) -> Result<()> {
    // In our implementation, cursor positioning is handled internally
    Ok(())
}

/// Get the data behind the form (array of field contents).
///
/// This is the ncurses `data_behind()` function.
pub fn data_behind(_form: &Form) -> bool {
    // Check if there's data before the visible area
    false
}

/// Get the data ahead of the form.
///
/// This is the ncurses `data_ahead()` function.
pub fn data_ahead(_form: &Form) -> bool {
    // Check if there's data after the visible area
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field() {
        let mut field = Field::new(1, 20, 0, 0, 0, 0);
        field.set_buffer("Hello");
        assert_eq!(field.buffer(), "Hello");

        field.cursor_end();
        field.insert_char('!');
        assert_eq!(field.buffer(), "Hello!");
    }

    #[test]
    fn test_field_editing() {
        let mut field = Field::new(1, 20, 0, 0, 0, 0);

        field.insert_char('H');
        field.insert_char('i');
        assert_eq!(field.buffer(), "Hi");

        field.delete_prev();
        assert_eq!(field.buffer(), "H");

        field.clear();
        assert_eq!(field.buffer(), "");
    }

    #[test]
    fn test_field_validation() {
        let mut field = Field::new(1, 20, 0, 0, 0, 0);
        field.set_buffer("12345");
        field.set_type(TypeInteger {
            padding: 0,
            min: 0,
            max: 99999,
        });
        assert!(field.validate());

        field.set_buffer("abc");
        assert!(!field.validate());
    }

    #[test]
    fn test_field_multiline() {
        let mut field = Field::new(5, 20, 0, 0, 5, 0);
        field.set_buffer("Line 1\nLine 2\nLine 3");
        assert_eq!(field.buffer_rows(), 3);
        assert_eq!(field.buffer_row(0), Some("Line 1"));
        assert_eq!(field.buffer_row(1), Some("Line 2"));
        assert_eq!(field.buffer_row(2), Some("Line 3"));
    }

    #[test]
    fn test_field_cursor_movement() {
        let mut field = Field::new(5, 20, 0, 0, 0, 0);
        field.set_buffer("Line 1\nLine 2\nLine 3");

        // Test vertical movement
        field.cursor_down();
        assert_eq!(field.cursor_row(), 1);

        field.cursor_down();
        assert_eq!(field.cursor_row(), 2);

        field.cursor_up();
        assert_eq!(field.cursor_row(), 1);

        // Test horizontal movement
        field.cursor_right();
        field.cursor_right();
        assert_eq!(field.cursor_col(), 2);

        field.cursor_left();
        assert_eq!(field.cursor_col(), 1);

        // Test line home/end
        field.cursor_line_end();
        assert_eq!(field.cursor_col(), 6); // "Line 2" has 6 chars

        field.cursor_line_home();
        assert_eq!(field.cursor_col(), 0);
    }

    #[test]
    fn test_field_word_navigation() {
        let mut field = Field::new(1, 40, 0, 0, 0, 0);
        field.set_buffer("hello world test");

        field.cursor_next_word();
        assert_eq!(field.cursor_col(), 6); // At 'w' in "world"

        field.cursor_next_word();
        assert_eq!(field.cursor_col(), 12); // At 't' in "test"

        field.cursor_prev_word();
        assert_eq!(field.cursor_col(), 6); // Back at 'w' in "world"

        field.cursor_prev_word();
        assert_eq!(field.cursor_col(), 0); // Back at 'h' in "hello"
    }

    #[test]
    fn test_form() {
        let fields = vec![Field::new(1, 20, 0, 0, 0, 0), Field::new(1, 20, 1, 0, 0, 0)];
        let mut form = Form::new(fields);

        assert_eq!(form.field_count(), 2);
        assert_eq!(form.current_field_index(), 0);

        form.driver(REQ_NEXT_FIELD).unwrap();
        assert_eq!(form.current_field_index(), 1);

        form.driver(REQ_FIRST_FIELD).unwrap();
        assert_eq!(form.current_field_index(), 0);
    }

    #[test]
    fn test_form_input() {
        let fields = vec![Field::new(1, 20, 0, 0, 0, 0)];
        let mut form = Form::new(fields);

        form.driver('H' as i32).unwrap();
        form.driver('i' as i32).unwrap();

        let data = form.data();
        assert_eq!(data[0], "Hi");
    }

    #[test]
    fn test_form_multiline_requests() {
        let fields = vec![Field::new(5, 20, 0, 0, 5, 0)];
        let mut form = Form::new(fields);

        // Set multiline content
        if let Some(field) = form.current_field() {
            field.borrow_mut().set_buffer("Line 1\nLine 2\nLine 3");
        }

        // Test line navigation
        form.driver(REQ_NEXT_LINE).unwrap();
        if let Some(field) = form.current_field() {
            assert_eq!(field.borrow().cursor_row(), 1);
        }

        form.driver(REQ_PREV_LINE).unwrap();
        if let Some(field) = form.current_field() {
            assert_eq!(field.borrow().cursor_row(), 0);
        }
    }

    #[test]
    fn test_form_scrolling() {
        let fields = vec![Field::new(2, 10, 0, 0, 10, 0)];
        let mut form = Form::new(fields);

        // Set content that exceeds visible area
        if let Some(field) = form.current_field() {
            field
                .borrow_mut()
                .set_buffer("Line 1\nLine 2\nLine 3\nLine 4\nLine 5");
        }

        // Scroll forward
        form.driver(REQ_SCR_FLINE).unwrap();
        if let Some(field) = form.current_field() {
            assert_eq!(field.borrow().scroll_row(), 1);
        }

        form.driver(REQ_SCR_BLINE).unwrap();
        if let Some(field) = form.current_field() {
            assert_eq!(field.borrow().scroll_row(), 0);
        }
    }
}
