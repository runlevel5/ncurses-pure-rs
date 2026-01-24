//! Line data structure for window content.
//!
//! This module defines the internal representation of a line of window
//! content, including change tracking for efficient refresh.

use crate::types::{NcursesSize, NEWINDEX, NOCHANGE};

#[cfg(feature = "wide")]
use crate::wide::CCharT;

/// A single line of window data.
///
/// This structure holds the content of one line in a window, along with
/// change tracking information used by the refresh mechanism.
#[derive(Clone)]
pub struct LineData {
    /// The character data for this line.
    ///
    /// For non-wide builds, this is a vector of `ChType` values.
    /// For wide-character builds, this is a vector of `CCharT` values.
    #[cfg(not(feature = "wide"))]
    text: Vec<ChType>,

    #[cfg(feature = "wide")]
    text: Vec<CCharT>,

    /// First changed column in this line (-1 = no change).
    ///
    /// This is set to the leftmost position that has been modified
    /// since the last refresh. A value of NOCHANGE (-1) indicates
    /// that no changes have been made.
    firstchar: NcursesSize,

    /// Last changed column in this line.
    ///
    /// This is set to the rightmost position that has been modified
    /// since the last refresh.
    lastchar: NcursesSize,

    /// Index of this line at last update.
    ///
    /// Used for scroll optimization. A value of NEWINDEX (-1) indicates
    /// that this is a new line (inserted or created by scrolling).
    oldindex: NcursesSize,
}

impl LineData {
    /// Create a new line with the specified width.
    pub fn new(width: usize) -> Self {
        Self {
            #[cfg(not(feature = "wide"))]
            text: vec![b' ' as ChType | A_NORMAL; width],
            #[cfg(feature = "wide")]
            text: vec![CCharT::from_char(' '); width],
            firstchar: NOCHANGE,
            lastchar: NOCHANGE,
            oldindex: NEWINDEX,
        }
    }

    /// Get the width of this line.
    #[inline]
    pub fn width(&self) -> usize {
        self.text.len()
    }

    /// Check if this line has any changes.
    #[inline]
    pub fn is_touched(&self) -> bool {
        self.firstchar != NOCHANGE
    }

    /// Get the range of changed columns.
    ///
    /// Returns `None` if the line has no changes.
    pub fn changed_range(&self) -> Option<(usize, usize)> {
        if self.firstchar == NOCHANGE {
            None
        } else {
            Some((self.firstchar as usize, self.lastchar as usize))
        }
    }

    /// Mark the entire line as changed.
    pub fn touch(&mut self) {
        self.firstchar = 0;
        self.lastchar = (self.text.len() - 1) as NcursesSize;
    }

    /// Clear the change tracking for this line.
    pub fn untouch(&mut self) {
        self.firstchar = NOCHANGE;
        self.lastchar = NOCHANGE;
    }

    /// Mark a specific position as changed.
    #[inline]
    pub fn mark_changed(&mut self, x: usize) {
        let x = x as NcursesSize;
        if self.firstchar == NOCHANGE {
            self.firstchar = x;
            self.lastchar = x;
        } else {
            if x < self.firstchar {
                self.firstchar = x;
            }
            if x > self.lastchar {
                self.lastchar = x;
            }
        }
    }

    /// Get the old index of this line.
    #[inline]
    pub fn oldindex(&self) -> NcursesSize {
        self.oldindex
    }

    /// Set the old index of this line.
    #[inline]
    pub fn set_oldindex(&mut self, index: NcursesSize) {
        self.oldindex = index;
    }

    /// Get a character at the specified position.
    #[cfg(not(feature = "wide"))]
    #[inline]
    pub fn get(&self, x: usize) -> ChType {
        self.text.get(x).copied().unwrap_or(0)
    }

    /// Get a character at the specified position (wide character version).
    #[cfg(feature = "wide")]
    #[inline]
    pub fn get(&self, x: usize) -> CCharT {
        self.text.get(x).copied().unwrap_or_default()
    }

    /// Set a character at the specified position.
    #[cfg(not(feature = "wide"))]
    #[inline]
    pub fn set(&mut self, x: usize, ch: ChType) {
        if x < self.text.len() {
            self.text[x] = ch;
            self.mark_changed(x);
        }
    }

    /// Set a character at the specified position (wide character version).
    #[cfg(feature = "wide")]
    #[inline]
    pub fn set(&mut self, x: usize, ch: CCharT) {
        if x < self.text.len() {
            self.text[x] = ch;
            self.mark_changed(x);
        }
    }

    /// Get a slice of the text data.
    #[cfg(not(feature = "wide"))]
    pub fn text(&self) -> &[ChType] {
        &self.text
    }

    /// Get a slice of the text data (wide character version).
    #[cfg(feature = "wide")]
    pub fn text(&self) -> &[CCharT] {
        &self.text
    }

    /// Get a mutable slice of the text data.
    #[cfg(not(feature = "wide"))]
    pub fn text_mut(&mut self) -> &mut [ChType] {
        &mut self.text
    }

    /// Get a mutable slice of the text data (wide character version).
    #[cfg(feature = "wide")]
    pub fn text_mut(&mut self) -> &mut [CCharT] {
        &mut self.text
    }

    /// Fill the line with a character.
    #[cfg(not(feature = "wide"))]
    pub fn fill(&mut self, ch: ChType) {
        self.text.fill(ch);
        self.touch();
    }

    /// Fill the line with a character (wide character version).
    #[cfg(feature = "wide")]
    pub fn fill(&mut self, ch: CCharT) {
        self.text.fill(ch);
        self.touch();
    }

    /// Fill a range of the line with a character.
    #[cfg(not(feature = "wide"))]
    pub fn fill_range(&mut self, start: usize, end: usize, ch: ChType) {
        let end = end.min(self.text.len());
        for x in start..end {
            self.text[x] = ch;
        }
        if start < end {
            self.mark_changed(start);
            self.mark_changed(end - 1);
        }
    }

    /// Fill a range of the line with a character (wide character version).
    #[cfg(feature = "wide")]
    pub fn fill_range(&mut self, start: usize, end: usize, ch: CCharT) {
        let end = end.min(self.text.len());
        for x in start..end {
            self.text[x] = ch;
        }
        if start < end {
            self.mark_changed(start);
            self.mark_changed(end - 1);
        }
    }

    /// Copy content from another line.
    pub fn copy_from(&mut self, other: &LineData) {
        let len = self.text.len().min(other.text.len());
        self.text[..len].copy_from_slice(&other.text[..len]);
        self.touch();
    }

    /// Resize the line to a new width.
    #[cfg(not(feature = "wide"))]
    pub fn resize(&mut self, new_width: usize, fill: ChType) {
        self.text.resize(new_width, fill);
        self.touch();
    }

    /// Resize the line to a new width (wide character version).
    #[cfg(feature = "wide")]
    pub fn resize(&mut self, new_width: usize, fill: CCharT) {
        self.text.resize(new_width, fill);
        self.touch();
    }

    /// Insert characters at a position, shifting content right.
    #[cfg(not(feature = "wide"))]
    pub fn insert(&mut self, x: usize, ch: ChType, count: usize) {
        if x >= self.text.len() {
            return;
        }
        let width = self.text.len();
        let count = count.min(width - x);
        // Shift content right using copy_within (more efficient than manual loop)
        self.text.copy_within(x..width - count, x + count);
        // Insert the character
        for i in x..(x + count) {
            self.text[i] = ch;
        }
        self.mark_changed(x);
        self.mark_changed(width - 1);
    }

    /// Insert characters at a position, shifting content right (wide character version).
    #[cfg(feature = "wide")]
    pub fn insert(&mut self, x: usize, ch: CCharT, count: usize) {
        if x >= self.text.len() {
            return;
        }
        let width = self.text.len();
        let count = count.min(width - x);
        // Shift content right using copy_within (more efficient than manual loop)
        self.text.copy_within(x..width - count, x + count);
        // Insert the character
        for i in x..(x + count) {
            self.text[i] = ch;
        }
        self.mark_changed(x);
        self.mark_changed(width - 1);
    }

    /// Delete characters at a position, shifting content left.
    #[cfg(not(feature = "wide"))]
    pub fn delete(&mut self, x: usize, count: usize, fill: ChType) {
        if x >= self.text.len() {
            return;
        }
        let width = self.text.len();
        let count = count.min(width - x);
        // Shift content left using copy_within (more efficient than manual loop)
        self.text.copy_within(x + count..width, x);
        // Fill the vacated space
        self.text[width - count..width].fill(fill);
        self.mark_changed(x);
        self.mark_changed(width - 1);
    }

    /// Delete characters at a position, shifting content left (wide character version).
    #[cfg(feature = "wide")]
    pub fn delete(&mut self, x: usize, count: usize, fill: CCharT) {
        if x >= self.text.len() {
            return;
        }
        let width = self.text.len();
        let count = count.min(width - x);
        // Shift content left using copy_within (more efficient than manual loop)
        self.text.copy_within(x + count..width, x);
        // Fill the vacated space
        self.text[width - count..width].fill(fill);
        self.mark_changed(x);
        self.mark_changed(width - 1);
    }
}

impl std::fmt::Debug for LineData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LineData")
            .field("width", &self.text.len())
            .field("firstchar", &self.firstchar)
            .field("lastchar", &self.lastchar)
            .field("oldindex", &self.oldindex)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_creation() {
        let line = LineData::new(80);
        assert_eq!(line.width(), 80);
        assert!(!line.is_touched());
    }

    #[test]
    fn test_change_tracking() {
        let mut line = LineData::new(80);
        assert!(!line.is_touched());

        line.mark_changed(10);
        assert!(line.is_touched());
        assert_eq!(line.changed_range(), Some((10, 10)));

        line.mark_changed(20);
        assert_eq!(line.changed_range(), Some((10, 20)));

        line.mark_changed(5);
        assert_eq!(line.changed_range(), Some((5, 20)));

        line.untouch();
        assert!(!line.is_touched());
    }

    #[test]
    fn test_touch() {
        let mut line = LineData::new(80);
        line.touch();
        assert!(line.is_touched());
        assert_eq!(line.changed_range(), Some((0, 79)));
    }

    #[cfg(not(feature = "wide"))]
    #[test]
    fn test_set_get() {
        let mut line = LineData::new(80);
        line.set(10, b'A' as ChType);
        assert_eq!(line.get(10), b'A' as ChType);
        assert!(line.is_touched());
    }
}
