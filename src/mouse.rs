//! Mouse support for ncurses-pure.
//!
//! This module provides mouse event handling functionality. Mouse support
//! must be enabled with the `mouse` feature flag.

use crate::types::MmaskT;
use std::collections::VecDeque;
use std::time::Instant;

// ============================================================================
// Mouse button masks
// ============================================================================

/// Mouse button 1 pressed.
pub const BUTTON1_PRESSED: MmaskT = 0x00000002;
/// Mouse button 1 released.
pub const BUTTON1_RELEASED: MmaskT = 0x00000001;
/// Mouse button 1 clicked.
pub const BUTTON1_CLICKED: MmaskT = 0x00000004;
/// Mouse button 1 double-clicked.
pub const BUTTON1_DOUBLE_CLICKED: MmaskT = 0x00000008;
/// Mouse button 1 triple-clicked.
pub const BUTTON1_TRIPLE_CLICKED: MmaskT = 0x00000010;

/// Mouse button 2 pressed.
pub const BUTTON2_PRESSED: MmaskT = 0x00000040;
/// Mouse button 2 released.
pub const BUTTON2_RELEASED: MmaskT = 0x00000020;
/// Mouse button 2 clicked.
pub const BUTTON2_CLICKED: MmaskT = 0x00000080;
/// Mouse button 2 double-clicked.
pub const BUTTON2_DOUBLE_CLICKED: MmaskT = 0x00000100;
/// Mouse button 2 triple-clicked.
pub const BUTTON2_TRIPLE_CLICKED: MmaskT = 0x00000200;

/// Mouse button 3 pressed.
pub const BUTTON3_PRESSED: MmaskT = 0x00000800;
/// Mouse button 3 released.
pub const BUTTON3_RELEASED: MmaskT = 0x00000400;
/// Mouse button 3 clicked.
pub const BUTTON3_CLICKED: MmaskT = 0x00001000;
/// Mouse button 3 double-clicked.
pub const BUTTON3_DOUBLE_CLICKED: MmaskT = 0x00002000;
/// Mouse button 3 triple-clicked.
pub const BUTTON3_TRIPLE_CLICKED: MmaskT = 0x00004000;

/// Mouse button 4 pressed (scroll up).
pub const BUTTON4_PRESSED: MmaskT = 0x00010000;
/// Mouse button 4 released.
pub const BUTTON4_RELEASED: MmaskT = 0x00008000;
/// Mouse button 4 clicked.
pub const BUTTON4_CLICKED: MmaskT = 0x00020000;
/// Mouse button 4 double-clicked.
pub const BUTTON4_DOUBLE_CLICKED: MmaskT = 0x00040000;
/// Mouse button 4 triple-clicked.
pub const BUTTON4_TRIPLE_CLICKED: MmaskT = 0x00080000;

/// Mouse button 5 pressed (scroll down).
pub const BUTTON5_PRESSED: MmaskT = 0x00200000;
/// Mouse button 5 released.
pub const BUTTON5_RELEASED: MmaskT = 0x00100000;
/// Mouse button 5 clicked.
pub const BUTTON5_CLICKED: MmaskT = 0x00400000;
/// Mouse button 5 double-clicked.
pub const BUTTON5_DOUBLE_CLICKED: MmaskT = 0x00800000;
/// Mouse button 5 triple-clicked.
pub const BUTTON5_TRIPLE_CLICKED: MmaskT = 0x01000000;

/// Shift was held during the mouse event.
pub const BUTTON_SHIFT: MmaskT = 0x04000000;
/// Ctrl was held during the mouse event.
pub const BUTTON_CTRL: MmaskT = 0x08000000;
/// Alt was held during the mouse event.
pub const BUTTON_ALT: MmaskT = 0x10000000;

/// Report mouse position changes.
pub const REPORT_MOUSE_POSITION: MmaskT = 0x20000000;

/// All mouse events.
pub const ALL_MOUSE_EVENTS: MmaskT = 0x1fffffff;

// ============================================================================
// Helper constants for button event lookup
// ============================================================================

/// Button pressed masks indexed by button number (0-4 for buttons 1-5).
const BUTTON_PRESSED: [MmaskT; 5] = [
    BUTTON1_PRESSED,
    BUTTON2_PRESSED,
    BUTTON3_PRESSED,
    BUTTON4_PRESSED,
    BUTTON5_PRESSED,
];

/// Button released masks indexed by button number.
const BUTTON_RELEASED: [MmaskT; 5] = [
    BUTTON1_RELEASED,
    BUTTON2_RELEASED,
    BUTTON3_RELEASED,
    BUTTON4_RELEASED,
    BUTTON5_RELEASED,
];

/// Button clicked masks indexed by button number.
const BUTTON_CLICKED: [MmaskT; 5] = [
    BUTTON1_CLICKED,
    BUTTON2_CLICKED,
    BUTTON3_CLICKED,
    BUTTON4_CLICKED,
    BUTTON5_CLICKED,
];

/// Button double-clicked masks indexed by button number.
const BUTTON_DOUBLE_CLICKED: [MmaskT; 5] = [
    BUTTON1_DOUBLE_CLICKED,
    BUTTON2_DOUBLE_CLICKED,
    BUTTON3_DOUBLE_CLICKED,
    BUTTON4_DOUBLE_CLICKED,
    BUTTON5_DOUBLE_CLICKED,
];

/// Button triple-clicked masks indexed by button number.
const BUTTON_TRIPLE_CLICKED: [MmaskT; 5] = [
    BUTTON1_TRIPLE_CLICKED,
    BUTTON2_TRIPLE_CLICKED,
    BUTTON3_TRIPLE_CLICKED,
    BUTTON4_TRIPLE_CLICKED,
    BUTTON5_TRIPLE_CLICKED,
];

// ============================================================================
// Mouse event structure
// ============================================================================

/// A mouse event.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MouseEvent {
    /// Event ID (for multiple mice).
    pub id: i16,
    /// X coordinate.
    pub x: i32,
    /// Y coordinate.
    pub y: i32,
    /// Z coordinate (for scroll).
    pub z: i32,
    /// Button state mask.
    pub bstate: MmaskT,
}

impl MouseEvent {
    /// Create a new mouse event.
    pub const fn new() -> Self {
        Self {
            id: 0,
            x: 0,
            y: 0,
            z: 0,
            bstate: 0,
        }
    }

    /// Check if a button event occurred.
    pub fn has_button(&self, mask: MmaskT) -> bool {
        (self.bstate & mask) != 0
    }
}

// ============================================================================
// Click tracking for double/triple click detection
// ============================================================================

/// Tracks click state for a single button.
#[derive(Clone, Debug, Default)]
struct ButtonClickState {
    /// Time of last click.
    last_click: Option<Instant>,
    /// Position of last click.
    last_pos: (i32, i32),
    /// Number of consecutive clicks (1, 2, or 3).
    click_count: u8,
}

/// Click tracker for detecting double and triple clicks.
#[derive(Clone, Debug, Default)]
pub struct ClickTracker {
    /// Click state for each button (5 buttons).
    buttons: [ButtonClickState; 5],
    /// Click interval in milliseconds.
    interval_ms: u64,
}

impl ClickTracker {
    /// Create a new click tracker with the given interval.
    pub fn new(interval_ms: i32) -> Self {
        Self {
            buttons: Default::default(),
            interval_ms: interval_ms.max(0) as u64,
        }
    }

    /// Set the click interval in milliseconds.
    pub fn set_interval(&mut self, interval_ms: i32) {
        self.interval_ms = interval_ms.max(0) as u64;
    }

    /// Record a button press and return the appropriate click mask.
    ///
    /// This tracks press/release pairs and timing to detect clicks,
    /// double-clicks, and triple-clicks.
    ///
    /// Returns the mask that should be added to the event's bstate.
    pub fn record_release(&mut self, button: usize, x: i32, y: i32) -> MmaskT {
        if button >= 5 {
            return 0;
        }

        let now = Instant::now();
        let state = &mut self.buttons[button];

        // Check if this is within the click interval and at the same position
        let is_consecutive = if let Some(last) = state.last_click {
            let elapsed = now.duration_since(last).as_millis() as u64;
            elapsed <= self.interval_ms && state.last_pos == (x, y)
        } else {
            false
        };

        // Update state
        state.last_click = Some(now);
        state.last_pos = (x, y);

        if is_consecutive {
            state.click_count = (state.click_count + 1).min(3);
        } else {
            state.click_count = 1;
        }

        // Return the appropriate click mask
        match state.click_count {
            1 => BUTTON_CLICKED[button],
            2 => BUTTON_DOUBLE_CLICKED[button],
            3 => {
                // Reset after triple click
                state.click_count = 0;
                BUTTON_TRIPLE_CLICKED[button]
            }
            _ => BUTTON_CLICKED[button],
        }
    }

    /// Reset click tracking for a button (e.g., on drag).
    pub fn reset(&mut self, button: usize) {
        if button < 5 {
            self.buttons[button] = ButtonClickState::default();
        }
    }

    /// Reset all buttons.
    pub fn reset_all(&mut self) {
        self.buttons = Default::default();
    }
}

// ============================================================================
// Mouse state management
// ============================================================================

/// Mouse state and configuration.
pub struct MouseState {
    /// Current mouse event mask.
    mask: MmaskT,
    /// Old mouse event mask.
    old_mask: MmaskT,
    /// Whether mouse is enabled.
    enabled: bool,
    /// Last mouse event.
    last_event: MouseEvent,
    /// Mouse event queue, oldest event first.
    event_queue: VecDeque<MouseEvent>,
    /// Click interval in milliseconds.
    click_interval: i32,
    /// Click tracker for double/triple click detection.
    click_tracker: ClickTracker,
    /// When each button that is currently down was pressed.
    buttons_pressed: [Option<Instant>; 5],
}

impl MouseState {
    /// Create a new mouse state.
    pub fn new() -> Self {
        let click_interval = 166; // Default click interval
        Self {
            mask: 0,
            old_mask: 0,
            enabled: false,
            last_event: MouseEvent::new(),
            event_queue: VecDeque::new(),
            click_interval,
            click_tracker: ClickTracker::new(click_interval),
            buttons_pressed: [None; 5],
        }
    }

    /// Enable mouse events with the given mask.
    pub fn mousemask(&mut self, newmask: MmaskT) -> MmaskT {
        self.old_mask = self.mask;
        self.mask = newmask;
        self.enabled = newmask != 0;
        self.old_mask
    }

    /// Get the current mouse mask.
    pub fn get_mask(&self) -> MmaskT {
        self.mask
    }

    /// Check if mouse is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get the next mouse event from the queue: the oldest one waiting.
    pub fn getmouse(&mut self) -> Option<MouseEvent> {
        self.event_queue.pop_front()
    }

    /// Add a mouse event to the queue, if the mouse mask asks for it, and
    /// say whether it was added.
    ///
    /// Only the parts of the event that the mask asks for are reported.
    pub fn push_event(&mut self, mut event: MouseEvent) -> bool {
        event.bstate &= self.mask;
        if event.bstate == 0 {
            return false;
        }
        self.last_event = event;
        self.event_queue.push_back(event);
        true
    }

    /// Process a raw mouse event and push it with click detection, and say
    /// whether it was added to the queue.
    ///
    /// A button released within the click interval of being pressed is a
    /// click, and the release event is marked as one, with double and
    /// triple clicks told apart. A click interval of 0 turns this off.
    pub fn process_event(&mut self, mut event: MouseEvent) -> bool {
        // Check for button press/release and track clicks
        for button in 0..5 {
            let pressed_mask = BUTTON_PRESSED[button];
            let released_mask = BUTTON_RELEASED[button];

            if event.has_button(pressed_mask) {
                self.buttons_pressed[button] = Some(Instant::now());
            }

            if event.has_button(released_mask)
                && let Some(pressed) = self.buttons_pressed[button].take()
                && self.click_interval > 0
                && pressed.elapsed().as_millis() <= self.click_interval as u128
            {
                // Record the release and get click type
                let click_mask = self.click_tracker.record_release(button, event.x, event.y);
                event.bstate |= click_mask;
            }
        }

        self.push_event(event)
    }

    /// Push an event back to the front of the queue, so that the next
    /// `getmouse` returns it.
    pub fn ungetmouse(&mut self, event: MouseEvent) -> bool {
        self.event_queue.push_front(event);
        true
    }

    /// Set the click interval.
    pub fn mouseinterval(&mut self, interval: i32) -> i32 {
        let old = self.click_interval;
        if interval >= 0 {
            self.click_interval = interval;
            self.click_tracker.set_interval(interval);
        }
        old
    }

    /// Check if there are pending mouse events.
    pub fn has_events(&self) -> bool {
        !self.event_queue.is_empty()
    }
}

impl Default for MouseState {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Terminal mouse protocol handling
// ============================================================================

/// Mouse protocol modes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MouseProtocol {
    /// No mouse support.
    #[default]
    None,
    /// X10 mouse protocol (basic, press only).
    X10,
    /// Normal tracking mode (press and release).
    Normal,
    /// Button event tracking (press, release, and motion with button).
    ButtonEvent,
    /// Any event tracking (all motion).
    AnyEvent,
    /// SGR extended mode (supports coordinates > 223).
    Sgr,
}

impl MouseProtocol {
    /// Get the escape sequence to enable this protocol.
    pub fn enable_sequence(&self) -> &'static str {
        match self {
            MouseProtocol::None => "",
            MouseProtocol::X10 => "\x1b[?9h",
            MouseProtocol::Normal => "\x1b[?1000h",
            MouseProtocol::ButtonEvent => "\x1b[?1002h",
            MouseProtocol::AnyEvent => "\x1b[?1003h",
            MouseProtocol::Sgr => "\x1b[?1006h",
        }
    }

    /// Get the escape sequence to disable this protocol.
    pub fn disable_sequence(&self) -> &'static str {
        match self {
            MouseProtocol::None => "",
            MouseProtocol::X10 => "\x1b[?9l",
            MouseProtocol::Normal => "\x1b[?1000l",
            MouseProtocol::ButtonEvent => "\x1b[?1002l",
            MouseProtocol::AnyEvent => "\x1b[?1003l",
            MouseProtocol::Sgr => "\x1b[?1006l",
        }
    }

    /// Detect the protocol from mouse event data.
    ///
    /// Returns the detected protocol and the minimum bytes needed.
    pub fn detect(data: &[u8]) -> Option<(MouseProtocol, usize)> {
        if data.len() < 3 {
            return None;
        }

        // SGR: \x1b[<...
        if data.len() >= 4 && &data[0..3] == b"\x1b[<" {
            return Some((MouseProtocol::Sgr, 9)); // Minimum SGR length
        }

        // X10/Normal/ButtonEvent: \x1b[M...
        if &data[0..3] == b"\x1b[M" {
            return Some((MouseProtocol::Normal, 6)); // 3 prefix + 3 data bytes
        }

        None
    }
}

/// Parse an SGR mouse event.
///
/// SGR format: `\x1b[<Cb;Cx;CyM` or `\x1b[<Cb;Cx;Cym`
/// where M = press, m = release
pub fn parse_sgr_mouse(data: &[u8]) -> Option<MouseEvent> {
    // Minimum: \x1b[<0;1;1M (9 bytes)
    if data.len() < 9 {
        return None;
    }

    // Check prefix
    if &data[0..3] != b"\x1b[<" {
        return None;
    }

    // Find the terminator
    let term_pos = data.iter().position(|&b| b == b'M' || b == b'm')?;
    let is_release = data[term_pos] == b'm';

    // Parse the parameters
    let params_str = std::str::from_utf8(&data[3..term_pos]).ok()?;
    let parts: Vec<&str> = params_str.split(';').collect();
    if parts.len() != 3 {
        return None;
    }

    let cb: u32 = parts[0].parse().ok()?;
    let cx: u32 = parts[1].parse().ok()?;
    let cy: u32 = parts[2].parse().ok()?;

    // Convert to 0-based coordinates. They are 1-based on the wire: a
    // terminal never sends 0, and nothing here may trust it not to.
    let x = i32::try_from(cx.checked_sub(1)?).ok()?;
    let y = i32::try_from(cy.checked_sub(1)?).ok()?;

    // Decode button
    let button = cb & 0x03;
    let shift = (cb & 0x04) != 0;
    let meta = (cb & 0x08) != 0;
    let ctrl = (cb & 0x10) != 0;
    let motion = (cb & 0x20) != 0;
    let scroll = (cb & 0x40) != 0;

    let mut bstate: MmaskT = 0;

    if scroll {
        // Scroll events
        if button == 0 {
            bstate |= BUTTON4_PRESSED; // Scroll up
        } else if button == 1 {
            bstate |= BUTTON5_PRESSED; // Scroll down
        }
    } else if motion {
        bstate |= REPORT_MOUSE_POSITION;
    } else {
        // Button events
        let (pressed, released) = match button {
            0 => (BUTTON1_PRESSED, BUTTON1_RELEASED),
            1 => (BUTTON2_PRESSED, BUTTON2_RELEASED),
            2 => (BUTTON3_PRESSED, BUTTON3_RELEASED),
            _ => (0, 0),
        };
        bstate |= if is_release { released } else { pressed };
    }

    // Add modifiers
    if shift {
        bstate |= BUTTON_SHIFT;
    }
    if meta {
        bstate |= BUTTON_ALT;
    }
    if ctrl {
        bstate |= BUTTON_CTRL;
    }

    Some(MouseEvent {
        id: 0,
        x,
        y,
        z: 0,
        bstate,
    })
}

/// Parse an X10 mouse event.
///
/// X10 format: `\x1b[MCbCxCy` (6 bytes total)
/// - Cb = button + 32
/// - Cx = x + 33 (1-based, offset by 32)
/// - Cy = y + 33 (1-based, offset by 32)
///
/// X10 only reports button presses, not releases.
pub fn parse_x10_mouse(data: &[u8]) -> Option<MouseEvent> {
    // X10: \x1b[M followed by 3 bytes
    if data.len() < 6 {
        return None;
    }

    // Check prefix
    if &data[0..3] != b"\x1b[M" {
        return None;
    }

    let cb = data[3];
    let cx = data[4];
    let cy = data[5];

    // Coordinates are offset by 33 (32 + 1 for 1-based)
    let x = (cx as i32).saturating_sub(33);
    let y = (cy as i32).saturating_sub(33);

    // Button is offset by 32
    let button_code = cb.saturating_sub(32);
    let button = button_code & 0x03;

    let mut bstate: MmaskT = 0;

    // X10 only reports presses
    match button {
        0 => bstate |= BUTTON1_PRESSED,
        1 => bstate |= BUTTON2_PRESSED,
        2 => bstate |= BUTTON3_PRESSED,
        _ => {}
    }

    Some(MouseEvent {
        id: 0,
        x,
        y,
        z: 0,
        bstate,
    })
}

/// Parse a Normal/Button-event mode mouse event.
///
/// Normal format: `\x1b[MCbCxCy` (6 bytes total)
/// - Cb = button + modifiers + 32
/// - Cx = x + 33
/// - Cy = y + 33
///
/// This is similar to X10 but includes:
/// - Release events (button = 3)
/// - Modifier keys (shift, meta, ctrl)
/// - Motion events (bit 5 set)
pub fn parse_normal_mouse(data: &[u8]) -> Option<MouseEvent> {
    // Normal: \x1b[M followed by 3 bytes
    if data.len() < 6 {
        return None;
    }

    // Check prefix
    if &data[0..3] != b"\x1b[M" {
        return None;
    }

    let cb = data[3];
    let cx = data[4];
    let cy = data[5];

    // Coordinates are offset by 33 (32 + 1 for 1-based)
    // Note: coordinates > 223 cannot be represented in this protocol
    let x = (cx as i32).saturating_sub(33);
    let y = (cy as i32).saturating_sub(33);

    // Decode the button byte
    let button_code = cb.saturating_sub(32);
    let button = button_code & 0x03;
    let shift = (button_code & 0x04) != 0;
    let meta = (button_code & 0x08) != 0;
    let ctrl = (button_code & 0x10) != 0;
    let motion = (button_code & 0x20) != 0;
    let scroll = (button_code & 0x40) != 0;

    let mut bstate: MmaskT = 0;

    if scroll {
        // Scroll events (button 4/5)
        if button == 0 {
            bstate |= BUTTON4_PRESSED; // Scroll up
        } else if button == 1 {
            bstate |= BUTTON5_PRESSED; // Scroll down
        }
    } else if motion {
        // Motion event
        bstate |= REPORT_MOUSE_POSITION;
        // If a button is held during motion, add its pressed state
        if button < 3 {
            bstate |= BUTTON_PRESSED[button as usize];
        }
    } else if button == 3 {
        // Release event (no way to know which button was released)
        // We'll report all buttons as potentially released
        // The caller should track which button was pressed
        bstate |= BUTTON1_RELEASED;
    } else {
        // Button press
        match button {
            0 => bstate |= BUTTON1_PRESSED,
            1 => bstate |= BUTTON2_PRESSED,
            2 => bstate |= BUTTON3_PRESSED,
            _ => {}
        }
    }

    // Add modifiers
    if shift {
        bstate |= BUTTON_SHIFT;
    }
    if meta {
        bstate |= BUTTON_ALT;
    }
    if ctrl {
        bstate |= BUTTON_CTRL;
    }

    Some(MouseEvent {
        id: 0,
        x,
        y,
        z: 0,
        bstate,
    })
}

/// Parse a mouse event, auto-detecting the protocol.
///
/// Tries SGR first (most capable), then falls back to Normal/X10.
pub fn parse_mouse_event(data: &[u8]) -> Option<MouseEvent> {
    // Try SGR first
    if data.len() >= 4 && &data[0..3] == b"\x1b[<" {
        return parse_sgr_mouse(data);
    }

    // Try Normal/X10
    if data.len() >= 6 && &data[0..3] == b"\x1b[M" {
        return parse_normal_mouse(data);
    }

    None
}

/// Check if the given input might be the start of a mouse sequence.
pub fn is_mouse_prefix(data: &[u8]) -> bool {
    if data.is_empty() {
        return false;
    }
    // SGR mouse starts with \x1b[<
    if data.len() >= 3 && &data[0..3] == b"\x1b[<" {
        return true;
    }
    // Normal/X10 mouse starts with \x1b[M
    if data.len() >= 3 && &data[0..3] == b"\x1b[M" {
        return true;
    }
    // Partial prefix
    if data.len() == 2 && &data[0..2] == b"\x1b[" {
        return true;
    }
    if data.len() == 1 && data[0] == 0x1b {
        return true;
    }
    false
}

/// Get the length of a complete mouse sequence, or None if incomplete.
pub fn mouse_sequence_length(data: &[u8]) -> Option<usize> {
    if data.len() < 3 {
        return None;
    }

    // SGR: find terminator M or m
    if data.len() >= 4 && &data[0..3] == b"\x1b[<" {
        for (i, &b) in data.iter().enumerate().skip(3) {
            if b == b'M' || b == b'm' {
                return Some(i + 1);
            }
        }
        return None; // Incomplete
    }

    // Normal/X10: fixed 6 bytes
    if &data[0..3] == b"\x1b[M" {
        if data.len() >= 6 {
            return Some(6);
        }
        return None; // Incomplete
    }

    None
}

/// Convenience function to check if a position is within screen bounds.
pub fn wenclose(win_y: i32, win_x: i32, win_h: i32, win_w: i32, y: i32, x: i32) -> bool {
    y >= win_y && y < win_y + win_h && x >= win_x && x < win_x + win_w
}

/// Convert screen coordinates to window coordinates.
pub fn wmouse_trafo(
    win_y: i32,
    win_x: i32,
    win_h: i32,
    win_w: i32,
    y: &mut i32,
    x: &mut i32,
    to_screen: bool,
) -> bool {
    if to_screen {
        // Window to screen
        *y += win_y;
        *x += win_x;
        true
    } else {
        // Screen to window
        if wenclose(win_y, win_x, win_h, win_w, *y, *x) {
            *y -= win_y;
            *x -= win_x;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mouse_event() {
        let mut event = MouseEvent::new();
        event.bstate = BUTTON1_PRESSED | BUTTON_SHIFT;
        assert!(event.has_button(BUTTON1_PRESSED));
        assert!(event.has_button(BUTTON_SHIFT));
        assert!(!event.has_button(BUTTON2_PRESSED));
    }

    #[test]
    fn test_mouse_state() {
        let mut state = MouseState::new();
        assert!(!state.is_enabled());

        state.mousemask(ALL_MOUSE_EVENTS);
        assert!(state.is_enabled());

        let event = MouseEvent {
            id: 0,
            x: 10,
            y: 20,
            z: 0,
            bstate: BUTTON1_CLICKED,
        };
        state.push_event(event);
        assert!(state.has_events());

        let got = state.getmouse().unwrap();
        assert_eq!(got.x, 10);
        assert_eq!(got.y, 20);
    }

    fn event(x: i32, bstate: MmaskT) -> MouseEvent {
        MouseEvent {
            id: 0,
            x,
            y: 0,
            z: 0,
            bstate,
        }
    }

    #[test]
    fn test_mouse_events_come_out_in_the_order_they_went_in() {
        let mut state = MouseState::new();
        state.mousemask(ALL_MOUSE_EVENTS);
        assert!(state.push_event(event(1, BUTTON1_PRESSED)));
        assert!(state.push_event(event(2, BUTTON1_RELEASED)));
        assert!(state.push_event(event(3, BUTTON2_PRESSED)));
        let order: Vec<i32> = std::iter::from_fn(|| state.getmouse())
            .map(|e| e.x)
            .collect();
        assert_eq!(order, [1, 2, 3]);
        assert!(!state.has_events());
    }

    #[test]
    fn test_ungetmouse_event_is_the_next_one_returned() {
        let mut state = MouseState::new();
        state.mousemask(ALL_MOUSE_EVENTS);
        state.push_event(event(1, BUTTON1_PRESSED));
        state.push_event(event(2, BUTTON1_RELEASED));
        let first = state.getmouse().unwrap();
        assert!(state.ungetmouse(first));
        assert_eq!(state.getmouse().unwrap().x, 1);
        assert_eq!(state.getmouse().unwrap().x, 2);
    }

    #[test]
    fn test_mouse_mask_filters_events_and_what_they_report() {
        let mut state = MouseState::new();
        state.mousemask(BUTTON1_CLICKED | BUTTON2_PRESSED);
        assert!(!state.push_event(event(1, BUTTON1_PRESSED)));
        assert!(!state.has_events());
        assert!(state.push_event(event(2, BUTTON1_RELEASED | BUTTON1_CLICKED)));
        assert_eq!(state.getmouse().unwrap().bstate, BUTTON1_CLICKED);

        // With no mask nothing is queued.
        state.mousemask(0);
        assert!(!state.push_event(event(3, BUTTON1_CLICKED)));
    }

    #[test]
    fn test_process_event_marks_a_quick_release_as_a_click() {
        let mut state = MouseState::new();
        state.mousemask(ALL_MOUSE_EVENTS);
        state.mouseinterval(10_000);
        assert!(state.process_event(event(1, BUTTON1_PRESSED)));
        assert!(state.process_event(event(1, BUTTON1_RELEASED)));
        assert_eq!(state.getmouse().unwrap().bstate, BUTTON1_PRESSED);
        assert_eq!(
            state.getmouse().unwrap().bstate,
            BUTTON1_RELEASED | BUTTON1_CLICKED
        );

        // A second click in the same place is a double click.
        state.process_event(event(1, BUTTON1_PRESSED));
        state.process_event(event(1, BUTTON1_RELEASED));
        state.getmouse();
        assert_eq!(
            state.getmouse().unwrap().bstate,
            BUTTON1_RELEASED | BUTTON1_DOUBLE_CLICKED
        );
    }

    #[test]
    fn test_process_event_without_a_click() {
        // A release with no press before it is not a click.
        let mut state = MouseState::new();
        state.mousemask(ALL_MOUSE_EVENTS);
        state.mouseinterval(10_000);
        state.process_event(event(1, BUTTON1_RELEASED));
        assert_eq!(state.getmouse().unwrap().bstate, BUTTON1_RELEASED);

        // A click interval of 0 turns click detection off.
        state.mouseinterval(0);
        state.process_event(event(1, BUTTON1_PRESSED));
        state.process_event(event(1, BUTTON1_RELEASED));
        state.getmouse();
        assert_eq!(state.getmouse().unwrap().bstate, BUTTON1_RELEASED);

        // A button held longer than the interval is not a click.
        state.mouseinterval(1);
        state.process_event(event(1, BUTTON1_PRESSED));
        std::thread::sleep(std::time::Duration::from_millis(20));
        state.process_event(event(1, BUTTON1_RELEASED));
        state.getmouse();
        assert_eq!(state.getmouse().unwrap().bstate, BUTTON1_RELEASED);
    }

    #[test]
    fn test_parse_sgr_mouse_rejects_positions_out_of_range() {
        for report in [
            &b"\x1b[<0;-2147483648;1M"[..],
            b"\x1b[<0;1;-2147483648M",
            b"\x1b[<0;-1;1M",
            b"\x1b[<0;0;1M",
            b"\x1b[<0;1;0M",
            b"\x1b[<0;4294967295;1M",
            b"\x1b[<0;99999999999;1M",
            b"\x1b[<-1;1;1M",
        ] {
            assert!(parse_sgr_mouse(report).is_none(), "{report:?}");
        }

        let far = parse_sgr_mouse(b"\x1b[<0;2147483647;1M").unwrap();
        assert_eq!((far.x, far.y), (i32::MAX - 1, 0));
    }

    #[test]
    fn test_parse_sgr_mouse() {
        // Button 1 press at (1,1)
        let event = parse_sgr_mouse(b"\x1b[<0;1;1M").unwrap();
        assert_eq!(event.x, 0);
        assert_eq!(event.y, 0);
        assert!(event.has_button(BUTTON1_PRESSED));

        // Button 1 release at (10,20)
        let event = parse_sgr_mouse(b"\x1b[<0;10;20m").unwrap();
        assert_eq!(event.x, 9);
        assert_eq!(event.y, 19);
        assert!(event.has_button(BUTTON1_RELEASED));

        // Scroll up
        let event = parse_sgr_mouse(b"\x1b[<64;5;5M").unwrap();
        assert!(event.has_button(BUTTON4_PRESSED));

        // With modifiers
        let event = parse_sgr_mouse(b"\x1b[<4;5;5M").unwrap();
        assert!(event.has_button(BUTTON_SHIFT));
    }

    #[test]
    fn test_parse_x10_mouse() {
        // Button 1 press at (0,0) - bytes are 32+0, 33+0, 33+0
        let event = parse_x10_mouse(b"\x1b[M !!").expect("should parse X10 mouse event");
        assert_eq!(event.x, 0);
        assert_eq!(event.y, 0);
        assert!(event.has_button(BUTTON1_PRESSED));

        // Button 2 press
        let event = parse_x10_mouse(b"\x1b[M!!!").expect("should parse X10 mouse event");
        assert!(event.has_button(BUTTON2_PRESSED));
    }

    #[test]
    fn test_parse_normal_mouse() {
        // Button 1 press at (0,0)
        let event = parse_normal_mouse(b"\x1b[M !!").expect("should parse normal mouse event");
        assert_eq!(event.x, 0);
        assert_eq!(event.y, 0);
        assert!(event.has_button(BUTTON1_PRESSED));

        // Button release (button code 3)
        let event = parse_normal_mouse(b"\x1b[M#!!").expect("should parse normal mouse release");
        assert!(event.has_button(BUTTON1_RELEASED));

        // With shift (button code = 0 + 4 = 4, plus 32 = 36 = '$')
        let event = parse_normal_mouse(b"\x1b[M$!!").expect("should parse shifted mouse event");
        assert!(event.has_button(BUTTON_SHIFT));
    }

    #[test]
    fn test_parse_mouse_event_auto() {
        // SGR
        let event = parse_mouse_event(b"\x1b[<0;1;1M").unwrap();
        assert!(event.has_button(BUTTON1_PRESSED));

        // Normal
        let event = parse_mouse_event(b"\x1b[M !!").unwrap();
        assert!(event.has_button(BUTTON1_PRESSED));
    }

    #[test]
    fn test_mouse_sequence_length() {
        assert_eq!(mouse_sequence_length(b"\x1b[<0;1;1M"), Some(9));
        assert_eq!(mouse_sequence_length(b"\x1b[<0;10;20m"), Some(11));
        assert_eq!(mouse_sequence_length(b"\x1b[M !!"), Some(6));
        assert_eq!(mouse_sequence_length(b"\x1b[<0;1"), None); // Incomplete
    }

    #[test]
    fn test_click_tracker() {
        let mut tracker = ClickTracker::new(500); // 500ms interval

        // First click
        let mask = tracker.record_release(0, 10, 20);
        assert_eq!(mask, BUTTON1_CLICKED);

        // Second click at same position (within interval)
        let mask = tracker.record_release(0, 10, 20);
        assert_eq!(mask, BUTTON1_DOUBLE_CLICKED);

        // Third click
        let mask = tracker.record_release(0, 10, 20);
        assert_eq!(mask, BUTTON1_TRIPLE_CLICKED);

        // Fourth click resets to single
        let mask = tracker.record_release(0, 10, 20);
        assert_eq!(mask, BUTTON1_CLICKED);
    }

    #[test]
    fn test_click_tracker_different_position() {
        let mut tracker = ClickTracker::new(500);

        // First click
        tracker.record_release(0, 10, 20);

        // Second click at different position - should be single click
        let mask = tracker.record_release(0, 15, 25);
        assert_eq!(mask, BUTTON1_CLICKED);
    }

    #[test]
    fn test_wenclose() {
        assert!(wenclose(0, 0, 10, 20, 5, 10));
        assert!(!wenclose(0, 0, 10, 20, 15, 10));
    }

    #[test]
    fn test_protocol_detect() {
        let (proto, _) = MouseProtocol::detect(b"\x1b[<0;1;1M").unwrap();
        assert_eq!(proto, MouseProtocol::Sgr);

        let (proto, _) = MouseProtocol::detect(b"\x1b[M !!").unwrap();
        assert_eq!(proto, MouseProtocol::Normal);

        assert!(MouseProtocol::detect(b"\x1b").is_none());
    }
}
