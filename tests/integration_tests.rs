//! Integration tests for ncurses-pure
//!
//! These tests verify the behavior of windows, attributes, and screen operations
//! by examining the internal state rather than actual terminal output.

use ncurses::*;

/// Test window creation and basic properties
#[test]
fn test_window_dimensions() {
    let win = Window::new(20, 40, 0, 0).unwrap();
    assert_eq!(win.getmaxy(), 20); // nlines
    assert_eq!(win.getmaxx(), 40); // ncols
    assert_eq!(win.getbegy(), 0);
    assert_eq!(win.getbegx(), 0);
}

/// Test window with position
#[test]
fn test_window_with_position() {
    let win = Window::new(10, 30, 5, 10).unwrap();
    assert_eq!(win.getmaxy(), 10);
    assert_eq!(win.getmaxx(), 30);
    assert_eq!(win.getbegy(), 5);
    assert_eq!(win.getbegx(), 10);
}

/// Test cursor movement
#[test]
fn test_cursor_movement() {
    let mut win = Window::new(20, 40, 0, 0).unwrap();

    // Initial position
    assert_eq!(win.getcury(), 0);
    assert_eq!(win.getcurx(), 0);

    // Move cursor
    win.mv(5, 10).unwrap();
    assert_eq!(win.getcury(), 5);
    assert_eq!(win.getcurx(), 10);

    // Move to origin
    win.mv(0, 0).unwrap();
    assert_eq!(win.getcury(), 0);
    assert_eq!(win.getcurx(), 0);
}

/// Test cursor bounds checking
#[test]
fn test_cursor_bounds() {
    let mut win = Window::new(10, 20, 0, 0).unwrap();

    // Valid moves
    assert!(win.mv(0, 0).is_ok());
    assert!(win.mv(9, 19).is_ok());

    // Invalid moves (out of bounds)
    assert!(win.mv(10, 0).is_err());
    assert!(win.mv(0, 20).is_err());
    assert!(win.mv(-1, 0).is_err());
    assert!(win.mv(0, -1).is_err());
}

/// Test character output advances cursor
#[test]
fn test_addch_cursor_advance() {
    let mut win = Window::new(10, 20, 0, 0).unwrap();

    win.addch(b'A' as ChType).unwrap();
    assert_eq!(win.getcurx(), 1);

    win.addch(b'B' as ChType).unwrap();
    assert_eq!(win.getcurx(), 2);
}

/// Test string output
#[test]
fn test_addstr() {
    let mut win = Window::new(10, 20, 0, 0).unwrap();

    win.addstr("Hello").unwrap();
    assert_eq!(win.getcurx(), 5);

    win.mv(1, 0).unwrap();
    win.addstr("World").unwrap();
    assert_eq!(win.getcury(), 1);
    assert_eq!(win.getcurx(), 5);
}

/// Test mvaddstr convenience method
#[test]
fn test_mvaddstr() {
    let mut win = Window::new(10, 40, 0, 0).unwrap();

    win.mvaddstr(3, 5, "Test").unwrap();
    assert_eq!(win.getcury(), 3);
    assert_eq!(win.getcurx(), 9); // 5 + 4 chars
}

/// Test mvaddch convenience method
#[test]
fn test_mvaddch() {
    let mut win = Window::new(10, 40, 0, 0).unwrap();

    win.mvaddch(2, 3, b'X' as ChType).unwrap();
    assert_eq!(win.getcury(), 2);
    assert_eq!(win.getcurx(), 4);
}

/// Test attribute operations
#[test]
fn test_attributes() {
    let mut win = Window::new(10, 20, 0, 0).unwrap();

    // Initially normal
    assert_eq!(win.getattrs(), attr::A_NORMAL);

    // Turn on bold
    win.attron(attr::A_BOLD).unwrap();
    assert!(win.getattrs() & attr::A_BOLD != 0);

    // Turn off bold
    win.attroff(attr::A_BOLD).unwrap();
    assert!(win.getattrs() & attr::A_BOLD == 0);

    // Set multiple attributes
    win.attrset(attr::A_BOLD | attr::A_UNDERLINE).unwrap();
    assert!(win.getattrs() & attr::A_BOLD != 0);
    assert!(win.getattrs() & attr::A_UNDERLINE != 0);
}

/// Test color pair macro
#[test]
fn test_color_pair() {
    let pair1 = attr::color_pair(1);
    let pair2 = attr::color_pair(2);

    assert_ne!(pair1, pair2);
    assert_ne!(pair1, 0);
}

/// Test erase operations
#[test]
fn test_erase() {
    let mut win = Window::new(10, 20, 0, 0).unwrap();

    win.addstr("Some text").unwrap();
    win.mv(5, 5).unwrap();

    win.erase().unwrap();

    // Cursor should be at origin after erase
    assert_eq!(win.getcury(), 0);
    assert_eq!(win.getcurx(), 0);
}

/// Test clear operations
#[test]
fn test_clear() {
    let mut win = Window::new(10, 20, 0, 0).unwrap();

    win.addstr("Some text").unwrap();
    win.mv(5, 5).unwrap();

    win.clear().unwrap();

    // Cursor should be at origin after clear
    assert_eq!(win.getcury(), 0);
    assert_eq!(win.getcurx(), 0);
}

/// Test clrtoeol
#[test]
fn test_clrtoeol() {
    let mut win = Window::new(10, 20, 0, 0).unwrap();

    win.mv(3, 5).unwrap();
    win.clrtoeol().unwrap();

    // Cursor position unchanged
    assert_eq!(win.getcury(), 3);
    assert_eq!(win.getcurx(), 5);
}

/// Test clrtobot
#[test]
fn test_clrtobot() {
    let mut win = Window::new(10, 20, 0, 0).unwrap();

    win.mv(3, 5).unwrap();
    win.clrtobot().unwrap();

    // Cursor position unchanged
    assert_eq!(win.getcury(), 3);
    assert_eq!(win.getcurx(), 5);
}

/// Test box drawing
#[test]
fn test_box() {
    let mut win = Window::new(5, 10, 0, 0).unwrap();
    win.box_(0, 0).unwrap();

    // Box should not move cursor
    assert_eq!(win.getcury(), 0);
    assert_eq!(win.getcurx(), 0);
}

/// Test border drawing
#[test]
fn test_border() {
    let mut win = Window::new(5, 10, 0, 0).unwrap();
    win.border(0, 0, 0, 0, 0, 0, 0, 0).unwrap();

    // Border should not move cursor
    assert_eq!(win.getcury(), 0);
    assert_eq!(win.getcurx(), 0);
}

/// Test horizontal line
#[test]
fn test_hline() {
    let mut win = Window::new(10, 20, 0, 0).unwrap();
    win.mv(5, 2).unwrap();
    win.hline(0, 10).unwrap();

    // hline should not move cursor
    assert_eq!(win.getcury(), 5);
    assert_eq!(win.getcurx(), 2);
}

/// Test vertical line
#[test]
fn test_vline() {
    let mut win = Window::new(10, 20, 0, 0).unwrap();
    win.mv(2, 5).unwrap();
    win.vline(0, 5).unwrap();

    // vline should not move cursor
    assert_eq!(win.getcury(), 2);
    assert_eq!(win.getcurx(), 5);
}

/// Test newline handling
#[test]
fn test_newline() {
    let mut win = Window::new(10, 20, 0, 0).unwrap();

    win.addstr("Line 1").unwrap();
    win.addch(b'\n' as ChType).unwrap();

    assert_eq!(win.getcury(), 1);
    assert_eq!(win.getcurx(), 0);
}

/// Test tab handling
#[test]
fn test_tab() {
    let mut win = Window::new(10, 40, 0, 0).unwrap();

    win.addch(b'\t' as ChType).unwrap();
    assert_eq!(win.getcurx(), 8); // Tab stop at column 8

    win.addch(b'A' as ChType).unwrap();
    win.addch(b'\t' as ChType).unwrap();
    assert_eq!(win.getcurx(), 16); // Next tab stop
}

/// Test scrolling flag
#[test]
fn test_scroll_flag() {
    let mut win = Window::new(10, 20, 0, 0).unwrap();

    // Initially scrolling is off
    win.scrollok(true);
    // No direct getter, but scrollok should work without error

    win.scrollok(false);
}

/// Test keypad flag
#[test]
fn test_keypad_flag() {
    let mut win = Window::new(10, 20, 0, 0).unwrap();

    win.keypad(true);
    win.keypad(false);
    // No assertion needed, just checking it doesn't panic
}

/// Test leaveok flag
#[test]
fn test_leaveok() {
    let mut win = Window::new(10, 20, 0, 0).unwrap();

    win.leaveok(true);
    win.leaveok(false);
}

/// Test nodelay flag
#[test]
fn test_nodelay() {
    let mut win = Window::new(10, 20, 0, 0).unwrap();

    win.nodelay(true);
    win.nodelay(false);
}

/// Test touchwin and untouchwin
#[test]
fn test_touch() {
    let mut win = Window::new(10, 20, 0, 0).unwrap();

    win.touchwin();
    win.untouchwin();
}

/// Test touchln
#[test]
fn test_touchln() {
    let mut win = Window::new(10, 20, 0, 0).unwrap();

    win.touchln(2, 5, true);
    // Should not panic
}

/// Test background character
#[test]
fn test_bkgd() {
    let mut win = Window::new(10, 20, 0, 0).unwrap();

    win.bkgd(b' ' as ChType | attr::A_REVERSE).unwrap();

    // Background should be set
    assert_ne!(win.getbkgd(), b' ' as ChType);
}

/// Test bkgdset
#[test]
fn test_bkgdset() {
    let mut win = Window::new(10, 20, 0, 0).unwrap();

    win.bkgdset(b'.' as ChType);
    assert_eq!(win.getbkgd(), b'.' as ChType);
}

/// Test inch - get character at current position
#[test]
fn test_inch() {
    let mut win = Window::new(10, 20, 0, 0).unwrap();

    win.addch(b'X' as ChType).unwrap();
    win.mv(0, 0).unwrap();

    let ch = win.inch();
    assert_eq!((ch & A_CHARTEXT) as u8, b'X');
}

/// Test instr - get string
#[test]
fn test_instr() {
    let mut win = Window::new(10, 40, 0, 0).unwrap();

    win.addstr("Hello World").unwrap();
    win.mv(0, 0).unwrap();

    let s = win.instr(5);
    assert_eq!(s, "Hello");
}

/// Test key constants are defined
#[test]
fn test_key_constants() {
    assert!(key::KEY_UP > 0x100);
    assert!(key::KEY_DOWN > 0x100);
    assert!(key::KEY_LEFT > 0x100);
    assert!(key::KEY_RIGHT > 0x100);
    assert!(key::KEY_HOME > 0x100);
    assert!(key::KEY_END > 0x100);
    assert!(key::KEY_BACKSPACE > 0);
    assert!(key::KEY_DC > 0);
}

/// Test KEY_F function
#[test]
fn test_key_f() {
    let f1 = key::key_f(1);
    let f2 = key::key_f(2);
    let f12 = key::key_f(12);

    assert!(f1 >= key::KEY_F0);
    assert_eq!(f2, f1 + 1);
    assert!(f12 > f1);
}

/// Test attribute constants are defined
#[test]
fn test_attr_constants() {
    assert_ne!(attr::A_BOLD, 0);
    assert_ne!(attr::A_UNDERLINE, 0);
    assert_ne!(attr::A_REVERSE, 0);
    assert_ne!(attr::A_BLINK, 0);
    assert_ne!(attr::A_DIM, 0);
    assert_ne!(attr::A_STANDOUT, 0);

    // Attributes should be distinct
    assert_ne!(attr::A_BOLD, attr::A_UNDERLINE);
    assert_ne!(attr::A_BOLD, attr::A_REVERSE);
}

/// Test color constants
#[test]
fn test_color_constants() {
    assert_eq!(COLOR_BLACK, 0);
    assert_eq!(COLOR_RED, 1);
    assert_eq!(COLOR_GREEN, 2);
    assert_eq!(COLOR_YELLOW, 3);
    assert_eq!(COLOR_BLUE, 4);
    assert_eq!(COLOR_MAGENTA, 5);
    assert_eq!(COLOR_CYAN, 6);
    assert_eq!(COLOR_WHITE, 7);
}

/// Test pad creation
#[test]
fn test_pad_creation() {
    let pad = Window::new_pad(100, 200).unwrap();
    assert_eq!(pad.getmaxy(), 100);
    assert_eq!(pad.getmaxx(), 200);
}

/// Test delwin (drop behavior)
#[test]
fn test_window_drop() {
    {
        let _win = Window::new(10, 20, 0, 0).unwrap();
        // Window should be dropped at end of scope without panic
    }
}

/// Test insertln
#[test]
fn test_insertln() {
    let mut win = Window::new(10, 20, 0, 0).unwrap();
    win.mv(3, 0).unwrap();
    win.insertln().unwrap();
    // Should not panic, cursor stays at same position
    assert_eq!(win.getcury(), 3);
}

/// Test deleteln
#[test]
fn test_deleteln() {
    let mut win = Window::new(10, 20, 0, 0).unwrap();
    win.mv(3, 0).unwrap();
    win.deleteln().unwrap();
    // Should not panic, cursor stays at same position
    assert_eq!(win.getcury(), 3);
}

/// Test insdelln
#[test]
fn test_insdelln() {
    let mut win = Window::new(10, 20, 0, 0).unwrap();
    win.mv(3, 0).unwrap();

    // Insert 2 lines
    win.insdelln(2).unwrap();
    assert_eq!(win.getcury(), 3);

    // Delete 1 line
    win.insdelln(-1).unwrap();
    assert_eq!(win.getcury(), 3);
}

/// Test scrl (scroll by n)
#[test]
fn test_scrl() {
    let mut win = Window::new(10, 20, 0, 0).unwrap();
    win.scrollok(true);
    win.scrl(2).unwrap();
    win.scrl(-1).unwrap();
    // Should not panic
}

/// Test setscrreg
#[test]
fn test_setscrreg() {
    let mut win = Window::new(10, 20, 0, 0).unwrap();
    win.setscrreg(2, 7).unwrap();
    // Should set scroll region without panic
}

/// Test insch
#[test]
fn test_insch() {
    let mut win = Window::new(10, 20, 0, 0).unwrap();
    win.addstr("Hello").unwrap();
    win.mv(0, 0).unwrap();
    win.insch(b'X' as ChType).unwrap();

    // Character should be inserted, cursor at same position
    assert_eq!(win.getcurx(), 0);
}

/// Test delch
#[test]
fn test_delch() {
    let mut win = Window::new(10, 20, 0, 0).unwrap();
    win.addstr("Hello").unwrap();
    win.mv(0, 0).unwrap();
    win.delch().unwrap();

    // Character deleted, cursor at same position
    assert_eq!(win.getcurx(), 0);
}

/// Test immedok
#[test]
fn test_immedok() {
    let mut win = Window::new(10, 20, 0, 0).unwrap();
    win.immedok(true);
    win.immedok(false);
    // Should not panic
}

/// Test syncok
#[test]
fn test_syncok() {
    let mut win = Window::new(10, 20, 0, 0).unwrap();
    win.syncok(true);
    win.syncok(false);
    // Should not panic
}

/// Test clearok
#[test]
fn test_clearok() {
    let mut win = Window::new(10, 20, 0, 0).unwrap();
    win.clearok(true);
    win.clearok(false);
    // Should not panic
}

/// Test idlok
#[test]
fn test_idlok() {
    let mut win = Window::new(10, 20, 0, 0).unwrap();
    win.idlok(true);
    win.idlok(false);
    // Should not panic
}

/// Test idcok
#[test]
fn test_idcok() {
    let mut win = Window::new(10, 20, 0, 0).unwrap();
    win.idcok(true);
    win.idcok(false);
    // Should not panic
}

/// Test addnstr - add string with max length
#[test]
fn test_addnstr() {
    let mut win = Window::new(10, 40, 0, 0).unwrap();

    win.addnstr("Hello World", 5).unwrap();
    assert_eq!(win.getcurx(), 5); // Only "Hello" was added
}

/// Test mvaddnstr
#[test]
fn test_mvaddnstr() {
    let mut win = Window::new(10, 40, 0, 0).unwrap();

    win.mvaddnstr(2, 3, "Hello World", 5).unwrap();
    assert_eq!(win.getcury(), 2);
    assert_eq!(win.getcurx(), 8); // 3 + 5 chars
}

/// Test addchstr - add character string
#[test]
fn test_addchstr() {
    let mut win = Window::new(10, 40, 0, 0).unwrap();

    let chstr: Vec<ChType> = vec![b'H' as ChType, b'i' as ChType, b'!' as ChType];
    win.addchstr(&chstr).unwrap();

    assert_eq!(win.getcurx(), 0); // addchstr doesn't move cursor
}

/// Test wrapped line behavior
#[test]
fn test_line_wrap() {
    let mut win = Window::new(5, 10, 0, 0).unwrap();

    // Fill first line completely (10 chars for a 10-wide window)
    win.addstr("0123456789").unwrap();

    // Cursor should have wrapped to next line
    assert_eq!(win.getcury(), 1);
    assert_eq!(win.getcurx(), 0);
}

/// Test is_pad
#[test]
fn test_is_pad() {
    let win = Window::new(10, 20, 0, 0).unwrap();
    let pad = Window::new_pad(50, 50).unwrap();

    assert!(!win.is_pad());
    assert!(pad.is_pad());
}

/// Test multiple attributes combined
#[test]
fn test_combined_attributes() {
    let mut win = Window::new(10, 20, 0, 0).unwrap();

    let attrs = attr::A_BOLD | attr::A_UNDERLINE | attr::A_REVERSE;
    win.attrset(attrs).unwrap();

    let current = win.getattrs();
    assert!(current & attr::A_BOLD != 0);
    assert!(current & attr::A_UNDERLINE != 0);
    assert!(current & attr::A_REVERSE != 0);
}

/// Test large window
#[test]
fn test_large_window() {
    let win = Window::new(1000, 1000, 0, 0).unwrap();
    assert_eq!(win.getmaxy(), 1000);
    assert_eq!(win.getmaxx(), 1000);
}

/// Test window at various positions
#[test]
fn test_window_positions() {
    let win1 = Window::new(5, 5, 0, 0).unwrap();
    let win2 = Window::new(5, 5, 10, 20).unwrap();
    let win3 = Window::new(5, 5, 100, 200).unwrap();

    assert_eq!(win1.getbegy(), 0);
    assert_eq!(win1.getbegx(), 0);

    assert_eq!(win2.getbegy(), 10);
    assert_eq!(win2.getbegx(), 20);

    assert_eq!(win3.getbegy(), 100);
    assert_eq!(win3.getbegx(), 200);
}

/// Test getpary/getparx for regular window
#[test]
fn test_parent_coords() {
    let win = Window::new(10, 20, 0, 0).unwrap();

    // Regular window has no parent, so getpary/getparx return 0
    // (only subwindows have meaningful parent coordinates)
    let pary = win.getpary();
    let parx = win.getparx();
    assert!(pary >= 0 || pary == -1); // Implementation-defined for non-subwindows
    assert!(parx >= 0 || parx == -1);
}

/// Test reading characters from window
#[test]
fn test_read_chars() {
    let mut win = Window::new(10, 40, 0, 0).unwrap();

    win.addstr("Hello World").unwrap();
    win.mv(0, 0).unwrap();

    // Read character at position
    let ch = win.inch();
    assert_eq!((ch & A_CHARTEXT) as u8, b'H');
}

/// Test scrolling at bottom of window
#[test]
fn test_scroll_at_bottom() {
    let mut win = Window::new(5, 20, 0, 0).unwrap();
    win.scrollok(true);

    // Fill window and cause scroll
    for i in 0..10 {
        win.addstr(&format!("Line {}\n", i)).unwrap();
    }

    // Should have scrolled without panic
}

/// Test carriage return
#[test]
fn test_carriage_return() {
    let mut win = Window::new(10, 20, 0, 0).unwrap();

    win.addstr("Hello").unwrap();
    win.addch(b'\r' as ChType).unwrap();

    assert_eq!(win.getcury(), 0);
    assert_eq!(win.getcurx(), 0);
}

/// Test mvwin - move window
#[test]
fn test_mvwin() {
    let mut win = Window::new(10, 20, 5, 5).unwrap();

    assert_eq!(win.getbegy(), 5);
    assert_eq!(win.getbegx(), 5);

    win.mvwin(10, 15).unwrap();

    assert_eq!(win.getbegy(), 10);
    assert_eq!(win.getbegx(), 15);
}

/// Test mvwin error cases
#[test]
fn test_mvwin_errors() {
    let mut win = Window::new(10, 20, 0, 0).unwrap();

    // Negative positions should fail
    assert!(win.mvwin(-1, 0).is_err());
    assert!(win.mvwin(0, -1).is_err());

    // Pads cannot be moved
    let mut pad = Window::new_pad(50, 50).unwrap();
    assert!(pad.mvwin(10, 10).is_err());
}

/// Test subwin - create subwindow
#[test]
fn test_subwin() {
    let parent = Window::new(20, 40, 5, 10).unwrap();

    // Create a subwindow (screen coordinates)
    let sub = parent.subwin(5, 10, 7, 15).unwrap();

    assert_eq!(sub.getmaxy(), 5);
    assert_eq!(sub.getmaxx(), 10);
    assert_eq!(sub.getbegy(), 7);
    assert_eq!(sub.getbegx(), 15);
    assert!(sub.is_subwin());
}

/// Test subwin bounds checking
#[test]
fn test_subwin_bounds() {
    let parent = Window::new(10, 20, 0, 0).unwrap();

    // Subwindow that extends beyond parent should fail
    assert!(parent.subwin(5, 10, 8, 0).is_err()); // extends past bottom
    assert!(parent.subwin(5, 15, 0, 10).is_err()); // extends past right
    assert!(parent.subwin(5, 10, -1, 0).is_err()); // negative position
}

/// Test derwin - derived window with parent-relative coordinates
#[test]
fn test_derwin() {
    let parent = Window::new(20, 40, 5, 10).unwrap();

    // Create derived window at (2,3) relative to parent
    let derived = parent.derwin(5, 10, 2, 3).unwrap();

    assert_eq!(derived.getmaxy(), 5);
    assert_eq!(derived.getmaxx(), 10);
    // Screen position should be parent position + relative offset
    assert_eq!(derived.getbegy(), 5 + 2); // parent_begy + begy
    assert_eq!(derived.getbegx(), 10 + 3); // parent_begx + begx
    assert!(derived.is_subwin());
}

/// Test derwin bounds checking  
#[test]
fn test_derwin_bounds() {
    let parent = Window::new(10, 20, 0, 0).unwrap();

    // Derived window that extends beyond parent should fail
    assert!(parent.derwin(5, 10, 8, 0).is_err());
    assert!(parent.derwin(5, 15, 0, 10).is_err());
}

/// Test dupwin - duplicate window
#[test]
fn test_dupwin() {
    let mut original = Window::new(10, 20, 5, 10).unwrap();
    original.addstr("Test content").unwrap();
    original.attron(attr::A_BOLD).unwrap();

    let dup = original.dupwin().unwrap();

    // Should have same dimensions and position
    assert_eq!(dup.getmaxy(), original.getmaxy());
    assert_eq!(dup.getmaxx(), original.getmaxx());
    assert_eq!(dup.getbegy(), original.getbegy());
    assert_eq!(dup.getbegx(), original.getbegx());

    // Should have same cursor position
    assert_eq!(dup.getcury(), original.getcury());
    assert_eq!(dup.getcurx(), original.getcurx());

    // Duplicated window should not be a subwindow
    assert!(!dup.is_subwin());
}

/// Test is_subwin
#[test]
fn test_is_subwin() {
    let regular = Window::new(10, 20, 0, 0).unwrap();
    let pad = Window::new_pad(50, 50).unwrap();
    let parent = Window::new(20, 40, 0, 0).unwrap();
    let sub = parent.subwin(5, 10, 2, 3).unwrap();

    assert!(!regular.is_subwin());
    assert!(!pad.is_subwin());
    assert!(sub.is_subwin());
}

/// Test pad_data access
#[test]
fn test_pad_data() {
    let regular = Window::new(10, 20, 0, 0).unwrap();
    let pad = Window::new_pad(50, 50).unwrap();

    // Regular window should return None
    assert!(regular.pad_data().is_none());

    // Pad should return Some with default values
    let data = pad.pad_data().unwrap();
    assert_eq!(data.pad_y, 0);
    assert_eq!(data.pad_x, 0);
}

/// Test set_pad_params
#[test]
fn test_set_pad_params() {
    let mut pad = Window::new_pad(100, 100).unwrap();

    // Set pad parameters
    pad.set_pad_params(5, 10, 0, 0, 20, 40).unwrap();

    let data = pad.pad_data().unwrap();
    assert_eq!(data.pad_y, 5);
    assert_eq!(data.pad_x, 10);
    assert_eq!(data.pad_top, 0);
    assert_eq!(data.pad_left, 0);
    assert_eq!(data.pad_bottom, 20);
    assert_eq!(data.pad_right, 40);
}

/// Test set_pad_params errors
#[test]
fn test_set_pad_params_errors() {
    let mut regular = Window::new(10, 20, 0, 0).unwrap();
    let mut pad = Window::new_pad(50, 50).unwrap();

    // Regular window should fail
    assert!(regular.set_pad_params(0, 0, 0, 0, 10, 10).is_err());

    // Negative parameters should fail
    assert!(pad.set_pad_params(-1, 0, 0, 0, 10, 10).is_err());
    assert!(pad.set_pad_params(0, -1, 0, 0, 10, 10).is_err());
    assert!(pad.set_pad_params(0, 0, -1, 0, 10, 10).is_err());
    assert!(pad.set_pad_params(0, 0, 0, -1, 10, 10).is_err());

    // Invalid screen region (max < min) should fail
    assert!(pad.set_pad_params(0, 0, 10, 0, 5, 10).is_err()); // smaxrow < sminrow
    assert!(pad.set_pad_params(0, 0, 0, 10, 10, 5).is_err()); // smaxcol < smincol
}

/// Test subpad creation
#[test]
fn test_subpad() {
    let parent = Window::new_pad(100, 100).unwrap();

    // Create a subpad
    let sub = parent.subpad(20, 30, 10, 15).unwrap();

    assert_eq!(sub.getmaxy(), 20);
    assert_eq!(sub.getmaxx(), 30);
    assert!(sub.is_pad());
    assert!(sub.is_subwin());
    assert_eq!(sub.getpary(), 10);
    assert_eq!(sub.getparx(), 15);
}

/// Test subpad bounds checking
#[test]
fn test_subpad_bounds() {
    let parent = Window::new_pad(50, 50).unwrap();

    // Subpad that extends beyond parent should fail
    assert!(parent.subpad(30, 20, 30, 0).is_err()); // extends past bottom
    assert!(parent.subpad(20, 40, 0, 20).is_err()); // extends past right
    assert!(parent.subpad(20, 20, -1, 0).is_err()); // negative position
}

/// Test subpad errors
#[test]
fn test_subpad_errors() {
    let regular = Window::new(20, 30, 0, 0).unwrap();

    // Can't create subpad from regular window
    assert!(regular.subpad(5, 5, 0, 0).is_err());
}

/// Test large pad creation
#[test]
fn test_large_pad() {
    // Pads can be larger than screen
    let pad = Window::new_pad(200, 300).unwrap();

    assert_eq!(pad.getmaxy(), 200);
    assert_eq!(pad.getmaxx(), 300);
    assert!(pad.is_pad());
}

/// Test pad content operations
#[test]
fn test_pad_content() {
    let mut pad = Window::new_pad(100, 100).unwrap();

    // Write to various positions
    pad.mvaddstr(50, 50, "Hello").unwrap();
    assert_eq!(pad.getcury(), 50);
    assert_eq!(pad.getcurx(), 55);

    pad.mv(99, 0).unwrap();
    pad.addstr("Last line").unwrap();

    // Should still be able to read from pad
    pad.mv(50, 50).unwrap();
    let s = pad.instr(5);
    assert_eq!(s, "Hello");
}

// ============================================================================
// Menu tests
// ============================================================================

#[cfg(feature = "menu")]
mod menu_tests {
    use ncurses::menu::*;

    /// Test menu creation with items
    #[test]
    fn test_menu_creation() {
        let items = vec![
            MenuItem::new("Item 1", "First item"),
            MenuItem::new("Item 2", "Second item"),
            MenuItem::new("Item 3", "Third item"),
        ];

        let menu = Menu::new(items);
        assert_eq!(menu.item_count(), 3);
    }

    /// Test menu items() method
    #[test]
    fn test_menu_items() {
        let items = vec![
            MenuItem::new("Apple", "A fruit"),
            MenuItem::new("Banana", "Yellow fruit"),
            MenuItem::new("Cherry", "Red fruit"),
        ];

        let menu = Menu::new(items);
        let menu_items = menu.items();

        assert_eq!(menu_items.len(), 3);
        assert_eq!(menu_items[0].borrow().name(), "Apple");
        assert_eq!(menu_items[1].borrow().name(), "Banana");
        assert_eq!(menu_items[2].borrow().name(), "Cherry");
    }

    /// Test menu set_items() method
    #[test]
    fn test_menu_set_items() {
        let initial_items = vec![MenuItem::new("Old 1", ""), MenuItem::new("Old 2", "")];

        let mut menu = Menu::new(initial_items);
        assert_eq!(menu.item_count(), 2);

        // Replace with new items
        let new_items = vec![
            MenuItem::new("New 1", "First new"),
            MenuItem::new("New 2", "Second new"),
            MenuItem::new("New 3", "Third new"),
            MenuItem::new("New 4", "Fourth new"),
        ];

        menu.set_items(new_items).unwrap();

        assert_eq!(menu.item_count(), 4);
        assert_eq!(menu.items()[0].borrow().name(), "New 1");
        assert_eq!(menu.items()[3].borrow().name(), "New 4");

        // Current item should be reset to 0
        assert_eq!(menu.current_item_index(), 0);
    }

    /// Test menu set_items() fails when posted
    #[test]
    fn test_menu_set_items_fails_when_posted() {
        let items = vec![MenuItem::new("Item", "")];
        let mut menu = Menu::new(items);

        // Post the menu
        menu.post().unwrap();

        // Trying to set items should fail
        let new_items = vec![MenuItem::new("New", "")];
        let result = menu.set_items(new_items);
        assert!(result.is_err());

        // Unpost and try again - should work
        menu.unpost().unwrap();
        let new_items = vec![MenuItem::new("New", "")];
        assert!(menu.set_items(new_items).is_ok());
    }

    /// Test menu item indices are updated after set_items
    #[test]
    fn test_menu_set_items_updates_indices() {
        let items = vec![MenuItem::new("A", "")];
        let mut menu = Menu::new(items);

        let new_items = vec![
            MenuItem::new("X", ""),
            MenuItem::new("Y", ""),
            MenuItem::new("Z", ""),
        ];
        menu.set_items(new_items).unwrap();

        // Check that indices are properly set
        assert_eq!(menu.items()[0].borrow().index(), 0);
        assert_eq!(menu.items()[1].borrow().index(), 1);
        assert_eq!(menu.items()[2].borrow().index(), 2);
    }

    /// Test menu_items() free function
    #[test]
    fn test_menu_items_free_function() {
        let items = vec![MenuItem::new("One", "1"), MenuItem::new("Two", "2")];
        let menu = Menu::new(items);

        let items_slice = menu_items(&menu);
        assert_eq!(items_slice.len(), 2);
    }

    /// Test set_menu_items() free function
    #[test]
    fn test_set_menu_items_free_function() {
        let items = vec![MenuItem::new("Old", "")];
        let mut menu = Menu::new(items);

        let new_items = vec![MenuItem::new("A", ""), MenuItem::new("B", "")];
        set_menu_items(&mut menu, new_items).unwrap();

        assert_eq!(menu.item_count(), 2);
    }

    /// Test menu current item navigation
    #[test]
    fn test_menu_navigation() {
        let items = vec![
            MenuItem::new("First", ""),
            MenuItem::new("Second", ""),
            MenuItem::new("Third", ""),
        ];

        let mut menu = Menu::new(items);
        assert_eq!(menu.current_item_index(), 0);

        menu.set_current_item(2).unwrap();
        assert_eq!(menu.current_item_index(), 2);

        // Invalid index should fail
        assert!(menu.set_current_item(10).is_err());
    }

    /// Test menu item properties
    #[test]
    fn test_menu_item_properties() {
        let mut item = MenuItem::new("Test Item", "A test description");

        assert_eq!(item.name(), "Test Item");
        assert_eq!(item.description(), "A test description");
        assert!(item.is_selectable());
        assert!(!item.is_selected());

        // Test options
        item.set_opts(ItemOpts::empty());
        assert!(!item.is_selectable());

        item.set_opts(ItemOpts::O_SELECTABLE);
        assert!(item.is_selectable());
    }

    /// Test menu options
    #[test]
    fn test_menu_options() {
        let items = vec![MenuItem::new("Item", "")];
        let mut menu = Menu::new(items);

        // Default options
        let default_opts = menu.opts();
        assert!(default_opts.contains(MenuOpts::O_ONEVALUE));

        // Modify options
        menu.opts_on(MenuOpts::O_SHOWDESC);
        assert!(menu.opts().contains(MenuOpts::O_SHOWDESC));

        menu.opts_off(MenuOpts::O_SHOWDESC);
        assert!(!menu.opts().contains(MenuOpts::O_SHOWDESC));
    }

    /// Test menu format
    #[test]
    fn test_menu_format() {
        let items = vec![MenuItem::new("Item", "")];
        let mut menu = Menu::new(items);

        menu.set_format(5, 3);
        let (rows, cols) = menu.format();
        assert_eq!(rows, 5);
        assert_eq!(cols, 3);
    }

    /// Test menu mark
    #[test]
    fn test_menu_mark() {
        let items = vec![MenuItem::new("Item", "")];
        let mut menu = Menu::new(items);

        menu.set_mark(">> ");
        assert_eq!(menu.mark(), ">> ");
    }

    /// Test menu pattern matching
    #[test]
    fn test_menu_pattern() {
        let items = vec![
            MenuItem::new("Apple", ""),
            MenuItem::new("Banana", ""),
            MenuItem::new("Apricot", ""),
        ];

        let mut menu = Menu::new(items);
        assert_eq!(menu.pattern(), "");

        menu.set_pattern("Ap").unwrap();
        assert_eq!(menu.pattern(), "Ap");
    }
}

// ============================================================================
// Form tests
// ============================================================================

#[cfg(feature = "form")]
mod form_tests {
    use ncurses::form::*;

    /// Test field creation
    #[test]
    fn test_field_creation() {
        let field = Field::new(1, 20, 5, 10, 0, 0);

        assert_eq!(field.height(), 1);
        assert_eq!(field.width(), 20);
        let (height, width, row, col) = field.dimensions();
        assert_eq!(height, 1);
        assert_eq!(width, 20);
        assert_eq!(row, 5);
        assert_eq!(col, 10);
    }

    /// Test Field::move_to() method
    #[test]
    fn test_field_move_to() {
        let mut field = Field::new(1, 20, 5, 10, 0, 0);

        // Initial position
        let (_, _, row, col) = field.dimensions();
        assert_eq!(row, 5);
        assert_eq!(col, 10);

        // Move to new position
        field.move_to(15, 25).unwrap();
        let (_, _, row, col) = field.dimensions();
        assert_eq!(row, 15);
        assert_eq!(col, 25);
    }

    /// Test Field::move_to() with invalid positions
    #[test]
    fn test_field_move_to_invalid() {
        let mut field = Field::new(1, 20, 5, 10, 0, 0);

        // Negative row should fail
        assert!(field.move_to(-1, 10).is_err());

        // Negative col should fail
        assert!(field.move_to(5, -1).is_err());

        // Both negative should fail
        assert!(field.move_to(-5, -10).is_err());

        // Original position should be unchanged
        let (_, _, row, col) = field.dimensions();
        assert_eq!(row, 5);
        assert_eq!(col, 10);
    }

    /// Test move_field() free function
    #[test]
    fn test_move_field_free_function() {
        let mut field = Field::new(1, 20, 0, 0, 0, 0);

        move_field(&mut field, 10, 20).unwrap();

        let (_, _, row, col) = field.dimensions();
        assert_eq!(row, 10);
        assert_eq!(col, 20);
    }

    /// Test form creation with fields
    #[test]
    fn test_form_creation() {
        let fields = vec![
            Field::new(1, 20, 0, 0, 0, 0),
            Field::new(1, 20, 2, 0, 0, 0),
            Field::new(1, 20, 4, 0, 0, 0),
        ];

        let form = Form::new(fields);
        assert_eq!(form.field_count(), 3);
    }

    /// Test form fields() method
    #[test]
    fn test_form_fields() {
        let fields = vec![
            Field::new(1, 10, 0, 0, 0, 0),
            Field::new(2, 20, 2, 0, 0, 0),
            Field::new(3, 30, 5, 0, 0, 0),
        ];

        let form = Form::new(fields);
        let form_fields = form.fields();

        assert_eq!(form_fields.len(), 3);
        assert_eq!(form_fields[0].borrow().height(), 1);
        assert_eq!(form_fields[0].borrow().width(), 10);
        assert_eq!(form_fields[1].borrow().height(), 2);
        assert_eq!(form_fields[1].borrow().width(), 20);
        assert_eq!(form_fields[2].borrow().height(), 3);
        assert_eq!(form_fields[2].borrow().width(), 30);
    }

    /// Test form set_fields() method
    #[test]
    fn test_form_set_fields() {
        let initial_fields = vec![Field::new(1, 10, 0, 0, 0, 0), Field::new(1, 10, 2, 0, 0, 0)];

        let mut form = Form::new(initial_fields);
        assert_eq!(form.field_count(), 2);

        // Replace with new fields
        let new_fields = vec![
            Field::new(2, 30, 0, 0, 0, 0),
            Field::new(2, 30, 3, 0, 0, 0),
            Field::new(2, 30, 6, 0, 0, 0),
            Field::new(2, 30, 9, 0, 0, 0),
        ];

        form.set_fields(new_fields).unwrap();

        assert_eq!(form.field_count(), 4);
        assert_eq!(form.fields()[0].borrow().height(), 2);
        assert_eq!(form.fields()[0].borrow().width(), 30);

        // Current field should be reset to 0
        assert_eq!(form.current_field_index(), 0);
    }

    /// Test form set_fields() fails when posted
    #[test]
    fn test_form_set_fields_fails_when_posted() {
        let fields = vec![Field::new(1, 20, 0, 0, 0, 0)];
        let mut form = Form::new(fields);

        // Post the form
        form.post().unwrap();

        // Trying to set fields should fail
        let new_fields = vec![Field::new(1, 10, 0, 0, 0, 0)];
        let result = form.set_fields(new_fields);
        assert!(result.is_err());

        // Unpost and try again - should work
        form.unpost().unwrap();
        let new_fields = vec![Field::new(1, 10, 0, 0, 0, 0)];
        assert!(form.set_fields(new_fields).is_ok());
    }

    /// Test form_fields() free function
    #[test]
    fn test_form_fields_free_function() {
        let fields = vec![Field::new(1, 20, 0, 0, 0, 0), Field::new(1, 20, 2, 0, 0, 0)];
        let form = Form::new(fields);

        let fields_slice = form_fields(&form);
        assert_eq!(fields_slice.len(), 2);
    }

    /// Test set_form_fields() free function
    #[test]
    fn test_set_form_fields_free_function() {
        let fields = vec![Field::new(1, 20, 0, 0, 0, 0)];
        let mut form = Form::new(fields);

        let new_fields = vec![Field::new(2, 30, 0, 0, 0, 0), Field::new(2, 30, 3, 0, 0, 0)];
        set_form_fields(&mut form, new_fields).unwrap();

        assert_eq!(form.field_count(), 2);
    }

    /// Test form current field navigation
    #[test]
    fn test_form_navigation() {
        let fields = vec![
            Field::new(1, 20, 0, 0, 0, 0),
            Field::new(1, 20, 2, 0, 0, 0),
            Field::new(1, 20, 4, 0, 0, 0),
        ];

        let mut form = Form::new(fields);
        assert_eq!(form.current_field_index(), 0);

        form.set_current_field(2).unwrap();
        assert_eq!(form.current_field_index(), 2);

        // Invalid index should fail
        assert!(form.set_current_field(10).is_err());
    }

    /// Test field options
    #[test]
    fn test_field_options() {
        let mut field = Field::new(1, 20, 0, 0, 0, 0);

        // Default options
        let default_opts = field.opts();
        assert!(default_opts.contains(FieldOpts::O_ACTIVE));

        // Modify options
        field.set_opts(FieldOpts::O_VISIBLE | FieldOpts::O_STATIC);
        assert!(field.opts().contains(FieldOpts::O_VISIBLE));
        assert!(field.opts().contains(FieldOpts::O_STATIC));
        assert!(!field.opts().contains(FieldOpts::O_ACTIVE));
    }

    /// Test field attributes
    #[test]
    fn test_field_attributes() {
        let mut field = Field::new(1, 20, 0, 0, 0, 0);

        field.set_fore(0x100);
        assert_eq!(field.fore(), 0x100);

        field.set_back(0x200);
        assert_eq!(field.back(), 0x200);

        field.set_pad('_');
        assert_eq!(field.pad(), '_');
    }

    /// Test field buffer operations
    #[test]
    fn test_field_buffer() {
        let mut field = Field::new(1, 20, 0, 0, 0, 0);

        // Initially empty
        assert_eq!(field.buffer(), "");

        // Set buffer
        field.set_buffer("Hello World");
        assert_eq!(field.buffer(), "Hello World");

        // Clear buffer
        field.clear();
        assert_eq!(field.buffer(), "");
    }

    /// Test form options
    #[test]
    fn test_form_options() {
        let fields = vec![Field::new(1, 20, 0, 0, 0, 0)];
        let mut form = Form::new(fields);

        // Set options
        form.set_opts(FormOpts::O_NL_OVERLOAD | FormOpts::O_BS_OVERLOAD);
        assert!(form.opts().contains(FormOpts::O_NL_OVERLOAD));
        assert!(form.opts().contains(FormOpts::O_BS_OVERLOAD));
    }

    /// Test field with offscreen rows (scrollable)
    #[test]
    fn test_field_scrollable() {
        let field = Field::new(2, 20, 0, 0, 5, 0); // 2 visible rows, 5 offscreen

        assert_eq!(field.height(), 2);
        assert_eq!(field.offscreen(), 5);
        assert_eq!(field.total_rows(), 7); // 2 + 5
    }

    /// Test multiple field moves
    #[test]
    fn test_field_multiple_moves() {
        let mut field = Field::new(1, 20, 0, 0, 0, 0);

        // Move multiple times
        field.move_to(5, 10).unwrap();
        field.move_to(10, 20).unwrap();
        field.move_to(15, 30).unwrap();

        let (_, _, row, col) = field.dimensions();
        assert_eq!(row, 15);
        assert_eq!(col, 30);
    }

    /// Test field move to origin
    #[test]
    fn test_field_move_to_origin() {
        let mut field = Field::new(1, 20, 10, 20, 0, 0);

        field.move_to(0, 0).unwrap();

        let (_, _, row, col) = field.dimensions();
        assert_eq!(row, 0);
        assert_eq!(col, 0);
    }
}

// ============================================================================
// Menu workflow integration tests
// ============================================================================

#[cfg(feature = "menu")]
mod menu_workflow_tests {
    use ncurses::menu::*;

    /// Test complete menu creation and interaction workflow
    #[test]
    fn test_menu_full_workflow() {
        // 1. Create items
        let items = vec![
            MenuItem::new("New", "Create new file"),
            MenuItem::new("Open", "Open existing file"),
            MenuItem::new("Save", "Save current file"),
            MenuItem::new("Exit", "Quit application"),
        ];

        // 2. Create menu
        let mut menu = Menu::new(items);

        // 3. Configure menu
        menu.set_format(4, 1);
        menu.set_mark("> ");

        // 4. Post menu
        menu.post().unwrap();

        // 5. Navigate through items
        assert_eq!(menu.current_item_index(), 0);
        assert_eq!(menu.current_item().unwrap().borrow().name(), "New");

        menu.driver(REQ_NEXT_ITEM).unwrap();
        assert_eq!(menu.current_item_index(), 1);
        assert_eq!(menu.current_item().unwrap().borrow().name(), "Open");

        menu.driver(REQ_NEXT_ITEM).unwrap();
        assert_eq!(menu.current_item_index(), 2);

        menu.driver(REQ_PREV_ITEM).unwrap();
        assert_eq!(menu.current_item_index(), 1);

        // 6. Jump to first/last
        menu.driver(REQ_LAST_ITEM).unwrap();
        assert_eq!(menu.current_item_index(), 3);
        assert_eq!(menu.current_item().unwrap().borrow().name(), "Exit");

        menu.driver(REQ_FIRST_ITEM).unwrap();
        assert_eq!(menu.current_item_index(), 0);
        assert_eq!(menu.current_item().unwrap().borrow().name(), "New");

        // 7. Unpost menu
        menu.unpost().unwrap();
    }

    /// Test menu item replacement workflow
    #[test]
    fn test_menu_dynamic_items_workflow() {
        // Start with a simple menu
        let items = vec![MenuItem::new("Option A", ""), MenuItem::new("Option B", "")];
        let mut menu = Menu::new(items);

        menu.post().unwrap();
        assert_eq!(menu.item_count(), 2);

        // Navigate to second item
        menu.driver(REQ_NEXT_ITEM).unwrap();
        assert_eq!(menu.current_item_index(), 1);

        // Unpost to change items
        menu.unpost().unwrap();

        // Replace items
        let new_items = vec![
            MenuItem::new("First", "1st option"),
            MenuItem::new("Second", "2nd option"),
            MenuItem::new("Third", "3rd option"),
            MenuItem::new("Fourth", "4th option"),
            MenuItem::new("Fifth", "5th option"),
        ];
        menu.set_items(new_items).unwrap();

        // Verify new state
        assert_eq!(menu.item_count(), 5);
        assert_eq!(menu.current_item_index(), 0); // Reset to first item

        // Re-post and verify
        menu.post().unwrap();
        assert_eq!(menu.current_item().unwrap().borrow().name(), "First");

        // Navigate through all items
        for expected_idx in 1..5 {
            menu.driver(REQ_NEXT_ITEM).unwrap();
            assert_eq!(menu.current_item_index(), expected_idx);
        }

        menu.unpost().unwrap();
    }

    /// Test menu pattern matching workflow
    #[test]
    fn test_menu_pattern_search_workflow() {
        let items = vec![
            MenuItem::new("Apple", "A fruit"),
            MenuItem::new("Apricot", "Another fruit"),
            MenuItem::new("Banana", "Yellow fruit"),
            MenuItem::new("Blueberry", "Blue fruit"),
            MenuItem::new("Cherry", "Red fruit"),
        ];

        let mut menu = Menu::new(items);
        menu.post().unwrap();

        // Type 'B' to jump to Banana
        menu.driver(b'B' as i32).unwrap();
        assert_eq!(menu.pattern(), "B");

        // Type 'l' to narrow to Blueberry
        menu.driver(b'l' as i32).unwrap();
        assert_eq!(menu.pattern(), "Bl");

        // Clear pattern
        menu.driver(REQ_CLEAR_PATTERN).unwrap();
        assert_eq!(menu.pattern(), "");

        menu.unpost().unwrap();
    }

    /// Test multi-value menu selection workflow
    #[test]
    fn test_menu_multi_selection_workflow() {
        let items = vec![
            MenuItem::new("Item 1", ""),
            MenuItem::new("Item 2", ""),
            MenuItem::new("Item 3", ""),
            MenuItem::new("Item 4", ""),
        ];

        let mut menu = Menu::new(items);

        // Enable multi-value selection
        menu.opts_off(MenuOpts::O_ONEVALUE);

        menu.post().unwrap();

        // Toggle selection on items
        menu.driver(REQ_TOGGLE_ITEM).unwrap(); // Select item 0
        assert!(menu.items()[0].borrow().is_selected());

        menu.driver(REQ_NEXT_ITEM).unwrap();
        menu.driver(REQ_TOGGLE_ITEM).unwrap(); // Select item 1
        assert!(menu.items()[1].borrow().is_selected());

        menu.driver(REQ_NEXT_ITEM).unwrap();
        menu.driver(REQ_NEXT_ITEM).unwrap();
        menu.driver(REQ_TOGGLE_ITEM).unwrap(); // Select item 3
        assert!(menu.items()[3].borrow().is_selected());

        // Verify selections
        let selected: Vec<_> = menu
            .items()
            .iter()
            .filter(|item| item.borrow().is_selected())
            .map(|item| item.borrow().name().to_string())
            .collect();

        assert_eq!(selected.len(), 3);
        assert!(selected.contains(&"Item 1".to_string()));
        assert!(selected.contains(&"Item 2".to_string()));
        assert!(selected.contains(&"Item 4".to_string()));

        menu.unpost().unwrap();
    }

    /// Test menu with custom user data workflow
    #[test]
    fn test_menu_userdata_workflow() {
        #[derive(Debug, Clone, PartialEq)]
        struct MenuAction {
            action_id: u32,
            shortcut: char,
        }

        let mut items = vec![
            MenuItem::new("New", "Create new"),
            MenuItem::new("Open", "Open file"),
            MenuItem::new("Save", "Save file"),
        ];

        // Attach user data to items
        items[0].set_userptr(MenuAction {
            action_id: 1,
            shortcut: 'n',
        });
        items[1].set_userptr(MenuAction {
            action_id: 2,
            shortcut: 'o',
        });
        items[2].set_userptr(MenuAction {
            action_id: 3,
            shortcut: 's',
        });

        let mut menu = Menu::new(items);
        menu.post().unwrap();

        // Retrieve and verify user data
        let item = menu.current_item().unwrap();
        let action = item.borrow().userptr::<MenuAction>().unwrap().clone();
        assert_eq!(action.action_id, 1);
        assert_eq!(action.shortcut, 'n');

        menu.driver(REQ_NEXT_ITEM).unwrap();
        let item = menu.current_item().unwrap();
        let action = item.borrow().userptr::<MenuAction>().unwrap().clone();
        assert_eq!(action.action_id, 2);
        assert_eq!(action.shortcut, 'o');

        menu.unpost().unwrap();
    }

    /// Test menu scrolling with many items
    #[test]
    fn test_menu_scrolling_workflow() {
        // Create many items
        let items: Vec<MenuItem> = (1..=20)
            .map(|i| MenuItem::new(&format!("Item {}", i), &format!("Description {}", i)))
            .collect();

        let mut menu = Menu::new(items);
        menu.set_format(5, 1); // Only show 5 items at a time

        menu.post().unwrap();

        // Scroll down through items
        for i in 0..19 {
            assert_eq!(menu.current_item_index(), i);
            menu.driver(REQ_NEXT_ITEM).unwrap();
        }
        assert_eq!(menu.current_item_index(), 19);
        assert_eq!(menu.current_item().unwrap().borrow().name(), "Item 20");

        // Jump back to first
        menu.driver(REQ_FIRST_ITEM).unwrap();
        assert_eq!(menu.current_item_index(), 0);

        menu.unpost().unwrap();
    }
}

// ============================================================================
// Form workflow integration tests
// ============================================================================

#[cfg(feature = "form")]
mod form_workflow_tests {
    use ncurses::form::*;

    /// Test complete form creation and interaction workflow
    #[test]
    fn test_form_full_workflow() {
        // 1. Create fields
        let fields = vec![
            Field::new(1, 30, 0, 15, 0, 0), // Name field
            Field::new(1, 30, 2, 15, 0, 0), // Email field
            Field::new(1, 15, 4, 15, 0, 0), // Phone field
            Field::new(3, 30, 6, 15, 0, 0), // Comments field (multi-line)
        ];

        // 2. Create form
        let mut form = Form::new(fields);

        // 3. Post form
        form.post().unwrap();

        // 4. Navigate through fields
        assert_eq!(form.current_field_index(), 0);

        form.driver(REQ_NEXT_FIELD).unwrap();
        assert_eq!(form.current_field_index(), 1);

        form.driver(REQ_NEXT_FIELD).unwrap();
        assert_eq!(form.current_field_index(), 2);

        form.driver(REQ_PREV_FIELD).unwrap();
        assert_eq!(form.current_field_index(), 1);

        form.driver(REQ_FIRST_FIELD).unwrap();
        assert_eq!(form.current_field_index(), 0);

        form.driver(REQ_LAST_FIELD).unwrap();
        assert_eq!(form.current_field_index(), 3);

        // 5. Unpost form
        form.unpost().unwrap();
    }

    /// Test form data entry workflow
    #[test]
    fn test_form_data_entry_workflow() {
        let fields = vec![Field::new(1, 20, 0, 0, 0, 0), Field::new(1, 20, 2, 0, 0, 0)];

        let mut form = Form::new(fields);
        form.post().unwrap();

        // Type "Hello" in first field
        for ch in "Hello".chars() {
            form.driver(ch as i32).unwrap();
        }

        // Verify content
        let data = form.data();
        assert_eq!(data[0], "Hello");

        // Move to next field
        form.driver(REQ_NEXT_FIELD).unwrap();

        // Type "World" in second field
        for ch in "World".chars() {
            form.driver(ch as i32).unwrap();
        }

        // Verify both fields
        let data = form.data();
        assert_eq!(data[0], "Hello");
        assert_eq!(data[1], "World");

        form.unpost().unwrap();
    }

    /// Test form cursor movement workflow
    #[test]
    fn test_form_cursor_movement_workflow() {
        let fields = vec![Field::new(1, 30, 0, 0, 0, 0)];
        let mut form = Form::new(fields);
        form.post().unwrap();

        // Type some text
        for ch in "Hello World".chars() {
            form.driver(ch as i32).unwrap();
        }

        // Check cursor position
        let field = form.current_field().unwrap();
        let (_, cursor_col) = field.borrow().cursor_pos();
        assert_eq!(cursor_col, 11); // After "Hello World"

        // Move cursor left
        form.driver(REQ_LEFT_CHAR).unwrap();
        let (_, cursor_col) = field.borrow().cursor_pos();
        assert_eq!(cursor_col, 10);

        // Move to beginning of field
        form.driver(REQ_BEG_FIELD).unwrap();
        let (_, cursor_col) = field.borrow().cursor_pos();
        assert_eq!(cursor_col, 0);

        // Move to end of field
        form.driver(REQ_END_FIELD).unwrap();
        let (_, cursor_col) = field.borrow().cursor_pos();
        assert_eq!(cursor_col, 11);

        form.unpost().unwrap();
    }

    /// Test form field editing workflow
    #[test]
    fn test_form_editing_workflow() {
        let fields = vec![Field::new(1, 30, 0, 0, 0, 0)];
        let mut form = Form::new(fields);
        form.post().unwrap();

        // Type text
        for ch in "ABCDE".chars() {
            form.driver(ch as i32).unwrap();
        }
        assert_eq!(form.data()[0], "ABCDE");

        // Delete previous character (backspace)
        form.driver(REQ_DEL_PREV).unwrap();
        assert_eq!(form.data()[0], "ABCD");

        // Move to beginning and delete forward
        form.driver(REQ_BEG_FIELD).unwrap();
        form.driver(REQ_DEL_CHAR).unwrap();
        assert_eq!(form.data()[0], "BCD");

        // Clear to end of line
        form.driver(REQ_CLR_EOL).unwrap();
        assert_eq!(form.data()[0], "");

        // Type new text
        for ch in "XYZ".chars() {
            form.driver(ch as i32).unwrap();
        }
        assert_eq!(form.data()[0], "XYZ");

        // Clear entire field
        form.driver(REQ_CLR_FIELD).unwrap();
        assert_eq!(form.data()[0], "");

        form.unpost().unwrap();
    }

    /// Test form field replacement workflow
    #[test]
    fn test_form_dynamic_fields_workflow() {
        // Start with simple form
        let fields = vec![Field::new(1, 20, 0, 0, 0, 0), Field::new(1, 20, 2, 0, 0, 0)];
        let mut form = Form::new(fields);

        form.post().unwrap();

        // Enter data in fields
        for ch in "Field1".chars() {
            form.driver(ch as i32).unwrap();
        }
        form.driver(REQ_NEXT_FIELD).unwrap();
        for ch in "Field2".chars() {
            form.driver(ch as i32).unwrap();
        }

        // Unpost to change fields
        form.unpost().unwrap();

        // Replace with new fields
        let new_fields = vec![
            Field::new(1, 30, 0, 0, 0, 0),
            Field::new(1, 30, 2, 0, 0, 0),
            Field::new(1, 30, 4, 0, 0, 0),
        ];
        form.set_fields(new_fields).unwrap();

        // Verify new state
        assert_eq!(form.field_count(), 3);
        assert_eq!(form.current_field_index(), 0);

        // All fields should be empty (new fields)
        let data = form.data();
        assert!(data.iter().all(|s| s.is_empty()));

        // Re-post and work with new fields
        form.post().unwrap();
        for ch in "NewField1".chars() {
            form.driver(ch as i32).unwrap();
        }
        assert_eq!(form.data()[0], "NewField1");

        form.unpost().unwrap();
    }

    /// Test form validation workflow
    #[test]
    fn test_form_validation_workflow() {
        let mut field = Field::new(1, 10, 0, 0, 0, 0);

        // Set integer type validation (1-100)
        field.set_type(TypeInteger {
            padding: 0,
            min: 1,
            max: 100,
        });

        let fields = vec![field];
        let mut form = Form::new(fields);
        form.post().unwrap();

        // Enter valid number
        for ch in "50".chars() {
            form.driver(ch as i32).unwrap();
        }
        assert!(form.validate_all());

        // Clear and enter invalid number (0)
        form.driver(REQ_CLR_FIELD).unwrap();
        for ch in "0".chars() {
            form.driver(ch as i32).unwrap();
        }
        assert!(!form.validate_all());

        // Clear and enter invalid number (150)
        form.driver(REQ_CLR_FIELD).unwrap();
        for ch in "150".chars() {
            form.driver(ch as i32).unwrap();
        }
        assert!(!form.validate_all());

        // Clear and enter non-numeric
        form.driver(REQ_CLR_FIELD).unwrap();
        for ch in "abc".chars() {
            form.driver(ch as i32).unwrap();
        }
        assert!(!form.validate_all());

        form.unpost().unwrap();
    }

    /// Test multi-line field workflow
    #[test]
    fn test_form_multiline_field_workflow() {
        let fields = vec![Field::new(3, 20, 0, 0, 2, 0)]; // 3 visible rows, 2 offscreen
        let mut form = Form::new(fields);
        form.post().unwrap();

        // Type on first line
        for ch in "Line 1".chars() {
            form.driver(ch as i32).unwrap();
        }

        // Move to next line
        form.driver(REQ_NEXT_LINE).unwrap();

        // Type on second line
        for ch in "Line 2".chars() {
            form.driver(ch as i32).unwrap();
        }

        // Move to next line
        form.driver(REQ_NEXT_LINE).unwrap();

        // Type on third line
        for ch in "Line 3".chars() {
            form.driver(ch as i32).unwrap();
        }

        // Verify content
        let field = form.current_field().unwrap();
        let buffer = field.borrow().buffer();
        assert!(buffer.contains("Line 1"));
        assert!(buffer.contains("Line 2"));
        assert!(buffer.contains("Line 3"));

        // Navigate up
        form.driver(REQ_UP_CHAR).unwrap();
        form.driver(REQ_UP_CHAR).unwrap();

        // Verify cursor is on first line
        let (cursor_row, _) = field.borrow().cursor_pos();
        assert_eq!(cursor_row, 0);

        form.unpost().unwrap();
    }

    /// Test field with custom attributes workflow
    #[test]
    fn test_form_field_attributes_workflow() {
        let mut field = Field::new(1, 20, 0, 0, 0, 0);

        // Set custom attributes
        field.set_fore(0x10000); // Bold
        field.set_back(0x20000); // Dim
        field.set_pad('_');

        assert_eq!(field.fore(), 0x10000);
        assert_eq!(field.back(), 0x20000);
        assert_eq!(field.pad(), '_');

        // Set field options
        field.set_opts(FieldOpts::O_VISIBLE | FieldOpts::O_ACTIVE | FieldOpts::O_STATIC);

        let fields = vec![field];
        let form = Form::new(fields);

        // Verify attributes are preserved
        let field_ref = form.field(0).unwrap();
        assert_eq!(field_ref.borrow().fore(), 0x10000);
        assert_eq!(field_ref.borrow().back(), 0x20000);
        assert_eq!(field_ref.borrow().pad(), '_');
        assert!(field_ref.borrow().opts().contains(FieldOpts::O_STATIC));
    }

    /// Test form with moved fields workflow
    #[test]
    fn test_form_moved_fields_workflow() {
        // Create fields at initial positions
        let mut field1 = Field::new(1, 20, 0, 0, 0, 0);
        let mut field2 = Field::new(1, 20, 2, 0, 0, 0);

        // Move fields to new positions
        field1.move_to(5, 10).unwrap();
        field2.move_to(7, 10).unwrap();

        // Create form with moved fields
        let form = Form::new(vec![field1, field2]);

        // Verify positions
        let f1 = form.field(0).unwrap();
        let f2 = form.field(1).unwrap();

        let (_, _, row1, col1) = f1.borrow().dimensions();
        let (_, _, row2, col2) = f2.borrow().dimensions();

        assert_eq!(row1, 5);
        assert_eq!(col1, 10);
        assert_eq!(row2, 7);
        assert_eq!(col2, 10);
    }
}
