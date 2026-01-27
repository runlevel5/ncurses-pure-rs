//! # Tracing and Debugging Support
//!
//! This module provides debugging and tracing facilities for ncurses applications.
//! These functions are only available when the `trace` feature is enabled.
//!
//! ## Overview
//!
//! The trace system allows you to log various aspects of ncurses operations
//! to a trace file for debugging purposes. This is particularly useful when
//! developing terminal applications.
//!
//! ## Example
//!
//! ```rust,ignore
//! use ncurses::trace::*;
//!
//! // Enable full tracing
//! trace(TRACE_MAXIMUM);
//!
//! // Log a custom message
//! tracef("Starting application");
//!
//! // Get a string representation of attributes
//! let attr_str = traceattr(attr::A_BOLD | attr::A_UNDERLINE);
//! ```

use crate::types::{AttrT, ChType};
use crate::Window;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(feature = "wide")]
use crate::wide::CCharT;

#[cfg(feature = "mouse")]
use crate::mouse::MouseEvent;

// ============================================================================
// Trace level constants
// ============================================================================

/// Disable tracing.
pub const TRACE_DISABLE: u32 = 0x0000;

/// Trace user and system times of updates.
pub const TRACE_TIMES: u32 = 0x0001;

/// Trace tputs calls.
pub const TRACE_TPUTS: u32 = 0x0002;

/// Trace update actions, old & new screens.
pub const TRACE_UPDATE: u32 = 0x0004;

/// Trace cursor movement and scrolling.
pub const TRACE_MOVE: u32 = 0x0008;

/// Trace all character outputs.
pub const TRACE_CHARPUT: u32 = 0x0010;

/// Trace all update actions (includes screen dumps).
pub const TRACE_ORDINARY: u32 = 0x001F;

/// Trace all curses calls with parameters and return values.
pub const TRACE_CALLS: u32 = 0x0020;

/// Trace virtual character puts (addch calls).
pub const TRACE_VIRTPUT: u32 = 0x0040;

/// Trace low-level input processing, including timeouts.
pub const TRACE_IEVENT: u32 = 0x0080;

/// Trace state of TTY control bits.
pub const TRACE_BITS: u32 = 0x0100;

/// Trace internal/nested calls.
pub const TRACE_ICALLS: u32 = 0x0200;

/// Trace per-character calls.
pub const TRACE_CCALLS: u32 = 0x0400;

/// Trace read/write of terminfo/termcap data.
pub const TRACE_DATABASE: u32 = 0x0800;

/// Trace changes to video attributes and colors.
pub const TRACE_ATTRS: u32 = 0x1000;

/// Maximum trace level - enables all trace features.
pub const TRACE_MAXIMUM: u32 = 0xFFFF;

// ============================================================================
// Global trace state
// ============================================================================

/// Thread-safe global trace state.
struct TraceState {
    /// Current trace level bitmask.
    level: u32,
    /// Trace output file.
    file: Option<File>,
    /// String buffers for traceattr2, tracechtype2, etc.
    buffers: [String; 10],
}

impl Default for TraceState {
    fn default() -> Self {
        Self {
            level: TRACE_DISABLE,
            file: None,
            buffers: Default::default(),
        }
    }
}

static TRACE_STATE: OnceLock<Mutex<TraceState>> = OnceLock::new();

fn get_trace_state() -> &'static Mutex<TraceState> {
    TRACE_STATE.get_or_init(|| Mutex::new(TraceState::default()))
}

// ============================================================================
// Main trace functions
// ============================================================================

/// Set the trace level and open the trace file if needed.
///
/// Calling `trace()` with a nonzero parameter opens the file "trace" in the
/// current directory for output. The parameter is formed by OR'ing values
/// from the `TRACE_*` constants.
///
/// # Arguments
///
/// * `param` - Bitmask of trace levels to enable
///
/// # Example
///
/// ```rust,ignore
/// use ncurses::trace::*;
///
/// // Enable call tracing and attribute tracing
/// trace(TRACE_CALLS | TRACE_ATTRS);
///
/// // Disable all tracing
/// trace(TRACE_DISABLE);
/// ```
pub fn trace(param: u32) {
    let mut state = get_trace_state().lock().unwrap();

    if param != TRACE_DISABLE && state.file.is_none() {
        // Open trace file
        if let Ok(file) = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open("trace")
        {
            state.file = Some(file);
        }
    } else if param == TRACE_DISABLE {
        state.file = None;
    }

    state.level = param;
}

/// Get the current trace level.
pub fn curses_trace() -> u32 {
    get_trace_state().lock().unwrap().level
}

/// Write a formatted message to the trace file.
///
/// This is the primary function for writing custom debug messages to the
/// trace output. The function is similar to printf in C.
///
/// # Arguments
///
/// * `msg` - The message to write
///
/// # Example
///
/// ```rust,ignore
/// use ncurses::trace::*;
///
/// trace(TRACE_CALLS);
/// tracef("Function called with x=10, y=20");
/// ```
pub fn tracef(msg: &str) {
    let mut state = get_trace_state().lock().unwrap();

    if state.level == TRACE_DISABLE {
        return;
    }

    if let Some(ref mut file) = state.file {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_micros())
            .unwrap_or(0);

        let _ = writeln!(file, "[{:012}] {}", timestamp, msg);
        let _ = file.flush();
    }
}

/// Write a formatted message to the trace file (C-style alias).
///
/// This is an alias for `tracef()` to match the ncurses `_tracef()` function name.
#[inline]
pub fn _tracef(msg: &str) {
    tracef(msg);
}

/// Dump window contents to the trace file.
///
/// Writes a representation of the window's contents to the trace file,
/// useful for debugging display issues.
///
/// # Arguments
///
/// * `label` - A label to identify this dump
/// * `win` - The window to dump
pub fn tracedump(label: &str, win: &Window) {
    let mut state = get_trace_state().lock().unwrap();

    if state.level == TRACE_DISABLE || (state.level & TRACE_UPDATE) == 0 {
        return;
    }

    if let Some(ref mut file) = state.file {
        let _ = writeln!(file, "=== WINDOW DUMP: {} ===", label);
        let _ = writeln!(
            file,
            "Position: ({}, {}), Size: {}x{}",
            win.getbegy(),
            win.getbegx(),
            win.getmaxy(),
            win.getmaxx()
        );
        let _ = writeln!(file, "Cursor: ({}, {})", win.getcury(), win.getcurx());
        let _ = writeln!(
            file,
            "Flags: keypad={}, scroll={}, leaveok={}",
            win.is_keypad(),
            win.is_scrollok(),
            win.is_leaveok()
        );
        let _ = writeln!(file, "=== END DUMP ===");
        let _ = file.flush();
    }
}

/// Dump window contents to the trace file (C-style alias).
#[inline]
pub fn _tracedump(label: &str, win: &Window) {
    tracedump(label, win);
}

// ============================================================================
// Attribute tracing functions
// ============================================================================

/// Return a string representation of video attributes.
///
/// This function returns a human-readable string describing the given
/// attributes, useful for debugging attribute-related issues.
///
/// # Arguments
///
/// * `attr` - The attributes to describe
///
/// # Returns
///
/// A string describing the attributes
pub fn traceattr(attr: AttrT) -> String {
    traceattr_impl(attr)
}

/// Return a string representation of video attributes (C-style alias).
#[inline]
pub fn _traceattr(attr: AttrT) -> String {
    traceattr(attr)
}

/// Return a string representation of video attributes using a specific buffer.
///
/// This variant allows specifying a buffer number (0-9) for storing the result,
/// useful when you need to trace multiple attributes in the same expression.
///
/// # Arguments
///
/// * `buffer` - Buffer number (0-9)
/// * `ch` - The chtype containing attributes
pub fn traceattr2(buffer: usize, ch: ChType) -> String {
    let mut state = get_trace_state().lock().unwrap();
    let buffer_idx = buffer.min(9);
    let result = traceattr_impl(ch as AttrT);
    state.buffers[buffer_idx] = result.clone();
    result
}

/// Return a string representation of video attributes using a specific buffer (C-style alias).
#[inline]
pub fn _traceattr2(buffer: usize, ch: ChType) -> String {
    traceattr2(buffer, ch)
}

fn traceattr_impl(attr: AttrT) -> String {
    use crate::attr;

    let mut parts: Vec<String> = Vec::new();

    if attr & attr::A_STANDOUT != 0 {
        parts.push("A_STANDOUT".to_string());
    }
    if attr & attr::A_UNDERLINE != 0 {
        parts.push("A_UNDERLINE".to_string());
    }
    if attr & attr::A_REVERSE != 0 {
        parts.push("A_REVERSE".to_string());
    }
    if attr & attr::A_BLINK != 0 {
        parts.push("A_BLINK".to_string());
    }
    if attr & attr::A_DIM != 0 {
        parts.push("A_DIM".to_string());
    }
    if attr & attr::A_BOLD != 0 {
        parts.push("A_BOLD".to_string());
    }
    if attr & attr::A_PROTECT != 0 {
        parts.push("A_PROTECT".to_string());
    }
    if attr & attr::A_INVIS != 0 {
        parts.push("A_INVIS".to_string());
    }
    if attr & attr::A_ALTCHARSET != 0 {
        parts.push("A_ALTCHARSET".to_string());
    }
    if attr & attr::A_ITALIC != 0 {
        parts.push("A_ITALIC".to_string());
    }

    let pair = attr::pair_number(attr);
    if pair != 0 {
        parts.push(format!("COLOR_PAIR({})", pair));
    }

    if parts.is_empty() {
        "A_NORMAL".to_string()
    } else {
        parts.join("|")
    }
}

// ============================================================================
// Character tracing functions
// ============================================================================

/// Return a printable representation of a character.
///
/// # Arguments
///
/// * `ch` - The character code
///
/// # Returns
///
/// A string representation of the character
pub fn tracechar(ch: i32) -> String {
    if (32..127).contains(&ch) {
        format!("'{}'", ch as u8 as char)
    } else if (0..32).contains(&ch) {
        format!("^{}", (ch + 64) as u8 as char)
    } else if ch == 127 {
        "^?".to_string()
    } else if ch < 0 {
        format!("ERR({})", ch)
    } else {
        format!("\\x{:02X}", ch)
    }
}

/// Return a printable representation of a character (C-style alias).
#[inline]
pub fn _tracechar(ch: i32) -> String {
    tracechar(ch)
}

/// Return a string representation of a chtype value.
///
/// This includes both the character and its attributes.
///
/// # Arguments
///
/// * `ch` - The chtype value
pub fn tracechtype(ch: ChType) -> String {
    let char_part = (ch & 0xFF) as u8 as char;
    let attr_part = ch & !0xFF;

    let char_str = if char_part.is_ascii_graphic() || char_part == ' ' {
        format!("'{}'", char_part)
    } else {
        format!("\\x{:02X}", char_part as u8)
    };

    if attr_part == 0 {
        char_str
    } else {
        format!("{} | {}", char_str, traceattr(attr_part as AttrT))
    }
}

/// Return a string representation of a chtype value (C-style alias).
#[inline]
pub fn _tracechtype(ch: ChType) -> String {
    tracechtype(ch)
}

/// Return a string representation of a chtype value using a specific buffer.
///
/// # Arguments
///
/// * `buffer` - Buffer number (0-9)
/// * `ch` - The chtype value
pub fn tracechtype2(buffer: usize, ch: ChType) -> String {
    let mut state = get_trace_state().lock().unwrap();
    let buffer_idx = buffer.min(9);
    let result = tracechtype(ch);
    state.buffers[buffer_idx] = result.clone();
    result
}

/// Return a string representation of a chtype value using a specific buffer (C-style alias).
#[inline]
pub fn _tracechtype2(buffer: usize, ch: ChType) -> String {
    tracechtype2(buffer, ch)
}

// ============================================================================
// Wide character tracing functions
// ============================================================================

/// Return a string representation of a cchar_t value.
#[cfg(feature = "wide")]
pub fn tracecchar_t(cch: &CCharT) -> String {
    let ch = cch.spacing_char();
    let attr = cch.attrs();

    let char_str = if ch.is_ascii_graphic() || ch == ' ' {
        format!("'{}'", ch)
    } else {
        format!("U+{:04X}", ch as u32)
    };

    if attr == 0 {
        char_str
    } else {
        format!("{} | {}", char_str, traceattr(attr))
    }
}

/// Return a string representation of a cchar_t value (C-style alias).
#[cfg(feature = "wide")]
#[inline]
pub fn _tracecchar_t(cch: &CCharT) -> String {
    tracecchar_t(cch)
}

/// Return a string representation of a cchar_t value using a specific buffer.
#[cfg(feature = "wide")]
pub fn tracecchar_t2(buffer: usize, cch: &CCharT) -> String {
    let mut state = get_trace_state().lock().unwrap();
    let buffer_idx = buffer.min(9);
    let result = tracecchar_t(cch);
    state.buffers[buffer_idx] = result.clone();
    result
}

/// Return a string representation of a cchar_t value using a specific buffer (C-style alias).
#[cfg(feature = "wide")]
#[inline]
pub fn _tracecchar_t2(buffer: usize, cch: &CCharT) -> String {
    tracecchar_t2(buffer, cch)
}

// ============================================================================
// Mouse tracing functions
// ============================================================================

/// Return a string representation of a mouse event.
#[cfg(feature = "mouse")]
pub fn tracemouse(event: &MouseEvent) -> String {
    format!(
        "MEVENT {{ id: {}, x: {}, y: {}, z: {}, bstate: 0x{:08X} }}",
        event.id, event.x, event.y, event.z, event.bstate
    )
}

/// Return a string representation of a mouse event (C-style alias).
#[cfg(feature = "mouse")]
#[inline]
pub fn _tracemouse(event: &MouseEvent) -> String {
    tracemouse(event)
}

// ============================================================================
// Miscellaneous trace functions
// ============================================================================

/// Return a string representation of TTY mode bits.
///
/// This function describes the current terminal mode settings.
pub fn nc_tracebits() -> String {
    // In our pure Rust implementation, we don't have direct access to
    // TTY mode bits like the C ncurses does. Return a placeholder.
    "TTY bits: (not available in pure Rust)".to_string()
}

/// Return a string representation of TTY mode bits (C-style alias).
#[inline]
pub fn _nc_tracebits() -> String {
    nc_tracebits()
}

/// Check if a specific trace feature is enabled.
///
/// # Arguments
///
/// * `flag` - The trace flag to check
///
/// # Returns
///
/// `true` if the flag is enabled
pub fn trace_enabled(flag: u32) -> bool {
    let state = get_trace_state().lock().unwrap();
    state.level != TRACE_DISABLE && (state.level & flag) != 0
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_levels() {
        assert_eq!(TRACE_DISABLE, 0);
        assert!(TRACE_MAXIMUM > TRACE_CALLS);
        assert!((TRACE_ORDINARY & TRACE_UPDATE) != 0);
    }

    #[test]
    fn test_traceattr() {
        use crate::attr;

        let result = traceattr(attr::A_BOLD);
        assert!(result.contains("A_BOLD"));

        let result = traceattr(attr::A_BOLD | attr::A_UNDERLINE);
        assert!(result.contains("A_BOLD"));
        assert!(result.contains("A_UNDERLINE"));

        let result = traceattr(0);
        assert_eq!(result, "A_NORMAL");
    }

    #[test]
    fn test_tracechar() {
        assert_eq!(tracechar(b'A' as i32), "'A'");
        assert_eq!(tracechar(0), "^@");
        assert_eq!(tracechar(127), "^?");
        assert_eq!(tracechar(-1), "ERR(-1)");
    }

    #[test]
    fn test_tracechtype() {
        let result = tracechtype(b'X' as ChType);
        assert!(result.contains("'X'"));
    }

    #[test]
    fn test_trace_state() {
        // Test that trace starts disabled
        let initial = curses_trace();
        assert_eq!(initial, TRACE_DISABLE);
    }
}
