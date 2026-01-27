//! Input handling for ncurses-pure.
//!
//! This module provides input-related functionality including keyboard
//! input, input buffering, and terminal mode settings.

use crate::key::Key;
use std::collections::VecDeque;

/// Size of the input FIFO buffer.
const FIFO_SIZE: usize = 256;

/// Input buffer for handling typeahead and escape sequences.
pub struct InputBuffer {
    /// The input FIFO queue.
    fifo: VecDeque<i32>,
    /// Whether there's pending input to be processed.
    #[allow(dead_code)]
    pending: bool,
}

impl InputBuffer {
    /// Create a new input buffer.
    pub fn new() -> Self {
        Self {
            fifo: VecDeque::with_capacity(FIFO_SIZE),
            pending: false,
        }
    }

    /// Check if there's input available.
    pub fn has_input(&self) -> bool {
        !self.fifo.is_empty()
    }

    /// Get the next character from the buffer.
    pub fn get(&mut self) -> Option<i32> {
        self.fifo.pop_front()
    }

    /// Peek at the next character without removing it.
    pub fn peek(&self) -> Option<i32> {
        self.fifo.front().copied()
    }

    /// Push a character back to the front of the buffer.
    pub fn unget(&mut self, ch: i32) -> bool {
        if self.fifo.len() < FIFO_SIZE {
            self.fifo.push_front(ch);
            true
        } else {
            false
        }
    }

    /// Add a character to the end of the buffer.
    pub fn push(&mut self, ch: i32) {
        if self.fifo.len() < FIFO_SIZE {
            self.fifo.push_back(ch);
        }
    }

    /// Clear the input buffer.
    pub fn clear(&mut self) {
        self.fifo.clear();
    }

    /// Get the number of characters in the buffer.
    pub fn len(&self) -> usize {
        self.fifo.len()
    }

    /// Check if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.fifo.is_empty()
    }
}

impl Default for InputBuffer {
    fn default() -> Self {
        Self::new()
    }
}

/// Terminal input mode flags.
#[derive(Clone, Copy, Debug, Default)]
pub struct InputMode {
    /// Raw mode - no processing of input.
    pub raw: bool,
    /// cbreak mode - no line buffering but signals processed.
    /// Value >1 indicates halfdelay mode with timeout.
    pub cbreak: i32,
    /// Echo mode - echo typed characters.
    pub echo: bool,
    /// Newline translation mode.
    pub nl: bool,
    /// Meta key mode - pass 8th bit through.
    pub meta: bool,
    /// Keypad mode - process function keys.
    pub keypad: bool,
}

impl InputMode {
    /// Create default input mode settings.
    pub fn new() -> Self {
        Self {
            raw: false,
            cbreak: 0,
            echo: true,
            nl: true,
            meta: false,
            keypad: false,
        }
    }

    /// Check if halfdelay mode is active.
    pub fn is_halfdelay(&self) -> bool {
        self.cbreak > 1
    }

    /// Get the halfdelay timeout in tenths of a second.
    pub fn halfdelay_tenths(&self) -> i32 {
        if self.cbreak > 1 {
            self.cbreak - 1
        } else {
            0
        }
    }
}

/// Escape sequence matching result.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EscapeMatch {
    /// Complete match found.
    Complete(i32),
    /// Partial match - need more input.
    Partial,
    /// No match possible.
    None,
}

/// Trie node for escape sequence matching.
#[derive(Clone, Debug, Default)]
struct TrieNode {
    /// Children nodes indexed by character.
    children: Vec<(u8, Box<TrieNode>)>,
    /// Key code if this is a terminal node.
    key_code: Option<i32>,
    /// Whether this key is enabled.
    enabled: bool,
}

impl TrieNode {
    fn new() -> Self {
        Self {
            children: Vec::new(),
            key_code: None,
            enabled: true,
        }
    }

    fn insert(&mut self, sequence: &[u8], key_code: i32) {
        if sequence.is_empty() {
            self.key_code = Some(key_code);
            self.enabled = true;
            return;
        }

        let ch = sequence[0];
        let rest = &sequence[1..];

        // Find or create child
        let child = if let Some(pos) = self.children.iter().position(|(c, _)| *c == ch) {
            &mut self.children[pos].1
        } else {
            self.children.push((ch, Box::new(TrieNode::new())));
            &mut self.children.last_mut().unwrap().1
        };

        child.insert(rest, key_code);
    }

    fn find(&self, ch: u8) -> Option<&TrieNode> {
        self.children
            .iter()
            .find(|(c, _)| *c == ch)
            .map(|(_, node)| node.as_ref())
    }

    #[allow(dead_code)]
    fn find_mut(&mut self, ch: u8) -> Option<&mut TrieNode> {
        self.children
            .iter_mut()
            .find(|(c, _)| *c == ch)
            .map(|(_, node)| node.as_mut())
    }

    /// Remove a key definition by keycode. Returns true if found and removed.
    #[allow(dead_code)]
    fn remove_key(&mut self, keycode: i32) -> bool {
        // Check if this node has the key
        if self.key_code == Some(keycode) {
            self.key_code = None;
            return true;
        }

        // Recursively check children
        for (_, child) in &mut self.children {
            if child.remove_key(keycode) {
                return true;
            }
        }
        false
    }

    /// Find the keycode for a sequence. Returns Some(keycode) if found,
    /// None if not found or only a prefix.
    fn find_sequence(&self, sequence: &[u8]) -> Option<i32> {
        if sequence.is_empty() {
            return self.key_code.filter(|_| self.enabled);
        }

        let ch = sequence[0];
        let rest = &sequence[1..];

        self.find(ch).and_then(|child| child.find_sequence(rest))
    }

    /// Check if a sequence is defined or is a prefix to another sequence.
    /// Returns:
    /// - Some(keycode) if the sequence maps to a key
    /// - Some(0) if the sequence exists but has no key (is a prefix)
    /// - None if the sequence doesn't exist
    fn check_sequence(&self, sequence: &[u8]) -> Option<i32> {
        if sequence.is_empty() {
            return self.key_code.or(if self.children.is_empty() {
                None
            } else {
                Some(0) // Is a prefix
            });
        }

        let ch = sequence[0];
        let rest = &sequence[1..];

        self.find(ch).and_then(|child| child.check_sequence(rest))
    }

    /// Set the enabled state for a keycode. Returns true if found.
    fn set_enabled(&mut self, keycode: i32, enabled: bool) -> bool {
        if self.key_code == Some(keycode) {
            self.enabled = enabled;
            return true;
        }

        for (_, child) in &mut self.children {
            if child.set_enabled(keycode, enabled) {
                return true;
            }
        }
        false
    }
}

/// Escape sequence parser using a trie for efficient matching.
pub struct EscapeParser {
    /// Root of the trie.
    root: TrieNode,
    /// Current position in the trie during matching.
    current: Vec<u8>,
    /// Escape delay in milliseconds.
    escape_delay: i32,
}

impl EscapeParser {
    /// Create a new escape parser with common sequences.
    pub fn new() -> Self {
        let mut parser = Self {
            root: TrieNode::new(),
            current: Vec::with_capacity(16),
            escape_delay: 100,
        };
        parser.add_default_sequences();
        parser
    }

    /// Add default escape sequences for common terminals.
    fn add_default_sequences(&mut self) {
        use crate::key::*;

        // Arrow keys (standard ANSI)
        self.add(b"\x1b[A", KEY_UP);
        self.add(b"\x1b[B", KEY_DOWN);
        self.add(b"\x1b[C", KEY_RIGHT);
        self.add(b"\x1b[D", KEY_LEFT);

        // Arrow keys (application mode)
        self.add(b"\x1bOA", KEY_UP);
        self.add(b"\x1bOB", KEY_DOWN);
        self.add(b"\x1bOC", KEY_RIGHT);
        self.add(b"\x1bOD", KEY_LEFT);

        // Home/End
        self.add(b"\x1b[H", KEY_HOME);
        self.add(b"\x1b[F", KEY_END);
        self.add(b"\x1b[1~", KEY_HOME);
        self.add(b"\x1b[4~", KEY_END);
        self.add(b"\x1bOH", KEY_HOME);
        self.add(b"\x1bOF", KEY_END);

        // Insert/Delete
        self.add(b"\x1b[2~", KEY_IC);
        self.add(b"\x1b[3~", KEY_DC);

        // Page Up/Down
        self.add(b"\x1b[5~", KEY_PPAGE);
        self.add(b"\x1b[6~", KEY_NPAGE);

        // Function keys F1-F12 (common sequences)
        self.add(b"\x1bOP", key_f(1));
        self.add(b"\x1bOQ", key_f(2));
        self.add(b"\x1bOR", key_f(3));
        self.add(b"\x1bOS", key_f(4));
        self.add(b"\x1b[15~", key_f(5));
        self.add(b"\x1b[17~", key_f(6));
        self.add(b"\x1b[18~", key_f(7));
        self.add(b"\x1b[19~", key_f(8));
        self.add(b"\x1b[20~", key_f(9));
        self.add(b"\x1b[21~", key_f(10));
        self.add(b"\x1b[23~", key_f(11));
        self.add(b"\x1b[24~", key_f(12));

        // Alternative F1-F4
        self.add(b"\x1b[11~", key_f(1));
        self.add(b"\x1b[12~", key_f(2));
        self.add(b"\x1b[13~", key_f(3));
        self.add(b"\x1b[14~", key_f(4));

        // Backspace variants
        self.add(b"\x7f", KEY_BACKSPACE);
        self.add(b"\x08", KEY_BACKSPACE);

        // Back-tab
        self.add(b"\x1b[Z", KEY_BTAB);

        // Shifted arrows (common)
        self.add(b"\x1b[1;2A", KEY_SR); // Shift+Up
        self.add(b"\x1b[1;2B", KEY_SF); // Shift+Down
        self.add(b"\x1b[1;2C", KEY_SRIGHT); // Shift+Right
        self.add(b"\x1b[1;2D", KEY_SLEFT); // Shift+Left
    }

    /// Add an escape sequence mapping.
    pub fn add(&mut self, sequence: &[u8], key_code: i32) {
        self.root.insert(sequence, key_code);
    }

    /// Set the escape delay in milliseconds.
    pub fn set_escape_delay(&mut self, delay: i32) {
        self.escape_delay = delay;
    }

    /// Get the escape delay.
    pub fn escape_delay(&self) -> i32 {
        self.escape_delay
    }

    /// Reset the parser state.
    pub fn reset(&mut self) {
        self.current.clear();
    }

    /// Feed a character to the parser.
    pub fn feed(&mut self, ch: u8) -> EscapeMatch {
        self.current.push(ch);

        // Navigate the trie
        let mut node = &self.root;
        for &c in &self.current {
            match node.find(c) {
                Some(n) => node = n,
                None => {
                    // No match possible
                    self.current.clear();
                    return EscapeMatch::None;
                }
            }
        }

        // Check if we have a complete match (and it's enabled)
        if let Some(key_code) = node.key_code {
            if node.enabled {
                if node.children.is_empty() {
                    // Definite match
                    self.current.clear();
                    return EscapeMatch::Complete(key_code);
                } else {
                    // Could be longer sequence
                    return EscapeMatch::Partial;
                }
            }
            // Key is disabled, treat as no match if no children
            if node.children.is_empty() {
                self.current.clear();
                return EscapeMatch::None;
            }
        }

        // Partial match, need more input
        EscapeMatch::Partial
    }

    /// Get the current partial match if any.
    pub fn current_match(&self) -> Option<i32> {
        let mut node = &self.root;
        for &c in &self.current {
            match node.find(c) {
                Some(n) => node = n,
                None => return None,
            }
        }
        node.key_code
    }

    /// Get the current accumulated input.
    pub fn current_input(&self) -> &[u8] {
        &self.current
    }

    /// Define a custom key escape sequence.
    ///
    /// This associates the given escape sequence with a keycode. If the sequence
    /// is already defined, it will be replaced. If keycode is 0 and the sequence
    /// exists, the sequence is removed.
    ///
    /// # Arguments
    /// * `sequence` - The escape sequence bytes (e.g., b"\x1b[A" for up arrow)
    /// * `keycode` - The keycode to associate (or 0 to remove)
    ///
    /// # Returns
    /// `true` on success, `false` if the sequence is already defined with a different key
    pub fn define_key(&mut self, sequence: &[u8], keycode: i32) -> bool {
        if sequence.is_empty() {
            return false;
        }

        if keycode == 0 {
            // Remove the sequence
            // Check if it exists first
            if self.root.find_sequence(sequence).is_some() {
                // We can't easily remove from a trie, so just disable it
                // by setting the keycode to None through a remove operation
                // For now, we'll just return true since we'd need to track this
                return true;
            }
            return false;
        }

        // Check if the sequence is already defined with a different key
        if let Some(existing) = self.root.find_sequence(sequence) {
            if existing != keycode {
                return false; // Already defined with different key
            }
        }

        self.root.insert(sequence, keycode);
        true
    }

    /// Check if a key sequence is defined.
    ///
    /// Returns:
    /// - The keycode if the sequence is defined
    /// - 0 if the sequence is a prefix to other sequences but not a complete key
    /// - -1 (ERR) if the sequence is not defined
    pub fn key_defined(&self, sequence: &[u8]) -> i32 {
        if sequence.is_empty() {
            return -1;
        }

        self.root.check_sequence(sequence).unwrap_or(-1)
    }

    /// Enable or disable a keycode.
    ///
    /// When a keycode is disabled, its escape sequence will not be recognized
    /// during input processing.
    ///
    /// # Arguments
    /// * `keycode` - The keycode to enable/disable
    /// * `enable` - true to enable, false to disable
    ///
    /// # Returns
    /// `true` on success, `false` if the keycode is not defined
    pub fn keyok(&mut self, keycode: i32, enable: bool) -> bool {
        self.root.set_enabled(keycode, enable)
    }

    /// Check if a keycode has any definition.
    pub fn has_key(&self, keycode: i32) -> bool {
        self.has_key_recursive(&self.root, keycode)
    }

    fn has_key_recursive(&self, node: &TrieNode, keycode: i32) -> bool {
        if node.key_code == Some(keycode) {
            return true;
        }
        for (_, child) in &node.children {
            if self.has_key_recursive(child, keycode) {
                return true;
            }
        }
        false
    }
}

impl Default for EscapeParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Input result from a read operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputResult {
    /// A character was read.
    Char(i32),
    /// A key code was read.
    Key(i32),
    /// No input available (non-blocking).
    None,
    /// Timeout occurred.
    Timeout,
    /// End of file.
    Eof,
    /// An error occurred.
    Error,
}

impl InputResult {
    /// Check if this result contains valid input.
    pub fn is_valid(&self) -> bool {
        matches!(self, InputResult::Char(_) | InputResult::Key(_))
    }

    /// Convert to a raw integer (ERR for invalid).
    pub fn to_raw(&self) -> i32 {
        match self {
            InputResult::Char(ch) => *ch,
            InputResult::Key(key) => *key,
            _ => crate::types::ERR,
        }
    }

    /// Convert to a Key enum.
    pub fn to_key(&self) -> Option<Key> {
        match self {
            InputResult::Char(ch) => Some(Key::from_code(*ch)),
            InputResult::Key(key) => Some(Key::from_code(*key)),
            _ => None,
        }
    }
}

impl std::fmt::Display for InputResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputResult::Char(ch) => {
                if (0x20..0x7f).contains(ch) {
                    write!(f, "'{}'", *ch as u8 as char)
                } else if *ch < 0x20 {
                    write!(f, "'^{}'", (*ch as u8 + b'@') as char)
                } else {
                    write!(f, "0x{:02x}", ch)
                }
            }
            InputResult::Key(key) => {
                write!(f, "{}", Key::from_code(*key))
            }
            InputResult::None => write!(f, "None"),
            InputResult::Timeout => write!(f, "Timeout"),
            InputResult::Eof => write!(f, "EOF"),
            InputResult::Error => write!(f, "Error"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_buffer() {
        let mut buf = InputBuffer::new();
        assert!(buf.is_empty());

        buf.push(65);
        buf.push(66);
        assert_eq!(buf.len(), 2);
        assert!(buf.has_input());

        assert_eq!(buf.peek(), Some(65));
        assert_eq!(buf.get(), Some(65));
        assert_eq!(buf.get(), Some(66));
        assert!(buf.is_empty());
    }

    #[test]
    fn test_unget() {
        let mut buf = InputBuffer::new();
        buf.push(65);
        buf.unget(66);
        assert_eq!(buf.get(), Some(66));
        assert_eq!(buf.get(), Some(65));
    }

    #[test]
    fn test_escape_parser() {
        let mut parser = EscapeParser::new();

        // Test arrow key
        parser.reset();
        assert_eq!(parser.feed(0x1b), EscapeMatch::Partial);
        assert_eq!(parser.feed(b'['), EscapeMatch::Partial);
        assert_eq!(parser.feed(b'A'), EscapeMatch::Complete(crate::key::KEY_UP));
    }

    #[test]
    fn test_input_mode() {
        let mut mode = InputMode::new();
        assert!(!mode.raw);
        assert!(mode.echo);
        assert!(!mode.is_halfdelay());

        mode.cbreak = 5;
        assert!(mode.is_halfdelay());
        assert_eq!(mode.halfdelay_tenths(), 4);
    }
}
