//! Screen management for ncurses-pure.
//!
//! This module implements the `Screen` structure, which is the main entry point
//! for using ncurses. It manages the terminal, windows, colors, and input/output.

use crate::attr::{self, A_NORMAL};
use crate::color::ColorManager;
use crate::error::{Error, Result};
use crate::input::{EscapeMatch, EscapeParser, InputBuffer, InputMode};
use crate::key::KEY_MOUSE;
#[cfg(feature = "mouse")]
use crate::mouse::{is_mouse_prefix, parse_sgr_mouse, MouseEvent, MouseProtocol, MouseState};
#[cfg(feature = "slk")]
use crate::slk::{SlkFormat, SlkState};
use crate::terminal::{TermState, Terminal};
use crate::types::ColorT;
use crate::types::CursorVisibility;
#[cfg(feature = "mouse")]
use crate::types::MmaskT;
use crate::types::{AttrT, ChType, Delay};
use crate::window::Window;

use std::time::{Duration, Instant};

/// The main ncurses screen structure.
///
/// This structure owns the terminal, windows, and all state necessary for
/// curses operations. It must be initialized before any curses functions
/// can be used.
///
/// # Example
///
/// ```rust,no_run
/// use ncurses::*;
///
/// fn main() -> Result<()> {
///     let mut screen = Screen::init()?;
///     
///     let stdscr = screen.stdscr_mut();
///     stdscr.addstr("Hello, World!")?;
///     
///     screen.refresh()?;
///     screen.getch()?;
///     
///     Ok(())
/// }
/// ```
pub struct Screen {
    /// The terminal interface.
    terminal: Terminal,

    /// The standard screen window.
    stdscr: Window,

    /// The current screen (what's actually on the terminal).
    curscr: Window,

    /// The new screen (pending updates).
    newscr: Window,

    /// Color manager.
    colors: ColorManager,

    /// Input mode settings.
    input_mode: InputMode,

    /// Input buffer for typeahead.
    input_buffer: InputBuffer,

    /// Escape sequence parser.
    escape_parser: EscapeParser,

    /// Current cursor visibility.
    cursor_visibility: CursorVisibility,

    /// Whether the screen has been initialized.
    initialized: bool,

    /// The ESCDELAY value in milliseconds.
    escdelay: i32,

    /// The TABSIZE value.
    tabsize: i32,

    /// Mouse state (when mouse feature is enabled).
    #[cfg(feature = "mouse")]
    mouse: MouseState,

    /// Current mouse protocol.
    #[cfg(feature = "mouse")]
    mouse_protocol: MouseProtocol,

    /// Whether filter mode is enabled (single-line mode).
    filtered: bool,

    /// Soft label key state (when slk feature is enabled).
    #[cfg(feature = "slk")]
    slk: Option<SlkState>,
}

impl Screen {
    /// Initialize the screen (equivalent to `initscr()`).
    ///
    /// This sets up the terminal for curses operation:
    /// - Saves the terminal state
    /// - Enters program mode (raw, no echo)
    /// - Creates the standard screen window
    /// - Initializes color support if available
    pub fn init() -> Result<Self> {
        let mut terminal = Terminal::from_stdio()?;

        // Get terminal dimensions
        let lines = terminal.lines();
        let cols = terminal.columns();

        // Update global dimensions
        globals::set_dimensions(lines, cols);

        // Create the three main windows
        let stdscr = Window::new(lines, cols, 0, 0)?;
        let curscr = Window::new(lines, cols, 0, 0)?;
        let newscr = Window::new(lines, cols, 0, 0)?;

        // Set up color manager
        let colors = ColorManager::new(
            terminal.colors(),
            terminal.color_pairs(),
            terminal.can_change_color(),
        );

        // Enter program mode
        terminal.enter_program_mode()?;

        let mut screen = Self {
            terminal,
            stdscr,
            curscr,
            newscr,
            colors,
            input_mode: InputMode::new(),
            input_buffer: InputBuffer::new(),
            escape_parser: EscapeParser::new(),
            cursor_visibility: CursorVisibility::Normal,
            initialized: true,
            escdelay: 100,
            tabsize: 8,
            #[cfg(feature = "mouse")]
            mouse: MouseState::new(),
            #[cfg(feature = "mouse")]
            mouse_protocol: MouseProtocol::None,
            filtered: false,
            #[cfg(feature = "slk")]
            slk: None,
        };

        // Set default input mode (cbreak, noecho)
        screen.cbreak()?;
        screen.noecho()?;

        // Clear the screen
        screen.terminal.clear_screen()?;
        screen.terminal.flush()?;

        Ok(screen)
    }

    /// End curses mode (equivalent to `endwin()`).
    ///
    /// This restores the terminal to its original state. After calling this,
    /// you can call `refresh()` to re-enter curses mode if needed.
    pub fn endwin(&mut self) -> Result<()> {
        if self.initialized {
            // Disable mouse if enabled
            #[cfg(feature = "mouse")]
            if self.mouse_protocol != MouseProtocol::None {
                let _ = self
                    .terminal
                    .write(self.mouse_protocol.disable_sequence().as_bytes());
                let _ = self
                    .terminal
                    .write(MouseProtocol::ButtonEvent.disable_sequence().as_bytes());
                self.mouse_protocol = MouseProtocol::None;
            }

            // Show cursor
            self.terminal.cursor_visible(true)?;

            // Leave program mode (restores terminal settings)
            self.terminal.leave_program_mode()?;

            self.initialized = false;
        }
        Ok(())
    }

    /// Check if curses mode has been suspended.
    pub fn isendwin(&self) -> bool {
        !self.initialized || self.terminal.state() == TermState::Suspend
    }

    /// Get a reference to the standard screen window.
    pub fn stdscr(&self) -> &Window {
        &self.stdscr
    }

    /// Get a mutable reference to the standard screen window.
    pub fn stdscr_mut(&mut self) -> &mut Window {
        &mut self.stdscr
    }

    /// Get the current screen window (for comparison during refresh).
    pub fn curscr(&self) -> &Window {
        &self.curscr
    }

    /// Get the new screen window.
    pub fn newscr(&self) -> &Window {
        &self.newscr
    }

    // ========================================================================
    // Terminal information
    // ========================================================================

    /// Get the number of lines on the screen.
    pub fn lines(&self) -> i32 {
        self.terminal.lines()
    }

    /// Get the number of columns on the screen.
    pub fn cols(&self) -> i32 {
        self.terminal.columns()
    }

    /// Get the terminal type name.
    pub fn termname(&self) -> &str {
        self.terminal.term_type()
    }

    /// Get the baud rate.
    pub fn baudrate(&self) -> i32 {
        crate::terminal::baudrate()
    }

    /// Update the terminal size (call after SIGWINCH).
    pub fn update_term_size(&mut self) -> Result<()> {
        self.terminal.update_size()?;

        let lines = self.terminal.lines();
        let cols = self.terminal.columns();

        // Resize windows
        self.resize_term(lines, cols)?;

        Ok(())
    }

    /// Resize the terminal to the specified size.
    pub fn resize_term(&mut self, lines: i32, cols: i32) -> Result<()> {
        // Update global dimensions
        globals::set_dimensions(lines, cols);

        // Recreate windows with new size
        self.stdscr = Window::new(lines, cols, 0, 0)?;
        self.curscr = Window::new(lines, cols, 0, 0)?;
        self.newscr = Window::new(lines, cols, 0, 0)?;

        // Mark everything as needing update
        self.stdscr.touchwin();

        Ok(())
    }

    /// Check if the terminal has been resized.
    ///
    /// Compares the current terminal size with the stored dimensions
    /// and returns `true` if they differ (meaning a resize is needed).
    pub fn is_term_resized(&self, lines: i32, cols: i32) -> bool {
        self.terminal.lines() != lines || self.terminal.columns() != cols
    }

    /// Resize the terminal (alias for `resize_term`).
    ///
    /// This is the ncurses `resizeterm()` function. It resizes the standard
    /// windows (stdscr, curscr, newscr) to the specified size.
    ///
    /// Unlike `resize_term`, this function may also attempt to adjust
    /// subwindows, but in the current implementation they are equivalent.
    pub fn resizeterm(&mut self, lines: i32, cols: i32) -> Result<()> {
        self.resize_term(lines, cols)
    }

    // ========================================================================
    // Window creation
    // ========================================================================

    /// Create a new window.
    pub fn newwin(&self, nlines: i32, ncols: i32, begy: i32, begx: i32) -> Result<Window> {
        // Use screen dimensions if 0
        let nlines = if nlines == 0 {
            self.terminal.lines() - begy
        } else {
            nlines
        };
        let ncols = if ncols == 0 {
            self.terminal.columns() - begx
        } else {
            ncols
        };

        Window::new(nlines, ncols, begy, begx)
    }

    /// Create a new pad.
    pub fn newpad(&self, nlines: i32, ncols: i32) -> Result<Window> {
        Window::new_pad(nlines, ncols)
    }

    // ========================================================================
    // Pad refresh operations
    // ========================================================================

    /// Refresh a pad, copying a portion to the physical screen.
    ///
    /// This is the pad equivalent of `wrefresh`. It copies a rectangular
    /// region of the pad to a rectangular region of the physical screen.
    ///
    /// # Arguments
    ///
    /// * `pad` - The pad to refresh
    /// * `pminrow` - Row in pad to start copying from
    /// * `pmincol` - Column in pad to start copying from
    /// * `sminrow` - Top row on screen for the display region
    /// * `smincol` - Left column on screen for the display region
    /// * `smaxrow` - Bottom row on screen for the display region
    /// * `smaxcol` - Right column on screen for the display region
    #[allow(clippy::too_many_arguments)]
    pub fn prefresh(
        &mut self,
        pad: &mut Window,
        pminrow: i32,
        pmincol: i32,
        sminrow: i32,
        smincol: i32,
        smaxrow: i32,
        smaxcol: i32,
    ) -> Result<()> {
        // First copy to virtual screen
        self.pnoutrefresh(pad, pminrow, pmincol, sminrow, smincol, smaxrow, smaxcol)?;
        // Then update physical screen
        self.doupdate()
    }

    /// Copy a pad to the virtual screen (without updating physical screen).
    ///
    /// This is the pad equivalent of `wnoutrefresh`. Use this followed by
    /// `doupdate()` for efficient updates when refreshing multiple pads.
    ///
    /// # Arguments
    ///
    /// * `pad` - The pad to copy
    /// * `pminrow` - Row in pad to start copying from
    /// * `pmincol` - Column in pad to start copying from
    /// * `sminrow` - Top row on screen for the display region
    /// * `smincol` - Left column on screen for the display region
    /// * `smaxrow` - Bottom row on screen for the display region
    /// * `smaxcol` - Right column on screen for the display region
    #[allow(clippy::too_many_arguments)]
    pub fn pnoutrefresh(
        &mut self,
        pad: &mut Window,
        pminrow: i32,
        pmincol: i32,
        sminrow: i32,
        smincol: i32,
        smaxrow: i32,
        smaxcol: i32,
    ) -> Result<()> {
        // Validate that this is actually a pad
        if !pad.is_pad() {
            return Err(Error::InvalidArgument("window is not a pad".into()));
        }

        // Validate parameters
        if pminrow < 0 || pmincol < 0 || sminrow < 0 || smincol < 0 {
            return Err(Error::InvalidArgument("negative pad coordinates".into()));
        }

        if smaxrow < sminrow || smaxcol < smincol {
            return Err(Error::InvalidArgument("invalid screen region".into()));
        }

        // Store pad parameters for future reference
        pad.set_pad_params(pminrow, pmincol, sminrow, smincol, smaxrow, smaxcol)?;

        // Calculate the dimensions to copy
        let pad_height = pad.getmaxy();
        let pad_width = pad.getmaxx();
        let screen_height = self.newscr.getmaxy();
        let screen_width = self.newscr.getmaxx();

        // Clamp screen region to actual screen dimensions
        let sminrow = sminrow.max(0);
        let smincol = smincol.max(0);
        let smaxrow = smaxrow.min(screen_height - 1);
        let smaxcol = smaxcol.min(screen_width - 1);

        // Calculate how many rows/cols to actually copy
        let copy_height = (smaxrow - sminrow + 1).min(pad_height - pminrow);
        let copy_width = (smaxcol - smincol + 1).min(pad_width - pmincol);

        if copy_height <= 0 || copy_width <= 0 {
            return Ok(()); // Nothing to copy
        }

        // Copy the pad content to newscr
        for dy in 0..copy_height {
            let pad_y = (pminrow + dy) as usize;
            let screen_y = (sminrow + dy) as usize;

            if pad_y >= pad_height as usize || screen_y >= screen_height as usize {
                break;
            }

            if let Some(pad_line) = pad.line(pad_y) {
                for dx in 0..copy_width {
                    let pad_x = (pmincol + dx) as usize;
                    let screen_x = (smincol + dx) as usize;

                    if pad_x >= pad_width as usize || screen_x >= screen_width as usize {
                        break;
                    }

                    let ch = pad_line.get(pad_x);
                    if let Some(newscr_line) = self.newscr.line_mut(screen_y) {
                        newscr_line.set(screen_x, ch);
                    }
                }
            }
        }

        // Mark the affected region as touched in newscr
        for dy in 0..copy_height {
            let screen_y = (sminrow + dy) as usize;
            if let Some(newscr_line) = self.newscr.line_mut(screen_y) {
                newscr_line.touch();
            }
        }

        // Clear touch flags on the pad since we've processed it
        pad.untouchwin();

        Ok(())
    }

    /// Add a character to a pad and refresh.
    ///
    /// This is equivalent to calling `addch` on the pad followed by `prefresh`
    /// with the pad's stored parameters. It's useful for interactive applications
    /// that need to update a pad one character at a time.
    ///
    /// # Arguments
    ///
    /// * `pad` - The pad to update
    /// * `ch` - The character to add
    ///
    /// # Note
    ///
    /// The pad must have had `prefresh` or `pnoutrefresh` called on it at least
    /// once to establish the display parameters.
    pub fn pechochar(&mut self, pad: &mut Window, ch: ChType) -> Result<()> {
        if !pad.is_pad() {
            return Err(Error::InvalidArgument("window is not a pad".into()));
        }

        // Add the character to the pad
        pad.addch(ch)?;

        // Get the stored pad parameters
        let pad_data = pad
            .pad_data()
            .ok_or_else(|| Error::InvalidArgument("pad has no stored parameters".into()))?;

        let pminrow = pad_data.pad_y as i32;
        let pmincol = pad_data.pad_x as i32;
        let sminrow = pad_data.pad_top as i32;
        let smincol = pad_data.pad_left as i32;
        let smaxrow = pad_data.pad_bottom as i32;
        let smaxcol = pad_data.pad_right as i32;

        // Refresh the pad
        self.prefresh(pad, pminrow, pmincol, sminrow, smincol, smaxrow, smaxcol)
    }

    // ========================================================================
    // Color support
    // ========================================================================

    /// Check if the terminal has color support.
    pub fn has_colors(&self) -> bool {
        self.colors.has_colors()
    }

    /// Start color mode.
    pub fn start_color(&mut self) -> Result<()> {
        self.colors.start()
    }

    /// Initialize a color pair.
    pub fn init_pair(&mut self, pair: i16, fg: ColorT, bg: ColorT) -> Result<()> {
        self.colors.init_pair(pair, fg, bg)
    }

    /// Get the foreground and background of a color pair.
    pub fn pair_content(&self, pair: i16) -> Result<(ColorT, ColorT)> {
        self.colors.pair_content(pair)
    }

    /// Initialize a color with RGB values.
    pub fn init_color(&mut self, color: ColorT, r: i16, g: i16, b: i16) -> Result<()> {
        self.colors.init_color(color, r, g, b)
    }

    /// Get the RGB content of a color.
    pub fn color_content(&self, color: ColorT) -> Result<(i16, i16, i16)> {
        self.colors.color_content(color)
    }

    /// Check if colors can be changed.
    pub fn can_change_color(&self) -> bool {
        self.colors.can_change_color()
    }

    /// Get the number of colors.
    pub fn num_colors(&self) -> i32 {
        self.colors.num_colors()
    }

    /// Get the number of color pairs.
    pub fn num_color_pairs(&self) -> i32 {
        self.colors.num_pairs()
    }

    /// Enable use of default colors (-1).
    pub fn use_default_colors(&mut self) -> Result<()> {
        self.colors.use_default_colors()
    }

    /// Set default foreground and background for pair 0.
    pub fn assume_default_colors(&mut self, fg: ColorT, bg: ColorT) -> Result<()> {
        self.colors.assume_default_colors(fg, bg)
    }

    // ========================================================================
    // Input mode control
    // ========================================================================

    /// Enable raw mode (no processing of input).
    pub fn raw(&mut self) -> Result<()> {
        self.terminal.raw(true)?;
        self.input_mode.raw = true;
        self.input_mode.cbreak = 0;
        Ok(())
    }

    /// Disable raw mode.
    pub fn noraw(&mut self) -> Result<()> {
        self.terminal.raw(false)?;
        self.input_mode.raw = false;
        Ok(())
    }

    /// Enable cbreak mode (no line buffering).
    pub fn cbreak(&mut self) -> Result<()> {
        self.terminal.cbreak(true)?;
        self.input_mode.cbreak = 1;
        self.input_mode.raw = false;
        Ok(())
    }

    /// Disable cbreak mode.
    pub fn nocbreak(&mut self) -> Result<()> {
        self.terminal.cbreak(false)?;
        self.input_mode.cbreak = 0;
        Ok(())
    }

    /// Enable echo mode.
    pub fn echo(&mut self) -> Result<()> {
        self.terminal.echo(true)?;
        self.input_mode.echo = true;
        Ok(())
    }

    /// Disable echo mode.
    pub fn noecho(&mut self) -> Result<()> {
        self.terminal.echo(false)?;
        self.input_mode.echo = false;
        Ok(())
    }

    /// Enable newline translation.
    pub fn nl(&mut self) -> Result<()> {
        self.input_mode.nl = true;
        Ok(())
    }

    /// Disable newline translation.
    pub fn nonl(&mut self) -> Result<()> {
        self.input_mode.nl = false;
        Ok(())
    }

    /// Enable halfdelay mode (timeout in tenths of seconds).
    pub fn halfdelay(&mut self, tenths: i32) -> Result<()> {
        if !(1..=255).contains(&tenths) {
            return Err(Error::InvalidArgument(
                "halfdelay must be 1-255 tenths of a second".into(),
            ));
        }
        self.input_mode.cbreak = tenths + 1;
        Ok(())
    }

    /// Set the ESCDELAY value.
    pub fn set_escdelay(&mut self, delay: i32) {
        self.escdelay = delay;
        self.escape_parser.set_escape_delay(delay);
    }

    /// Get the ESCDELAY value.
    pub fn get_escdelay(&self) -> i32 {
        self.escdelay
    }

    /// Set the TABSIZE value.
    pub fn set_tabsize(&mut self, size: i32) {
        self.tabsize = size;
    }

    /// Get the TABSIZE value.
    pub fn get_tabsize(&self) -> i32 {
        self.tabsize
    }

    // ========================================================================
    // Mouse support (requires "mouse" feature)
    // ========================================================================

    /// Enable mouse events with the specified mask.
    ///
    /// Returns the old mask. Pass 0 to disable mouse support.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use ncurses::*;
    /// use ncurses::mouse::*;
    ///
    /// # fn main() -> Result<()> {
    /// let mut screen = Screen::init()?;
    /// let old_mask = screen.mousemask(ALL_MOUSE_EVENTS);
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "mouse")]
    pub fn mousemask(&mut self, newmask: MmaskT) -> MmaskT {
        let old = self.mouse.mousemask(newmask);

        // Enable/disable terminal mouse reporting
        if newmask != 0 && self.mouse_protocol == MouseProtocol::None {
            // Enable SGR mouse protocol (most modern and feature-rich)
            self.mouse_protocol = MouseProtocol::Sgr;
            let _ = self
                .terminal
                .write(MouseProtocol::Sgr.enable_sequence().as_bytes());
            // Also enable button event tracking
            let _ = self
                .terminal
                .write(MouseProtocol::ButtonEvent.enable_sequence().as_bytes());
            let _ = self.terminal.flush();
        } else if newmask == 0 && self.mouse_protocol != MouseProtocol::None {
            // Disable mouse reporting
            let _ = self
                .terminal
                .write(self.mouse_protocol.disable_sequence().as_bytes());
            let _ = self
                .terminal
                .write(MouseProtocol::ButtonEvent.disable_sequence().as_bytes());
            let _ = self.terminal.flush();
            self.mouse_protocol = MouseProtocol::None;
        }

        old
    }

    /// Get the next mouse event.
    ///
    /// Call this after receiving KEY_MOUSE from getch().
    #[cfg(feature = "mouse")]
    pub fn getmouse(&mut self) -> Option<MouseEvent> {
        self.mouse.getmouse()
    }

    /// Push a mouse event back to the queue.
    #[cfg(feature = "mouse")]
    pub fn ungetmouse(&mut self, event: MouseEvent) -> bool {
        self.mouse.ungetmouse(event)
    }

    /// Set the click interval in milliseconds.
    ///
    /// Events within this interval are considered clicks/double-clicks.
    /// Returns the old interval.
    #[cfg(feature = "mouse")]
    pub fn mouseinterval(&mut self, interval: i32) -> i32 {
        self.mouse.mouseinterval(interval)
    }

    /// Check if mouse support is currently enabled.
    #[cfg(feature = "mouse")]
    pub fn has_mouse(&self) -> bool {
        self.mouse.is_enabled()
    }

    // ========================================================================
    // Cursor control
    // ========================================================================

    /// Set cursor visibility.
    ///
    /// # Arguments
    ///
    /// * `visibility` - The desired cursor visibility. Can be:
    ///   - `CursorVisibility::Invisible` (0) - cursor is hidden
    ///   - `CursorVisibility::Normal` (1) - default cursor visibility  
    ///   - `CursorVisibility::VeryVisible` (2) - very visible cursor (e.g., block)
    ///
    /// # Returns
    ///
    /// Returns the previous cursor visibility setting.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ncurses::types::CursorVisibility;
    ///
    /// // Hide the cursor
    /// let old = screen.curs_set(CursorVisibility::Invisible)?;
    ///
    /// // Also accepts i32 for compatibility
    /// let old = screen.curs_set(0)?;
    /// ```
    pub fn curs_set(
        &mut self,
        visibility: impl TryInto<CursorVisibility>,
    ) -> Result<CursorVisibility> {
        let visibility = visibility
            .try_into()
            .map_err(|_| Error::InvalidArgument("cursor visibility must be 0, 1, or 2".into()))?;

        let old = self.cursor_visibility;
        self.cursor_visibility = visibility;

        match visibility {
            CursorVisibility::Invisible => self.terminal.cursor_visible(false)?,
            _ => self.terminal.cursor_visible(true)?,
        }

        Ok(old)
    }

    /// Get the current cursor visibility setting.
    #[must_use]
    pub fn cursor_visibility(&self) -> CursorVisibility {
        self.cursor_visibility
    }

    // ========================================================================
    // Output functions
    // ========================================================================

    /// Ring the terminal bell.
    pub fn beep(&mut self) -> Result<()> {
        self.terminal.beep()?;
        self.terminal.flush()
    }

    /// Flash the screen (visual bell).
    pub fn flash(&mut self) -> Result<()> {
        self.terminal.flash()
    }

    // ========================================================================
    // Refresh operations
    // ========================================================================

    /// Refresh the standard screen.
    pub fn refresh(&mut self) -> Result<()> {
        // Copy stdscr to newscr then update
        self.stdscr_to_newscr()?;
        self.doupdate()
    }

    /// Copy stdscr to the new screen buffer.
    fn stdscr_to_newscr(&mut self) -> Result<()> {
        let maxy = self.stdscr.getmaxy();
        let maxx = self.stdscr.getmaxx();

        for y in 0..maxy {
            if let Some(line) = self.stdscr.line(y as usize) {
                if line.is_touched() {
                    if let Some((first, last)) = line.changed_range() {
                        for x in first..=last {
                            if x >= maxx as usize {
                                break;
                            }
                            let ch = line.get(x);
                            if let Some(newscr_line) = self.newscr.line_mut(y as usize) {
                                newscr_line.set(x, ch);
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Refresh a window (copy to physical screen).
    pub fn wrefresh(&mut self, win: &mut Window) -> Result<()> {
        // Copy window to newscr
        self.wnoutrefresh(win)?;
        // Update the physical screen
        self.doupdate()
    }

    /// Copy a window to the virtual screen (but don't update physical screen).
    pub fn wnoutrefresh(&mut self, win: &Window) -> Result<()> {
        // Copy changed portions of win to newscr
        let begy = win.getbegy();
        let begx = win.getbegx();
        let maxy = win.getmaxy();

        for y in 0..maxy {
            let screen_y = (begy + y) as usize;
            if screen_y >= self.newscr.getmaxy() as usize {
                break;
            }

            if let Some(line) = win.line(y as usize) {
                if line.is_touched() {
                    if let Some((first, last)) = line.changed_range() {
                        for x in first..=last {
                            let screen_x = (begx as usize) + x;
                            if screen_x >= self.newscr.getmaxx() as usize {
                                break;
                            }

                            let ch = line.get(x);
                            if let Some(newscr_line) = self.newscr.line_mut(screen_y) {
                                newscr_line.set(screen_x, ch);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Update the physical screen from the virtual screen.
    pub fn doupdate(&mut self) -> Result<()> {
        // Check if we need to clear the screen first
        let do_clear = self.stdscr.take_clear_flag();
        if do_clear {
            self.terminal.clear_screen()?;
            self.curscr.erase()?;
            self.curscr.touchwin();
        }

        let lines = self.newscr.getmaxy() as usize;
        let cols = self.newscr.getmaxx() as usize;

        // Collect changes first to avoid borrow issues
        #[cfg(not(feature = "wide"))]
        type CellData = (usize, usize, ChType);
        #[cfg(feature = "wide")]
        type CellData = (usize, usize, crate::wide::CCharT);

        let mut changes: Vec<CellData> = Vec::new();

        for y in 0..lines {
            let newscr_line = match self.newscr.line(y) {
                Some(l) => l,
                None => continue,
            };
            let curscr_line = match self.curscr.line(y) {
                Some(l) => l,
                None => continue,
            };

            // Check if line has changes
            if !newscr_line.is_touched() {
                continue;
            }

            if let Some((first, last)) = newscr_line.changed_range() {
                for x in first..=last.min(cols - 1) {
                    let new_cell = newscr_line.get(x);
                    let cur_cell = curscr_line.get(x);

                    if new_cell != cur_cell || do_clear {
                        changes.push((y, x, new_cell));
                    }
                }
            }
        }

        // Now output the changes
        let mut last_attr: AttrT = A_NORMAL;
        let mut current_y: i32 = -1;
        let mut current_x: i32 = -1;

        for (y, x, cell) in changes {
            // Move cursor if needed
            if current_y != y as i32 || current_x != x as i32 {
                self.terminal.move_cursor(y as i32, x as i32)?;
                current_y = y as i32;
                current_x = x as i32;
            }

            #[cfg(not(feature = "wide"))]
            {
                // Handle attributes
                let new_attr = cell & !A_CHARTEXT;
                if new_attr != last_attr {
                    self.output_attr(new_attr)?;
                    last_attr = new_attr;
                }

                // Output the character
                let c = (cell & A_CHARTEXT) as u8;
                if c >= 0x20 && c < 0x7f {
                    self.terminal.write(&[c])?;
                } else if c == 0 {
                    self.terminal.write(b" ")?;
                } else {
                    // Control character or high byte - output as space
                    self.terminal.write(b" ")?;
                }
            }

            #[cfg(feature = "wide")]
            {
                // Handle attributes
                let new_attr = cell.attrs();
                if new_attr != last_attr {
                    self.output_attr(new_attr)?;
                    last_attr = new_attr;
                }

                // Output the character
                let c = cell.spacing_char();
                if c == '\0' {
                    self.terminal.write(b" ")?;
                } else {
                    let mut buf = [0u8; 4];
                    let s = c.encode_utf8(&mut buf);
                    self.terminal.write(s.as_bytes())?;
                }
            }

            current_x += 1;
        }

        // Reset attributes
        if last_attr != A_NORMAL {
            self.terminal.set_attributes(A_NORMAL)?;
        }

        // Position cursor at stdscr's cursor position
        let cursor_y = self.stdscr.getcury();
        let cursor_x = self.stdscr.getcurx();
        if !self.stdscr.is_leaveok() {
            self.terminal.move_cursor(cursor_y, cursor_x)?;
        }

        // Flush output
        self.terminal.flush()?;

        // Copy newscr to curscr and clear touch flags
        for y in 0..lines {
            if let (Some(newscr_line), Some(curscr_line)) =
                (self.newscr.line(y), self.curscr.line_mut(y))
            {
                curscr_line.copy_from(newscr_line);
                curscr_line.untouch();
            }
            if let Some(newscr_line) = self.newscr.line_mut(y) {
                newscr_line.untouch();
            }
        }

        // Clear touch flags on stdscr
        self.stdscr.untouchwin();

        Ok(())
    }

    /// Output attribute changes to the terminal.
    fn output_attr(&mut self, attr: AttrT) -> Result<()> {
        // Set text attributes
        self.terminal.set_attributes(attr)?;

        // Handle color pair
        let pair = attr::pair_number(attr);
        if pair > 0 {
            if let Ok((fg, bg)) = self.colors.pair_content(pair) {
                self.terminal.set_fg_color(fg)?;
                self.terminal.set_bg_color(bg)?;
            }
        } else {
            // Reset to default colors
            self.terminal.set_fg_color(-1)?;
            self.terminal.set_bg_color(-1)?;
        }

        Ok(())
    }

    // ========================================================================
    // Input operations
    // ========================================================================

    /// Read a character from the terminal (using stdscr settings).
    pub fn getch(&mut self) -> Result<i32> {
        // If immedok is set, refresh first
        if self.stdscr.is_immedok() {
            self.refresh()?;
        }

        // Get delay setting from stdscr
        let delay = Delay::from_raw(self.stdscr.getdelay());
        let use_keypad = self.stdscr.is_keypad();

        self.getch_internal(delay, use_keypad)
    }

    /// Read a character from a window.
    pub fn wgetch(&mut self, win: &mut Window) -> Result<i32> {
        // If immedok is set, refresh first
        if win.is_immedok() {
            self.wrefresh(win)?;
        }

        // Get delay setting from window
        let delay = Delay::from_raw(win.getdelay());
        let use_keypad = win.is_keypad();

        self.getch_internal(delay, use_keypad)
    }

    /// Move cursor and read a character from stdscr.
    pub fn mvgetch(&mut self, y: i32, x: i32) -> Result<i32> {
        self.stdscr.mv(y, x)?;
        self.getch()
    }

    /// Move cursor and read a character from a window.
    pub fn mvwgetch(&mut self, win: &mut Window, y: i32, x: i32) -> Result<i32> {
        win.mv(y, x)?;
        self.wgetch(win)
    }

    /// Read a wide character from the terminal (using stdscr settings).
    ///
    /// Returns the wide character result, which can be a character or key code.
    /// This is the Rust equivalent of `get_wch()`.
    #[cfg(feature = "wide")]
    pub fn get_wch(&mut self) -> Result<crate::wide::WideInput> {
        // If immedok is set, refresh first
        if self.stdscr.is_immedok() {
            self.refresh()?;
        }

        let delay = Delay::from_raw(self.stdscr.getdelay());
        let use_keypad = self.stdscr.is_keypad();

        self.get_wch_internal(delay, use_keypad)
    }

    /// Read a wide character from a window.
    ///
    /// Returns the wide character result, which can be a character or key code.
    /// This is the Rust equivalent of `wget_wch()`.
    #[cfg(feature = "wide")]
    pub fn wget_wch(&mut self, win: &mut Window) -> Result<crate::wide::WideInput> {
        // If immedok is set, refresh first
        if win.is_immedok() {
            self.wrefresh(win)?;
        }

        let delay = Delay::from_raw(win.getdelay());
        let use_keypad = win.is_keypad();

        self.get_wch_internal(delay, use_keypad)
    }

    /// Move cursor and read a wide character from stdscr.
    #[cfg(feature = "wide")]
    pub fn mvget_wch(&mut self, y: i32, x: i32) -> Result<crate::wide::WideInput> {
        self.stdscr.mv(y, x)?;
        self.get_wch()
    }

    /// Move cursor and read a wide character from a window.
    #[cfg(feature = "wide")]
    pub fn mvwget_wch(
        &mut self,
        win: &mut Window,
        y: i32,
        x: i32,
    ) -> Result<crate::wide::WideInput> {
        win.mv(y, x)?;
        self.wget_wch(win)
    }

    /// Internal wide character reading logic.
    #[cfg(feature = "wide")]
    fn get_wch_internal(
        &mut self,
        delay: Delay,
        use_keypad: bool,
    ) -> Result<crate::wide::WideInput> {
        use crate::wide::WideInput;

        // First get a character using the normal getch
        let ch = match self.getch_internal(delay, use_keypad) {
            Ok(c) => c,
            Err(Error::Timeout) => return Ok(WideInput::None),
            Err(Error::NoInput) => return Ok(WideInput::None),
            Err(Error::Eof) => return Ok(WideInput::Eof),
            Err(e) => return Err(e),
        };

        // Check if it's a key code (high values or negative)
        if !(0..=255).contains(&ch) {
            return Ok(WideInput::Key(ch));
        }

        // Handle as UTF-8
        let first_byte = ch as u8;

        // Single-byte ASCII
        if first_byte < 0x80 {
            return Ok(WideInput::Char(first_byte as char));
        }

        // Multi-byte UTF-8 sequence
        let needed = if first_byte & 0xE0 == 0xC0 {
            1 // 2-byte sequence
        } else if first_byte & 0xF0 == 0xE0 {
            2 // 3-byte sequence
        } else if first_byte & 0xF8 == 0xF0 {
            3 // 4-byte sequence
        } else {
            // Invalid UTF-8 start byte, return as-is
            return Ok(WideInput::Char(first_byte as char));
        };

        let mut bytes = vec![first_byte];

        // Read continuation bytes
        for _ in 0..needed {
            // Use a short timeout for continuation bytes
            let timeout = Delay::Timeout(50);
            match self.getch_internal(timeout, false) {
                Ok(b) if (b as u8) & 0xC0 == 0x80 => {
                    bytes.push(b as u8);
                }
                Ok(b) => {
                    // Not a continuation byte, push it back
                    self.input_buffer.unget(b);
                    break;
                }
                Err(_) => break,
            }
        }

        // Try to decode as UTF-8
        match std::str::from_utf8(&bytes) {
            Ok(s) => {
                if let Some(c) = s.chars().next() {
                    Ok(WideInput::Char(c))
                } else {
                    Ok(WideInput::Error)
                }
            }
            Err(_) => {
                // Invalid UTF-8, return first byte as char
                Ok(WideInput::Char(first_byte as char))
            }
        }
    }

    /// Internal character reading logic.
    fn getch_internal(&mut self, delay: Delay, use_keypad: bool) -> Result<i32> {
        // Check input buffer first
        if let Some(ch) = self.input_buffer.get() {
            return Ok(ch);
        }

        // Determine timeout
        let timeout = match delay {
            Delay::NoDelay => Some(Duration::ZERO),
            Delay::Blocking => {
                if self.input_mode.is_halfdelay() {
                    Some(Duration::from_millis(
                        (self.input_mode.halfdelay_tenths() * 100) as u64,
                    ))
                } else {
                    None // Block indefinitely
                }
            }
            Delay::Timeout(ms) => Some(Duration::from_millis(ms as u64)),
        };

        let start = Instant::now();

        loop {
            // Check if input is available
            if !self.terminal.has_input() {
                // For NoDelay mode, return immediately if no input
                if timeout == Some(Duration::ZERO) {
                    return Err(Error::NoInput);
                }
                // Check for timeout (only for non-zero timeouts)
                if let Some(t) = timeout {
                    if start.elapsed() >= t {
                        return Err(Error::Timeout);
                    }
                }
                // Brief sleep to avoid busy waiting
                std::thread::sleep(Duration::from_millis(1));
                continue;
            }

            // Read a byte
            let byte = match self.terminal.read_byte()? {
                Some(b) => b,
                None => return Err(Error::Eof),
            };

            // Handle escape sequences if keypad mode is enabled
            if use_keypad && byte == 0x1b {
                // Start escape sequence parsing
                self.escape_parser.reset();
                let result = self.parse_escape_sequence()?;
                return Ok(result);
            }

            // Handle newline translation
            if self.input_mode.nl && byte == b'\r' {
                return Ok(b'\n' as i32);
            }

            return Ok(byte as i32);
        }
    }

    /// Parse an escape sequence after receiving ESC.
    fn parse_escape_sequence(&mut self) -> Result<i32> {
        self.escape_parser.reset();
        self.escape_parser.feed(0x1b);

        let start = Instant::now();
        let escape_timeout = Duration::from_millis(self.escdelay as u64);

        // Buffer to accumulate the sequence for mouse parsing
        let mut sequence_buf: Vec<u8> = vec![0x1b];

        loop {
            // Check timeout
            if start.elapsed() >= escape_timeout {
                // Timeout - return the accumulated input
                let input = self.escape_parser.current_input();
                if input.len() == 1 {
                    // Just ESC
                    return Ok(0x1b);
                }
                // Return current match if any, otherwise just ESC
                if let Some(key) = self.escape_parser.current_match() {
                    return Ok(key);
                }
                // Push remaining bytes back to buffer (except ESC which we return)
                for &b in &input[1..] {
                    self.input_buffer.push(b as i32);
                }
                return Ok(0x1b);
            }

            // Check for more input
            if !self.terminal.has_input() {
                std::thread::sleep(Duration::from_millis(1));
                continue;
            }

            let byte = match self.terminal.read_byte()? {
                Some(b) => b,
                None => {
                    // EOF during escape - return what we have
                    if let Some(key) = self.escape_parser.current_match() {
                        return Ok(key);
                    }
                    return Ok(0x1b);
                }
            };

            sequence_buf.push(byte);

            // Check for SGR mouse sequence: \x1b[<...M or \x1b[<...m
            #[cfg(feature = "mouse")]
            if self.mouse.is_enabled() && is_mouse_prefix(&sequence_buf) {
                // Check if we have a complete mouse sequence
                if sequence_buf.len() >= 3 && &sequence_buf[0..3] == b"\x1b[<" {
                    // Look for terminator
                    if byte == b'M' || byte == b'm' {
                        // Complete mouse sequence
                        if let Some(event) = parse_sgr_mouse(&sequence_buf) {
                            self.mouse.push_event(event);
                            return Ok(KEY_MOUSE);
                        }
                    }
                    // Continue accumulating if not complete
                    if sequence_buf.len() < 20 {
                        // Reasonable max length for mouse sequence
                        continue;
                    }
                }
            }

            match self.escape_parser.feed(byte) {
                EscapeMatch::Complete(key) => {
                    return Ok(key);
                }
                EscapeMatch::None => {
                    // No match - return ESC and push rest of sequence_buf to buffer
                    // Note: We use sequence_buf here because the parser clears its
                    // internal state when returning None, making current_input() empty.
                    // sequence_buf[0] is ESC which we return, so push [1..] to buffer.
                    for &b in &sequence_buf[1..] {
                        self.input_buffer.push(b as i32);
                    }
                    return Ok(0x1b);
                }
                EscapeMatch::Partial => {
                    // Continue reading
                }
            }
        }
    }

    /// Push a character back into the input buffer.
    pub fn ungetch(&mut self, ch: i32) -> Result<()> {
        if self.input_buffer.unget(ch) {
            Ok(())
        } else {
            Err(Error::BufferFull)
        }
    }

    /// Push a wide character back into the input buffer.
    ///
    /// This converts the wide character to its UTF-8 representation and
    /// pushes each byte back in reverse order so they will be read in
    /// the correct order.
    #[cfg(feature = "wide")]
    pub fn unget_wch(&mut self, wch: char) -> Result<()> {
        let mut buf = [0u8; 4];
        let encoded = wch.encode_utf8(&mut buf);
        let bytes = encoded.as_bytes();

        // Push bytes in reverse order
        for &byte in bytes.iter().rev() {
            if !self.input_buffer.unget(byte as i32) {
                return Err(Error::BufferFull);
            }
        }
        Ok(())
    }

    /// Check if there's typeahead input.
    pub fn has_key(&self) -> bool {
        self.input_buffer.has_input() || self.terminal.has_input()
    }

    /// Flush the input buffer.
    pub fn flushinp(&mut self) {
        self.input_buffer.clear();
        // Also try to drain the terminal input
        while self.terminal.has_input() {
            let _ = self.terminal.read_byte();
        }
    }

    /// Get a string from the user with simple line editing (using stdscr).
    pub fn getstr(&mut self, maxlen: usize) -> Result<String> {
        let mut result = String::new();
        let echo_enabled = self.input_mode.echo;

        // Get delay and keypad settings from stdscr
        let delay = Delay::from_raw(self.stdscr.getdelay());
        let use_keypad = self.stdscr.is_keypad();

        loop {
            let ch = self.getch_internal(delay, use_keypad)?;

            match ch {
                // Enter/Return
                0x0a | 0x0d => {
                    if echo_enabled {
                        self.stdscr.addch(b'\n' as ChType)?;
                        self.refresh()?;
                    }
                    break;
                }
                // Backspace
                0x08 | 0x7f => {
                    if !result.is_empty() {
                        result.pop();
                        if echo_enabled {
                            let (y, x) = (self.stdscr.getcury(), self.stdscr.getcurx());
                            if x > 0 {
                                self.stdscr.mv(y, x - 1)?;
                                self.stdscr.addch(b' ' as ChType)?;
                                self.stdscr.mv(y, x - 1)?;
                                self.refresh()?;
                            }
                        }
                    }
                }
                // Regular character
                _ if (0x20..0x7f).contains(&ch) => {
                    if result.len() < maxlen {
                        result.push(ch as u8 as char);
                        if echo_enabled {
                            self.stdscr.addch(ch as ChType)?;
                            self.refresh()?;
                        }
                    }
                }
                // Control-C, Control-D - cancel
                0x03 | 0x04 => {
                    return Err(Error::Interrupted);
                }
                _ => {}
            }
        }

        Ok(result)
    }

    /// Get a string from a window with simple line editing.
    pub fn wgetstr(&mut self, win: &mut Window, maxlen: usize) -> Result<String> {
        let mut result = String::new();
        let echo_enabled = self.input_mode.echo;

        // Get delay and keypad settings from window
        let delay = Delay::from_raw(win.getdelay());
        let use_keypad = win.is_keypad();

        loop {
            let ch = self.getch_internal(delay, use_keypad)?;

            match ch {
                // Enter/Return
                0x0a | 0x0d => {
                    if echo_enabled {
                        win.addch(b'\n' as ChType)?;
                        self.wrefresh(win)?;
                    }
                    break;
                }
                // Backspace
                0x08 | 0x7f => {
                    if !result.is_empty() {
                        result.pop();
                        if echo_enabled {
                            let (y, x) = (win.getcury(), win.getcurx());
                            if x > 0 {
                                win.mv(y, x - 1)?;
                                win.addch(b' ' as ChType)?;
                                win.mv(y, x - 1)?;
                                self.wrefresh(win)?;
                            }
                        }
                    }
                }
                // Regular character
                _ if (0x20..0x7f).contains(&ch) => {
                    if result.len() < maxlen {
                        result.push(ch as u8 as char);
                        if echo_enabled {
                            win.addch(ch as ChType)?;
                            self.wrefresh(win)?;
                        }
                    }
                }
                // Control-C, Control-D - cancel
                0x03 | 0x04 => {
                    return Err(Error::Interrupted);
                }
                _ => {}
            }
        }

        Ok(result)
    }

    // ========================================================================
    // Convenience methods for stdscr operations
    // ========================================================================

    /// Move cursor to position (y, x) in stdscr.
    pub fn mv(&mut self, y: i32, x: i32) -> Result<()> {
        self.stdscr.mv(y, x)
    }

    /// Add a character at the current cursor position in stdscr.
    pub fn addch(&mut self, ch: ChType) -> Result<()> {
        self.stdscr.addch(ch)
    }

    /// Add a string at the current cursor position in stdscr.
    pub fn addstr(&mut self, s: &str) -> Result<()> {
        self.stdscr.addstr(s)
    }

    /// Move to (y, x) and add a character in stdscr.
    pub fn mvaddch(&mut self, y: i32, x: i32, ch: ChType) -> Result<()> {
        self.stdscr.mv(y, x)?;
        self.stdscr.addch(ch)
    }

    /// Move to (y, x) and add a string in stdscr.
    pub fn mvaddstr(&mut self, y: i32, x: i32, s: &str) -> Result<()> {
        self.stdscr.mv(y, x)?;
        self.stdscr.addstr(s)
    }

    /// Add a wide string at the current cursor position in stdscr.
    ///
    /// This is the Rust equivalent of `addwstr()`.
    #[cfg(feature = "wide")]
    pub fn addwstr(&mut self, s: &str) -> Result<()> {
        self.stdscr.addwstr(s)
    }

    /// Add a wide string with a maximum length in stdscr.
    ///
    /// This is the Rust equivalent of `addnwstr()`.
    #[cfg(feature = "wide")]
    pub fn addnwstr(&mut self, s: &str, n: i32) -> Result<()> {
        self.stdscr.addnwstr(s, n)
    }

    /// Move to (y, x) and add a wide string in stdscr.
    #[cfg(feature = "wide")]
    pub fn mvaddwstr(&mut self, y: i32, x: i32, s: &str) -> Result<()> {
        self.stdscr.mvaddwstr(y, x, s)
    }

    /// Move to (y, x) and add a wide string with a maximum length in stdscr.
    #[cfg(feature = "wide")]
    pub fn mvaddnwstr(&mut self, y: i32, x: i32, s: &str, n: i32) -> Result<()> {
        self.stdscr.mvaddnwstr(y, x, s, n)
    }

    /// Add a wide string at the current cursor position in a window.
    ///
    /// This is the Rust equivalent of `waddwstr()`.
    #[cfg(feature = "wide")]
    pub fn waddwstr(&mut self, win: &mut Window, s: &str) -> Result<()> {
        win.addwstr(s)
    }

    /// Add a wide string with a maximum length in a window.
    ///
    /// This is the Rust equivalent of `waddnwstr()`.
    #[cfg(feature = "wide")]
    pub fn waddnwstr(&mut self, win: &mut Window, s: &str, n: i32) -> Result<()> {
        win.addnwstr(s, n)
    }

    /// Move cursor and add a wide string in a window.
    #[cfg(feature = "wide")]
    pub fn mvwaddwstr(&mut self, win: &mut Window, y: i32, x: i32, s: &str) -> Result<()> {
        win.mvaddwstr(y, x, s)
    }

    /// Move cursor and add a wide string with a maximum length in a window.
    #[cfg(feature = "wide")]
    pub fn mvwaddnwstr(&mut self, win: &mut Window, y: i32, x: i32, s: &str, n: i32) -> Result<()> {
        win.mvaddnwstr(y, x, s, n)
    }

    /// Turn on attributes in stdscr.
    pub fn attron(&mut self, attr: AttrT) -> Result<()> {
        self.stdscr.attron(attr)
    }

    /// Turn off attributes in stdscr.
    pub fn attroff(&mut self, attr: AttrT) -> Result<()> {
        self.stdscr.attroff(attr)
    }

    /// Set attributes in stdscr.
    pub fn attrset(&mut self, attr: AttrT) -> Result<()> {
        self.stdscr.attrset(attr)
    }

    /// Clear the stdscr.
    pub fn clear(&mut self) -> Result<()> {
        self.stdscr.clear()
    }

    /// Erase the stdscr.
    pub fn erase(&mut self) -> Result<()> {
        self.stdscr.erase()
    }

    /// Clear to end of line in stdscr.
    pub fn clrtoeol(&mut self) -> Result<()> {
        self.stdscr.clrtoeol()
    }

    /// Clear to bottom of screen in stdscr.
    pub fn clrtobot(&mut self) -> Result<()> {
        self.stdscr.clrtobot()
    }

    /// Draw a border around stdscr.
    #[allow(clippy::too_many_arguments)]
    pub fn border(
        &mut self,
        ls: ChType,
        rs: ChType,
        ts: ChType,
        bs: ChType,
        tl: ChType,
        tr: ChType,
        bl: ChType,
        br: ChType,
    ) -> Result<()> {
        self.stdscr.border(ls, rs, ts, bs, tl, tr, bl, br)
    }

    /// Draw a box around stdscr using default characters.
    pub fn r#box(&mut self, verch: ChType, horch: ChType) -> Result<()> {
        self.stdscr.box_(verch, horch)
    }

    /// Draw a horizontal line in stdscr.
    pub fn hline(&mut self, ch: ChType, n: i32) -> Result<()> {
        self.stdscr.hline(ch, n)
    }

    /// Draw a vertical line in stdscr.
    pub fn vline(&mut self, ch: ChType, n: i32) -> Result<()> {
        self.stdscr.vline(ch, n)
    }

    /// Get current cursor Y position in stdscr.
    pub fn getcury(&self) -> i32 {
        self.stdscr.getcury()
    }

    /// Get current cursor X position in stdscr.
    pub fn getcurx(&self) -> i32 {
        self.stdscr.getcurx()
    }

    /// Get current cursor position in stdscr (y, x).
    pub fn getyx(&self) -> (i32, i32) {
        (self.stdscr.getcury(), self.stdscr.getcurx())
    }

    /// Get maximum Y coordinate in stdscr.
    pub fn getmaxy(&self) -> i32 {
        self.stdscr.getmaxy()
    }

    /// Get maximum X coordinate in stdscr.
    pub fn getmaxx(&self) -> i32 {
        self.stdscr.getmaxx()
    }

    /// Enable scrolling in stdscr.
    pub fn scrollok(&mut self, bf: bool) {
        self.stdscr.scrollok(bf);
    }

    /// Set scrolling region in stdscr.
    pub fn setscrreg(&mut self, top: i32, bot: i32) -> Result<()> {
        self.stdscr.setscrreg(top, bot)
    }

    /// Scroll stdscr by n lines.
    pub fn scroll(&mut self, n: i32) -> Result<()> {
        self.stdscr.scrl(n)
    }

    /// Set keypad mode in stdscr.
    pub fn keypad(&mut self, bf: bool) {
        self.stdscr.keypad(bf);
    }

    /// Set nodelay mode in stdscr.
    pub fn nodelay(&mut self, bf: bool) {
        self.stdscr.nodelay(bf);
    }

    /// Set timeout for input in stdscr.
    pub fn timeout(&mut self, delay: i32) {
        self.stdscr.timeout(delay);
    }

    /// Insert a character at the current position in stdscr.
    pub fn insch(&mut self, ch: ChType) -> Result<()> {
        self.stdscr.insch(ch)
    }

    /// Insert a string at the current position in stdscr.
    ///
    /// Characters to the right are shifted right; characters shifted off
    /// the right edge are lost. The cursor position does not change.
    pub fn insstr(&mut self, s: &str) -> Result<()> {
        self.stdscr.insstr(s)
    }

    /// Insert at most n characters of a string at the current position in stdscr.
    ///
    /// If n is negative, the entire string is inserted.
    pub fn insnstr(&mut self, s: &str, n: i32) -> Result<()> {
        self.stdscr.insnstr(s, n)
    }

    /// Move cursor and insert a string in stdscr.
    pub fn mvinsstr(&mut self, y: i32, x: i32, s: &str) -> Result<()> {
        self.stdscr.mvinsstr(y, x, s)
    }

    /// Move cursor and insert at most n characters in stdscr.
    pub fn mvinsnstr(&mut self, y: i32, x: i32, s: &str, n: i32) -> Result<()> {
        self.stdscr.mvinsnstr(y, x, s, n)
    }

    /// Insert a string at the current position in a window.
    pub fn winsstr(&mut self, win: &mut Window, s: &str) -> Result<()> {
        win.insstr(s)
    }

    /// Insert at most n characters of a string in a window.
    pub fn winsnstr(&mut self, win: &mut Window, s: &str, n: i32) -> Result<()> {
        win.insnstr(s, n)
    }

    /// Move cursor and insert a string in a window.
    pub fn mvwinsstr(&mut self, win: &mut Window, y: i32, x: i32, s: &str) -> Result<()> {
        win.mvinsstr(y, x, s)
    }

    /// Move cursor and insert at most n characters in a window.
    pub fn mvwinsnstr(&mut self, win: &mut Window, y: i32, x: i32, s: &str, n: i32) -> Result<()> {
        win.mvinsnstr(y, x, s, n)
    }

    /// Delete the character at the current position in stdscr.
    pub fn delch(&mut self) -> Result<()> {
        self.stdscr.delch()
    }

    /// Insert a blank line above the current line in stdscr.
    pub fn insertln(&mut self) -> Result<()> {
        self.stdscr.insertln()
    }

    /// Delete the current line in stdscr.
    pub fn deleteln(&mut self) -> Result<()> {
        self.stdscr.deleteln()
    }

    /// Set background character in stdscr.
    pub fn bkgdset(&mut self, ch: ChType) {
        self.stdscr.bkgdset(ch);
    }

    /// Set and apply background to stdscr.
    pub fn bkgd(&mut self, ch: ChType) -> Result<()> {
        self.stdscr.bkgd(ch)
    }

    /// Print a string to stdscr (simplified version of C's printw).
    ///
    /// **Note:** Unlike C ncurses, this does not support printf-style format strings.
    /// Use Rust's `format!` macro to format your string before calling this:
    ///
    /// ```rust,ignore
    /// // C ncurses: printw("Value: %d", value);
    /// // Rust:
    /// screen.printw(&format!("Value: {}", value))?;
    /// ```
    pub fn printw(&mut self, s: &str) -> Result<()> {
        self.stdscr.addstr(s)
    }

    /// Move cursor and print a string to stdscr.
    ///
    /// **Note:** Unlike C ncurses, this does not support printf-style format strings.
    /// Use Rust's `format!` macro:
    ///
    /// ```rust,ignore
    /// // C ncurses: mvprintw(y, x, "Value: %d", value);
    /// // Rust:
    /// screen.mvprintw(y, x, &format!("Value: {}", value))?;
    /// ```
    pub fn mvprintw(&mut self, y: i32, x: i32, s: &str) -> Result<()> {
        self.stdscr.mv(y, x)?;
        self.stdscr.addstr(s)
    }

    /// Print a string to a window (simplified version of C's wprintw).
    ///
    /// **Note:** Unlike C ncurses, this does not support printf-style format strings.
    /// Use Rust's `format!` macro:
    ///
    /// ```rust,ignore
    /// // C ncurses: wprintw(win, "Value: %d", value);
    /// // Rust:
    /// screen.wprintw(win, &format!("Value: {}", value))?;
    /// ```
    pub fn wprintw(&mut self, win: &mut Window, s: &str) -> Result<()> {
        win.addstr(s)
    }

    /// Move cursor and print a string to a window.
    ///
    /// **Note:** Unlike C ncurses, this does not support printf-style format strings.
    /// Use Rust's `format!` macro:
    ///
    /// ```rust,ignore
    /// // C ncurses: mvwprintw(win, y, x, "Value: %d", value);
    /// // Rust:
    /// screen.mvwprintw(win, y, x, &format!("Value: {}", value))?;
    /// ```
    pub fn mvwprintw(&mut self, win: &mut Window, y: i32, x: i32, s: &str) -> Result<()> {
        win.mv(y, x)?;
        win.addstr(s)
    }

    // ========================================================================
    // Additional ncurses compatibility functions
    // ========================================================================

    /// Sleep for the specified number of milliseconds.
    ///
    /// This is the ncurses `napms()` function.
    pub fn napms(&self, ms: i32) {
        if ms > 0 {
            std::thread::sleep(Duration::from_millis(ms as u64));
        }
    }

    /// Output padding characters to delay for the specified number of milliseconds.
    ///
    /// This function inserts a delay in the output stream by outputting padding
    /// characters. The actual delay may vary depending on terminal baud rate.
    ///
    /// In this implementation, we simply use a sleep since modern terminals
    /// don't need padding characters.
    pub fn delay_output(&mut self, ms: i32) -> Result<()> {
        if ms > 0 {
            self.terminal.flush()?;
            std::thread::sleep(Duration::from_millis(ms as u64));
        }
        Ok(())
    }

    /// Save the current program terminal mode.
    ///
    /// This saves the current terminal settings so they can be restored
    /// later with `reset_prog_mode()`.
    pub fn def_prog_mode(&mut self) -> Result<()> {
        self.terminal.save_prog_mode()
    }

    /// Save the current shell terminal mode.
    ///
    /// This saves the current terminal settings (typically the shell's settings)
    /// so they can be restored later with `reset_shell_mode()`.
    pub fn def_shell_mode(&mut self) -> Result<()> {
        self.terminal.save_shell_mode()
    }

    /// Restore the saved program terminal mode.
    ///
    /// This restores the terminal settings saved by `def_prog_mode()`.
    pub fn reset_prog_mode(&mut self) -> Result<()> {
        self.terminal.restore_prog_mode()
    }

    /// Restore the saved shell terminal mode.
    ///
    /// This restores the terminal settings saved by `def_shell_mode()`.
    pub fn reset_shell_mode(&mut self) -> Result<()> {
        self.terminal.restore_shell_mode()
    }

    /// Reset the terminal to program mode after shell escape.
    ///
    /// Equivalent to calling `reset_prog_mode()` followed by refreshing.
    pub fn resetty(&mut self) -> Result<()> {
        self.terminal.restore_prog_mode()?;
        self.refresh()
    }

    /// Save the terminal state for later restoration.
    ///
    /// Equivalent to calling `def_prog_mode()`.
    pub fn savetty(&mut self) -> Result<()> {
        self.terminal.save_prog_mode()
    }

    /// Check if the terminal has insert/delete character capabilities.
    ///
    /// Returns true if the terminal supports inserting and deleting characters.
    /// This is based on the terminal type detection.
    pub fn has_ic(&self) -> bool {
        self.terminal.has_ic()
    }

    /// Check if the terminal has insert/delete line capabilities.
    ///
    /// Returns true if the terminal supports inserting and deleting lines.
    /// This is based on the terminal type detection.
    pub fn has_il(&self) -> bool {
        self.terminal.has_il()
    }

    /// Get the kill character (line kill).
    pub fn killchar(&self) -> char {
        crate::terminal::killchar()
    }

    /// Get the erase character (backspace).
    pub fn erasechar(&self) -> char {
        crate::terminal::erasechar()
    }

    /// Get the erase character as a wide character.
    ///
    /// This is the ncurses `erasewchar()` function. It returns the terminal's
    /// erase character as a wide character via the output parameter.
    ///
    /// # Arguments
    /// * `ch` - Output parameter to receive the erase character
    ///
    /// # Returns
    /// `Ok(())` on success
    #[cfg(feature = "wide")]
    pub fn erasewchar(&self, ch: &mut char) -> Result<()> {
        *ch = crate::terminal::erasechar();
        Ok(())
    }

    /// Get a longer terminal description.
    ///
    /// In ncurses this returns the terminfo description, but we return
    /// a simple description based on the terminal type.
    pub fn longname(&self) -> String {
        let term = self.terminal.term_type();
        match term {
            "xterm" => "xterm terminal emulator".to_string(),
            "xterm-256color" => "xterm with 256 colors".to_string(),
            "screen" => "GNU Screen".to_string(),
            "screen-256color" => "GNU Screen with 256 colors".to_string(),
            "tmux" => "tmux terminal multiplexer".to_string(),
            "tmux-256color" => "tmux with 256 colors".to_string(),
            "linux" => "Linux console".to_string(),
            "dumb" => "dumb terminal".to_string(),
            _ => format!("{} terminal", term),
        }
    }

    /// Check if filter mode is enabled.
    ///
    /// Filter mode restricts curses to a single line, turning off cursor
    /// movement capabilities. This is typically used for simple line-editing
    /// applications.
    pub fn isfilter(&self) -> bool {
        self.filtered
    }

    /// Enable filter mode.
    ///
    /// When filter mode is enabled, the terminal is treated as a single-line
    /// terminal (LINES=1), and cursor movement functions are disabled.
    /// This should be called before or immediately after screen initialization.
    pub fn filter(&mut self) {
        self.filtered = true;
        // In filter mode, we only use one line
        // Resize the screen to 1 line
        let cols = self.terminal.columns();
        let _ = self.resize_term(1, cols);
    }

    /// Disable filter mode.
    ///
    /// This restores normal terminal operation with full screen capabilities.
    pub fn nofilter(&mut self) {
        if self.filtered {
            self.filtered = false;
            // Restore full terminal size
            let _ = self.terminal.update_size();
            let lines = self.terminal.lines();
            let cols = self.terminal.columns();
            let _ = self.resize_term(lines, cols);
        }
    }

    /// Check if the standard screen has clearok mode enabled.
    ///
    /// When clearok is enabled, the next refresh will clear the screen
    /// and redraw from scratch.
    pub fn is_cleared(&self) -> bool {
        self.stdscr.is_cleared()
    }

    /// Check if the standard screen is a pad window.
    ///
    /// The standard screen is never a pad, so this always returns false.
    pub fn is_pad(&self) -> bool {
        self.stdscr.is_pad()
    }

    /// Check if we're using the standard screen.
    pub fn is_wintouched(&self) -> bool {
        self.stdscr.is_wintouched()
    }

    /// Control flushing of input queue on interrupt.
    ///
    /// If `enable` is true (the default), when an interrupt key is pressed,
    /// all input in the queue will be flushed.
    pub fn intrflush(&mut self, enable: bool) -> Result<()> {
        self.terminal.intrflush(enable)
    }

    /// Enable 8-bit input mode.
    ///
    /// If `enable` is true, the terminal passes 8-bit characters through
    /// without stripping the high bit.
    pub fn meta(&mut self, enable: bool) -> Result<()> {
        self.terminal.meta(enable)
    }

    /// Set the file descriptor for typeahead checking.
    ///
    /// If `fd` is -1, no typeahead checking is done. The default is stdin (0).
    /// This controls whether curses checks for pending input before output.
    pub fn typeahead(&mut self, fd: i32) {
        self.terminal.set_typeahead_fd(fd);
    }

    /// Enable flushing of output and input queues on interrupt.
    ///
    /// When enabled (the default), pressing an interrupt key will flush
    /// both the input and output queues.
    pub fn qiflush(&mut self) {
        let _ = self.terminal.qiflush(true);
    }

    /// Disable flushing of output and input queues on interrupt.
    ///
    /// When disabled, pressing an interrupt key will not flush the queues.
    pub fn noqiflush(&mut self) {
        let _ = self.terminal.qiflush(false);
    }

    // ========================================================================
    // Window wrapper functions (w* prefix functions for ncurses compatibility)
    // ========================================================================

    /// Add a character to a window at current cursor position.
    pub fn waddch(&mut self, win: &mut Window, ch: ChType) -> Result<()> {
        win.addch(ch)
    }

    /// Add a character to a window at specified position.
    pub fn mvwaddch(&mut self, win: &mut Window, y: i32, x: i32, ch: ChType) -> Result<()> {
        win.mvaddch(y, x, ch)
    }

    /// Add a character and refresh immediately (stdscr).
    ///
    /// Equivalent to calling `addch()` followed by `refresh()`.
    pub fn echochar(&mut self, ch: ChType) -> Result<()> {
        self.stdscr.addch(ch)?;
        self.refresh()
    }

    /// Add a character to a window and refresh immediately.
    ///
    /// Equivalent to calling `waddch()` followed by `wrefresh()`.
    pub fn wechochar(&mut self, win: &mut Window, ch: ChType) -> Result<()> {
        win.addch(ch)?;
        self.wrefresh(win)
    }

    /// Add a wide character and refresh immediately (stdscr).
    ///
    /// Equivalent to calling `add_wch()` followed by `refresh()`.
    #[cfg(feature = "wide")]
    pub fn echo_wchar(&mut self, wch: &crate::wide::CCharT) -> Result<()> {
        self.stdscr.add_wch(wch)?;
        self.refresh()
    }

    /// Add a wide character to a window and refresh immediately.
    ///
    /// Equivalent to calling `wadd_wch()` followed by `wrefresh()`.
    #[cfg(feature = "wide")]
    pub fn wecho_wchar(&mut self, win: &mut Window, wch: &crate::wide::CCharT) -> Result<()> {
        win.add_wch(wch)?;
        self.wrefresh(win)
    }

    /// Add a wide character to a pad and refresh immediately.
    ///
    /// Equivalent to calling `add_wch()` on the pad followed by `prefresh()`.
    /// The pad must have had `prefresh` or `pnoutrefresh` called at least once
    /// to establish display parameters.
    #[cfg(feature = "wide")]
    pub fn pecho_wchar(&mut self, pad: &mut Window, wch: &crate::wide::CCharT) -> Result<()> {
        if !pad.is_pad() {
            return Err(Error::InvalidArgument("window is not a pad".into()));
        }

        pad.add_wch(wch)?;

        // Get the stored pad parameters
        let pad_data = pad
            .pad_data()
            .ok_or_else(|| Error::InvalidArgument("pad has no stored parameters".into()))?;

        let pminrow = pad_data.pad_y as i32;
        let pmincol = pad_data.pad_x as i32;
        let sminrow = pad_data.pad_top as i32;
        let smincol = pad_data.pad_left as i32;
        let smaxrow = pad_data.pad_bottom as i32;
        let smaxcol = pad_data.pad_right as i32;

        self.prefresh(pad, pminrow, pmincol, sminrow, smincol, smaxrow, smaxcol)
    }

    /// Add a string to a window at current cursor position.
    pub fn waddstr(&mut self, win: &mut Window, s: &str) -> Result<()> {
        win.addstr(s)
    }

    /// Add at most n characters of a string to a window.
    pub fn waddnstr(&mut self, win: &mut Window, s: &str, n: i32) -> Result<()> {
        win.addnstr(s, n)
    }

    /// Add a string to a window at specified position.
    pub fn mvwaddstr(&mut self, win: &mut Window, y: i32, x: i32, s: &str) -> Result<()> {
        win.mvaddstr(y, x, s)
    }

    /// Add at most n characters of a string to a window at specified position.
    pub fn mvwaddnstr(&mut self, win: &mut Window, y: i32, x: i32, s: &str, n: i32) -> Result<()> {
        win.mvaddnstr(y, x, s, n)
    }

    /// Add a character string to a window.
    pub fn waddchstr(&mut self, win: &mut Window, chstr: &[ChType]) -> Result<()> {
        win.addchstr(chstr)
    }

    /// Add at most n characters from a character string to a window.
    pub fn waddchnstr(&mut self, win: &mut Window, chstr: &[ChType], n: i32) -> Result<()> {
        win.addchnstr(chstr, n)
    }

    // ========================================================================
    // Wide character window wrapper functions
    // ========================================================================

    /// Add a wide character to stdscr at the current position.
    ///
    /// This is the ncurses `add_wch()` function.
    #[cfg(feature = "wide")]
    pub fn add_wch(&mut self, wch: &crate::wide::CCharT) -> Result<()> {
        self.stdscr.add_wch(wch)
    }

    /// Add a wide character to a window at the current position.
    ///
    /// This is the ncurses `wadd_wch()` function.
    #[cfg(feature = "wide")]
    pub fn wadd_wch(&mut self, win: &mut Window, wch: &crate::wide::CCharT) -> Result<()> {
        win.add_wch(wch)
    }

    /// Move cursor and add a wide character to stdscr.
    ///
    /// This is the ncurses `mvadd_wch()` function.
    #[cfg(feature = "wide")]
    pub fn mvadd_wch(&mut self, y: i32, x: i32, wch: &crate::wide::CCharT) -> Result<()> {
        self.stdscr.mvadd_wch(y, x, wch)
    }

    /// Move cursor and add a wide character to a window.
    ///
    /// This is the ncurses `mvwadd_wch()` function.
    #[cfg(feature = "wide")]
    pub fn mvwadd_wch(
        &mut self,
        win: &mut Window,
        y: i32,
        x: i32,
        wch: &crate::wide::CCharT,
    ) -> Result<()> {
        win.mvadd_wch(y, x, wch)
    }

    /// Add a wide character string (array of cchar_t) to stdscr.
    ///
    /// This is the ncurses `add_wchstr()` function.
    #[cfg(feature = "wide")]
    pub fn add_wchstr(&mut self, wchstr: &[crate::wide::CCharT]) -> Result<()> {
        self.stdscr.add_wchstr(wchstr)
    }

    /// Add a wide character string (array of cchar_t) to a window.
    ///
    /// This is the ncurses `wadd_wchstr()` function.
    #[cfg(feature = "wide")]
    pub fn wadd_wchstr(&mut self, win: &mut Window, wchstr: &[crate::wide::CCharT]) -> Result<()> {
        win.add_wchstr(wchstr)
    }

    /// Add at most n wide characters from a string to stdscr.
    ///
    /// This is the ncurses `add_wchnstr()` function.
    #[cfg(feature = "wide")]
    pub fn add_wchnstr(&mut self, wchstr: &[crate::wide::CCharT], n: i32) -> Result<()> {
        self.stdscr.add_wchnstr(wchstr, n)
    }

    /// Add at most n wide characters from a string to a window.
    ///
    /// This is the ncurses `wadd_wchnstr()` function.
    #[cfg(feature = "wide")]
    pub fn wadd_wchnstr(
        &mut self,
        win: &mut Window,
        wchstr: &[crate::wide::CCharT],
        n: i32,
    ) -> Result<()> {
        win.add_wchnstr(wchstr, n)
    }

    /// Move cursor and add a wide character string to stdscr.
    ///
    /// This is the ncurses `mvadd_wchstr()` function.
    #[cfg(feature = "wide")]
    pub fn mvadd_wchstr(&mut self, y: i32, x: i32, wchstr: &[crate::wide::CCharT]) -> Result<()> {
        self.stdscr.mvadd_wchstr(y, x, wchstr)
    }

    /// Move cursor and add at most n wide characters to stdscr.
    ///
    /// This is the ncurses `mvadd_wchnstr()` function.
    #[cfg(feature = "wide")]
    pub fn mvadd_wchnstr(
        &mut self,
        y: i32,
        x: i32,
        wchstr: &[crate::wide::CCharT],
        n: i32,
    ) -> Result<()> {
        self.stdscr.mvadd_wchnstr(y, x, wchstr, n)
    }

    /// Move cursor and add a wide character string to a window.
    ///
    /// This is the ncurses `mvwadd_wchstr()` function.
    #[cfg(feature = "wide")]
    pub fn mvwadd_wchstr(
        &mut self,
        win: &mut Window,
        y: i32,
        x: i32,
        wchstr: &[crate::wide::CCharT],
    ) -> Result<()> {
        win.mvadd_wchstr(y, x, wchstr)
    }

    /// Move cursor and add at most n wide characters to a window.
    ///
    /// This is the ncurses `mvwadd_wchnstr()` function.
    #[cfg(feature = "wide")]
    pub fn mvwadd_wchnstr(
        &mut self,
        win: &mut Window,
        y: i32,
        x: i32,
        wchstr: &[crate::wide::CCharT],
        n: i32,
    ) -> Result<()> {
        win.mvadd_wchnstr(y, x, wchstr, n)
    }

    /// Move cursor in a window.
    pub fn wmove(&mut self, win: &mut Window, y: i32, x: i32) -> Result<()> {
        win.mv(y, x)
    }

    /// Clear the entire window.
    pub fn wclear(&mut self, win: &mut Window) -> Result<()> {
        win.clear()
    }

    /// Erase the window (fill with blanks).
    pub fn werase(&mut self, win: &mut Window) -> Result<()> {
        win.erase()
    }

    /// Clear from cursor to end of line.
    pub fn wclrtoeol(&mut self, win: &mut Window) -> Result<()> {
        win.clrtoeol()
    }

    /// Clear from cursor to end of window.
    pub fn wclrtobot(&mut self, win: &mut Window) -> Result<()> {
        win.clrtobot()
    }

    /// Draw a box around a window.
    pub fn wbox(&mut self, win: &mut Window, verch: ChType, horch: ChType) -> Result<()> {
        win.box_(verch, horch)
    }

    /// Draw a border around a window.
    #[allow(clippy::too_many_arguments)]
    pub fn wborder(
        &mut self,
        win: &mut Window,
        ls: ChType,
        rs: ChType,
        ts: ChType,
        bs: ChType,
        tl: ChType,
        tr: ChType,
        bl: ChType,
        br: ChType,
    ) -> Result<()> {
        win.border(ls, rs, ts, bs, tl, tr, bl, br)
    }

    /// Draw a horizontal line.
    pub fn whline(&mut self, win: &mut Window, ch: ChType, n: i32) -> Result<()> {
        win.hline(ch, n)
    }

    /// Draw a vertical line.
    pub fn wvline(&mut self, win: &mut Window, ch: ChType, n: i32) -> Result<()> {
        win.vline(ch, n)
    }

    /// Draw a horizontal line at position.
    pub fn mvwhline(&mut self, win: &mut Window, y: i32, x: i32, ch: ChType, n: i32) -> Result<()> {
        win.mv(y, x)?;
        win.hline(ch, n)
    }

    /// Draw a vertical line at position.
    pub fn mvwvline(&mut self, win: &mut Window, y: i32, x: i32, ch: ChType, n: i32) -> Result<()> {
        win.mv(y, x)?;
        win.vline(ch, n)
    }

    /// Insert a character before the cursor.
    pub fn winsch(&mut self, win: &mut Window, ch: ChType) -> Result<()> {
        win.insch(ch)
    }

    /// Insert a character at specified position.
    pub fn mvwinsch(&mut self, win: &mut Window, y: i32, x: i32, ch: ChType) -> Result<()> {
        win.mv(y, x)?;
        win.insch(ch)
    }

    /// Delete the character under the cursor.
    pub fn wdelch(&mut self, win: &mut Window) -> Result<()> {
        win.delch()
    }

    /// Delete a character at specified position.
    pub fn mvwdelch(&mut self, win: &mut Window, y: i32, x: i32) -> Result<()> {
        win.mv(y, x)?;
        win.delch()
    }

    /// Insert a blank line above the cursor.
    pub fn winsertln(&mut self, win: &mut Window) -> Result<()> {
        win.insertln()
    }

    /// Delete the line under the cursor.
    pub fn wdeleteln(&mut self, win: &mut Window) -> Result<()> {
        win.deleteln()
    }

    /// Insert/delete lines.
    pub fn winsdelln(&mut self, win: &mut Window, n: i32) -> Result<()> {
        win.insdelln(n)
    }

    /// Get the character at the current cursor position.
    pub fn winch(&self, win: &Window) -> ChType {
        win.inch()
    }

    /// Get the character at specified position.
    pub fn mvwinch(&mut self, win: &mut Window, y: i32, x: i32) -> Result<ChType> {
        win.mv(y, x)?;
        Ok(win.inch())
    }

    /// Get a string from a window.
    pub fn winnstr(&self, win: &Window, n: i32) -> String {
        win.instr(n)
    }

    /// Get a string from specified position.
    pub fn mvwinnstr(&mut self, win: &mut Window, y: i32, x: i32, n: i32) -> Result<String> {
        win.mv(y, x)?;
        Ok(win.instr(n))
    }

    // ========================================================================
    // stdscr inch/instr family
    // ========================================================================

    /// Get the character at the current cursor position on stdscr.
    #[must_use]
    pub fn inch(&self) -> ChType {
        self.stdscr.inch()
    }

    /// Move cursor and get the character at that position on stdscr.
    pub fn mvinch(&mut self, y: i32, x: i32) -> Result<ChType> {
        self.stdscr.mvinch(y, x)
    }

    /// Get a string from stdscr at the current position.
    #[must_use]
    pub fn instr(&self, n: i32) -> String {
        self.stdscr.instr(n)
    }

    /// Get a string from stdscr with a limit (alias for instr).
    #[must_use]
    pub fn innstr(&self, n: i32) -> String {
        self.stdscr.innstr(n)
    }

    /// Move cursor and get a string from stdscr.
    pub fn mvinstr(&mut self, y: i32, x: i32, n: i32) -> Result<String> {
        self.stdscr.mv(y, x)?;
        Ok(self.stdscr.instr(n))
    }

    /// Move cursor and get a string from stdscr with a limit.
    pub fn mvinnstr(&mut self, y: i32, x: i32, n: i32) -> Result<String> {
        self.stdscr.mvinnstr(y, x, n)
    }

    // ========================================================================
    // inchnstr family (character array with attributes)
    // ========================================================================

    /// Get a string of characters with attributes from stdscr.
    pub fn inchnstr(&self, chstr: &mut [ChType], n: i32) -> i32 {
        self.stdscr.inchnstr(chstr, n)
    }

    /// Move cursor and get a string of characters with attributes from stdscr.
    pub fn mvinchnstr(&mut self, y: i32, x: i32, chstr: &mut [ChType], n: i32) -> Result<i32> {
        self.stdscr.mvinchnstr(y, x, chstr, n)
    }

    /// Get a string of characters with attributes from a window.
    pub fn winchnstr(&self, win: &Window, chstr: &mut [ChType], n: i32) -> i32 {
        win.inchnstr(chstr, n)
    }

    /// Move cursor and get a string of characters with attributes from a window.
    pub fn mvwinchnstr(
        &mut self,
        win: &mut Window,
        y: i32,
        x: i32,
        chstr: &mut [ChType],
        n: i32,
    ) -> Result<i32> {
        win.mvinchnstr(y, x, chstr, n)
    }

    /// Set window background character.
    pub fn wbkgdset(&mut self, win: &mut Window, ch: ChType) {
        win.bkgdset(ch);
    }

    /// Get window background character.
    pub fn wgetbkgd(&self, win: &Window) -> ChType {
        win.getbkgd()
    }

    /// Set window background and apply to all characters.
    pub fn wbkgd(&mut self, win: &mut Window, ch: ChType) -> Result<()> {
        win.bkgd(ch)
    }

    /// Turn on window attributes.
    pub fn wattron(&mut self, win: &mut Window, attr: AttrT) -> Result<()> {
        win.attron(attr)
    }

    /// Turn off window attributes.
    pub fn wattroff(&mut self, win: &mut Window, attr: AttrT) -> Result<()> {
        win.attroff(attr)
    }

    /// Set window attributes.
    pub fn wattrset(&mut self, win: &mut Window, attr: AttrT) -> Result<()> {
        win.attrset(attr)
    }

    /// Get window attributes.
    pub fn wgetattrs(&self, win: &Window) -> AttrT {
        win.getattrs()
    }

    /// Change attributes of characters at current position in stdscr.
    ///
    /// Changes the attributes of `n` characters starting at the cursor.
    /// If `n` is -1, changes attributes to end of line.
    pub fn chgat(&mut self, n: i32, attr: AttrT, color: i16) -> Result<()> {
        self.stdscr.chgat(n, attr, color)
    }

    /// Move cursor and change attributes in stdscr.
    pub fn mvchgat(&mut self, y: i32, x: i32, n: i32, attr: AttrT, color: i16) -> Result<()> {
        self.stdscr.mvchgat(y, x, n, attr, color)
    }

    /// Change attributes of characters at current position in window.
    pub fn wchgat(&mut self, win: &mut Window, n: i32, attr: AttrT, color: i16) -> Result<()> {
        win.wchgat(n, attr, color)
    }

    /// Move cursor and change attributes in window.
    pub fn mvwchgat(
        &mut self,
        win: &mut Window,
        y: i32,
        x: i32,
        n: i32,
        attr: AttrT,
        color: i16,
    ) -> Result<()> {
        win.mvwchgat(y, x, n, attr, color)
    }

    /// Set window color pair.
    pub fn wcolor_set(&mut self, win: &mut Window, pair: i16) -> Result<()> {
        win.color_set(pair)
    }

    /// Scroll window up.
    pub fn wscrl(&mut self, win: &mut Window, n: i32) -> Result<()> {
        win.scrl(n)
    }

    /// Set scrolling region.
    pub fn wsetscrreg(&mut self, win: &mut Window, top: i32, bot: i32) -> Result<()> {
        win.setscrreg(top, bot)
    }

    /// Get scrolling region.
    pub fn wgetscrreg(&self, win: &Window) -> (i32, i32) {
        win.getscrreg()
    }

    /// Enable/disable scrolling for a window.
    pub fn wscrollok(&mut self, win: &mut Window, bf: bool) {
        win.scrollok(bf);
    }

    /// Check if scrolling is enabled for a window.
    pub fn wis_scrollok(&self, win: &Window) -> bool {
        win.is_scrollok()
    }

    /// Enable/disable keypad for a window.
    pub fn wkeypad(&mut self, win: &mut Window, bf: bool) {
        win.keypad(bf);
    }

    /// Check if keypad is enabled for a window.
    pub fn wis_keypad(&self, win: &Window) -> bool {
        win.is_keypad()
    }

    /// Enable/disable nodelay mode for a window.
    pub fn wnodelay(&mut self, win: &mut Window, bf: bool) {
        win.nodelay(bf);
    }

    /// Check if nodelay mode is enabled for a window.
    pub fn wis_nodelay(&self, win: &Window) -> bool {
        win.is_nodelay()
    }

    /// Set input timeout for a window.
    pub fn wtimeout(&mut self, win: &mut Window, delay: i32) {
        win.timeout(delay);
    }

    /// Get input timeout for a window.
    pub fn wgetdelay(&self, win: &Window) -> i32 {
        win.getdelay()
    }

    /// Enable/disable cursor leave mode for a window.
    pub fn wleaveok(&mut self, win: &mut Window, bf: bool) {
        win.leaveok(bf);
    }

    /// Check if cursor leave mode is enabled.
    pub fn wis_leaveok(&self, win: &Window) -> bool {
        win.is_leaveok()
    }

    /// Enable/disable clear-on-refresh for a window.
    pub fn wclearok(&mut self, win: &mut Window, bf: bool) {
        win.clearok(bf);
    }

    /// Check if clear-on-refresh is enabled for a window.
    pub fn wis_cleared(&self, win: &Window) -> bool {
        win.is_cleared()
    }

    /// Enable/disable hardware line insertion for a window.
    pub fn widlok(&mut self, win: &mut Window, bf: bool) {
        win.idlok(bf);
    }

    /// Check if hardware line insertion is enabled.
    pub fn wis_idlok(&self, win: &Window) -> bool {
        win.is_idlok()
    }

    /// Enable/disable hardware character insertion for a window.
    pub fn widcok(&mut self, win: &mut Window, bf: bool) {
        win.idcok(bf);
    }

    /// Check if hardware character insertion is enabled.
    pub fn wis_idcok(&self, win: &Window) -> bool {
        win.is_idcok()
    }

    /// Enable/disable immediate refresh for a window.
    pub fn wimmedok(&mut self, win: &mut Window, bf: bool) {
        win.immedok(bf);
    }

    /// Check if immediate refresh is enabled.
    pub fn wis_immedok(&self, win: &Window) -> bool {
        win.is_immedok()
    }

    /// Enable/disable automatic sync for a window.
    pub fn wsyncok(&mut self, win: &mut Window, bf: bool) {
        win.syncok(bf);
    }

    /// Check if automatic sync is enabled.
    pub fn wis_syncok(&self, win: &Window) -> bool {
        win.is_syncok()
    }

    /// Enable/disable notimeout mode for a window.
    pub fn wnotimeout(&mut self, win: &mut Window, bf: bool) {
        win.notimeout(bf);
    }

    /// Check if notimeout mode is enabled.
    pub fn wis_notimeout(&self, win: &Window) -> bool {
        win.is_notimeout()
    }

    /// Touch a window (mark all lines as changed).
    pub fn touchwin(&mut self, win: &mut Window) {
        win.touchwin();
    }

    /// Untouch a window (mark all lines as unchanged).
    pub fn untouchwin(&mut self, win: &mut Window) {
        win.untouchwin();
    }

    /// Touch specific lines in a window.
    pub fn wtouchln(&mut self, win: &mut Window, start: i32, count: i32, changed: bool) {
        win.touchln(start, count, changed);
    }

    /// Check if a specific line is touched.
    pub fn is_linetouched(&self, win: &Window, line: i32) -> bool {
        win.is_linetouched(line)
    }

    /// Check if any part of the window is touched.
    pub fn wis_wintouched(&self, win: &Window) -> bool {
        win.is_wintouched()
    }

    /// Get window Y position (row count).
    pub fn wgetmaxy(&self, win: &Window) -> i32 {
        win.getmaxy()
    }

    /// Get window X position (column count).
    pub fn wgetmaxx(&self, win: &Window) -> i32 {
        win.getmaxx()
    }

    /// Get window begin Y position.
    pub fn wgetbegy(&self, win: &Window) -> i32 {
        win.getbegy()
    }

    /// Get window begin X position.
    pub fn wgetbegx(&self, win: &Window) -> i32 {
        win.getbegx()
    }

    /// Get current cursor Y position.
    pub fn wgetcury(&self, win: &Window) -> i32 {
        win.getcury()
    }

    /// Get current cursor X position.
    pub fn wgetcurx(&self, win: &Window) -> i32 {
        win.getcurx()
    }

    /// Get parent window relative Y position.
    pub fn wgetpary(&self, win: &Window) -> i32 {
        win.getpary()
    }

    /// Get parent window relative X position.
    pub fn wgetparx(&self, win: &Window) -> i32 {
        win.getparx()
    }

    /// Get both max Y and X dimensions.
    pub fn wgetmaxyx(&self, win: &Window) -> (i32, i32) {
        (win.getmaxy(), win.getmaxx())
    }

    /// Get both begin Y and X positions.
    pub fn wgetbegyx(&self, win: &Window) -> (i32, i32) {
        (win.getbegy(), win.getbegx())
    }

    /// Get both cursor Y and X positions.
    pub fn wgetyx(&self, win: &Window) -> (i32, i32) {
        (win.getcury(), win.getcurx())
    }

    /// Get both parent Y and X positions.
    pub fn wgetparyx(&self, win: &Window) -> (i32, i32) {
        (win.getpary(), win.getparx())
    }

    /// Check if window is a pad.
    pub fn wis_pad(&self, win: &Window) -> bool {
        win.is_pad()
    }

    /// Check if window is a subwindow.
    pub fn wis_subwin(&self, win: &Window) -> bool {
        win.is_subwin()
    }

    /// Create a subwindow.
    pub fn wsubwin(
        &mut self,
        win: &Window,
        nlines: i32,
        ncols: i32,
        begy: i32,
        begx: i32,
    ) -> Result<Window> {
        win.subwin(nlines, ncols, begy, begx)
    }

    /// Create a derived window (relative coordinates).
    pub fn wderwin(
        &mut self,
        win: &Window,
        nlines: i32,
        ncols: i32,
        begy: i32,
        begx: i32,
    ) -> Result<Window> {
        win.derwin(nlines, ncols, begy, begx)
    }

    /// Duplicate a window.
    pub fn wdupwin(&mut self, win: &Window) -> Result<Window> {
        win.dupwin()
    }

    /// Move a window to a new position.
    pub fn wmvwin(&mut self, win: &mut Window, y: i32, x: i32) -> Result<()> {
        win.mvwin(y, x)
    }

    /// Resize a window.
    ///
    /// This changes the window's dimensions. Content is preserved where possible.
    pub fn wresize(&mut self, win: &mut Window, lines: i32, cols: i32) -> Result<()> {
        win.resize(lines, cols)
    }

    /// Create a subpad.
    pub fn wsubpad(
        &mut self,
        win: &Window,
        nlines: i32,
        ncols: i32,
        begy: i32,
        begx: i32,
    ) -> Result<Window> {
        win.subpad(nlines, ncols, begy, begx)
    }

    /// Copy one window onto another.
    #[allow(clippy::too_many_arguments)]
    pub fn copywin(
        &mut self,
        src: &Window,
        dst: &mut Window,
        sminrow: i32,
        smincol: i32,
        dminrow: i32,
        dmincol: i32,
        dmaxrow: i32,
        dmaxcol: i32,
        overlay: bool,
    ) -> Result<()> {
        let src_maxy = src.getmaxy();
        let src_maxx = src.getmaxx();
        let dst_maxy = dst.getmaxy();
        let dst_maxx = dst.getmaxx();

        for dst_y in dminrow..=dmaxrow {
            let src_y = sminrow + (dst_y - dminrow);
            if src_y >= src_maxy || dst_y >= dst_maxy {
                break;
            }

            for dst_x in dmincol..=dmaxcol {
                let src_x = smincol + (dst_x - dmincol);
                if src_x >= src_maxx || dst_x >= dst_maxx {
                    break;
                }

                if let Some(src_line) = src.line(src_y as usize) {
                    let ch = src_line.get(src_x as usize);
                    // In overlay mode, skip blanks
                    if overlay {
                        #[cfg(not(feature = "wide"))]
                        let is_blank = (ch & 0xFF) == b' ' as ChType;
                        #[cfg(feature = "wide")]
                        let is_blank = ch.chars[0] == ' ';
                        if is_blank {
                            continue;
                        }
                    }
                    if let Some(dst_line) = dst.line_mut(dst_y as usize) {
                        dst_line.set(dst_x as usize, ch);
                    }
                }
            }
        }

        Ok(())
    }

    /// Copy source window to destination, overlaying non-blank characters.
    pub fn overlay(&mut self, src: &Window, dst: &mut Window) -> Result<()> {
        let dmaxy = dst.getmaxy().min(src.getmaxy());
        let dmaxx = dst.getmaxx().min(src.getmaxx());
        self.copywin(src, dst, 0, 0, 0, 0, dmaxy - 1, dmaxx - 1, true)
    }

    /// Copy source window to destination, overwriting all characters.
    pub fn overwrite(&mut self, src: &Window, dst: &mut Window) -> Result<()> {
        let dmaxy = dst.getmaxy().min(src.getmaxy());
        let dmaxx = dst.getmaxx().min(src.getmaxx());
        self.copywin(src, dst, 0, 0, 0, 0, dmaxy - 1, dmaxx - 1, false)
    }

    /// Mark a window for complete redraw on next refresh.
    pub fn redrawwin(&mut self, win: &mut Window) -> Result<()> {
        win.touchwin();
        Ok(())
    }

    /// Request that lines be redrawn (to fix screen corruption).
    pub fn wredrawln(&mut self, win: &mut Window, beg_line: i32, num_lines: i32) -> Result<()> {
        win.touchln(beg_line, num_lines, true);
        Ok(())
    }

    // ========================================================================
    // Soft Label Key (SLK) functions
    // ========================================================================

    /// Initialize soft labels with the specified format.
    ///
    /// This must be called before `Screen::init()` to reserve space for soft labels.
    /// The format specifies how many labels and their arrangement:
    /// - 0: 3-2-3 format (8 labels)
    /// - 1: 4-4 format (8 labels)
    /// - 2: 4-4-4 format (12 labels)
    /// - 3: 4-4-4 PC style format (12 labels)
    #[cfg(feature = "slk")]
    pub fn slk_init(&mut self, fmt: i32) -> Result<()> {
        let format = SlkFormat::from_int(fmt)
            .ok_or_else(|| Error::InvalidArgument("invalid SLK format".to_string()))?;
        let mut slk = SlkState::new(format);
        slk.init_with_size(self.terminal.columns(), self.terminal.lines())?;
        self.slk = Some(slk);
        Ok(())
    }

    /// Set a soft label.
    ///
    /// Sets the text for label `labnum` (0-based for 8 labels, 0-11 for 12 labels).
    /// The `justify` parameter specifies text alignment:
    /// - 0: Left justify
    /// - 1: Center
    /// - 2: Right justify
    #[cfg(feature = "slk")]
    pub fn slk_set(&mut self, labnum: i32, label: &str, justify: i32) -> Result<()> {
        let slk = self.slk.as_mut().ok_or(Error::NotInitialized)?;
        slk.set(labnum, label, justify)
    }

    /// Get a soft label.
    ///
    /// Returns the text of label `labnum`, or None if not set.
    #[cfg(feature = "slk")]
    pub fn slk_label(&self, labnum: i32) -> Option<&str> {
        self.slk.as_ref().and_then(|slk| slk.label(labnum))
    }

    /// Refresh the soft labels, writing them to the terminal.
    ///
    /// This outputs the soft labels to the screen immediately.
    #[cfg(feature = "slk")]
    pub fn slk_refresh(&mut self) -> Result<()> {
        if let Some(slk) = &mut self.slk {
            // Render the SLK line
            let output = slk.render_ansi();
            if !output.is_empty() {
                self.terminal.write(output.as_bytes())?;
                self.terminal.flush()?;
            }
            slk.refresh()?;
        }
        Ok(())
    }

    /// Mark soft labels for refresh without writing to terminal.
    ///
    /// The labels will be refreshed on the next `doupdate()` call.
    #[cfg(feature = "slk")]
    pub fn slk_noutrefresh(&mut self) -> Result<()> {
        if let Some(slk) = &mut self.slk {
            slk.noutrefresh()?;
        }
        Ok(())
    }

    /// Clear soft labels from the screen.
    ///
    /// The labels are hidden but not destroyed. Use `slk_restore()` to show them again.
    #[cfg(feature = "slk")]
    pub fn slk_clear(&mut self) -> Result<()> {
        if let Some(slk) = &mut self.slk {
            slk.clear_all()?;
            // Clear the SLK area on screen
            let row = slk.display_row();
            let cols = self.terminal.columns();
            let clear_line = format!("\x1b[{};1H{}\x1b[0m", row + 1, " ".repeat(cols as usize));
            self.terminal.write(clear_line.as_bytes())?;
            self.terminal.flush()?;
        }
        Ok(())
    }

    /// Restore soft labels after they were cleared.
    #[cfg(feature = "slk")]
    pub fn slk_restore(&mut self) -> Result<()> {
        if let Some(slk) = &mut self.slk {
            slk.restore()?;
            self.slk_refresh()?;
        }
        Ok(())
    }

    /// Touch soft labels, marking them for refresh.
    #[cfg(feature = "slk")]
    pub fn slk_touch(&mut self) -> Result<()> {
        if let Some(slk) = &mut self.slk {
            slk.touch();
        }
        Ok(())
    }

    /// Set the attributes for soft labels.
    #[cfg(feature = "slk")]
    pub fn slk_attrset(&mut self, attrs: AttrT) -> Result<()> {
        if let Some(slk) = &mut self.slk {
            slk.attrset(attrs)?;
        }
        Ok(())
    }

    /// Turn on attributes for soft labels.
    #[cfg(feature = "slk")]
    pub fn slk_attron(&mut self, attrs: AttrT) -> Result<()> {
        if let Some(slk) = &mut self.slk {
            slk.attron(attrs)?;
        }
        Ok(())
    }

    /// Turn off attributes for soft labels.
    #[cfg(feature = "slk")]
    pub fn slk_attroff(&mut self, attrs: AttrT) -> Result<()> {
        if let Some(slk) = &mut self.slk {
            slk.attroff(attrs)?;
        }
        Ok(())
    }

    /// Get the current attributes for soft labels.
    #[cfg(feature = "slk")]
    pub fn slk_attr(&self) -> AttrT {
        self.slk.as_ref().map_or(0, |slk| slk.attr())
    }

    /// Set the color pair for soft labels.
    #[cfg(feature = "slk")]
    pub fn slk_color(&mut self, pair: i16) -> Result<()> {
        if let Some(slk) = &mut self.slk {
            slk.color(pair)?;
        }
        Ok(())
    }

    /// Turn off attributes for soft labels (X/Open style).
    #[cfg(feature = "slk")]
    pub fn slk_attr_off(&mut self, attrs: AttrT) -> Result<()> {
        self.slk_attroff(attrs)
    }

    /// Turn on attributes for soft labels (X/Open style).
    #[cfg(feature = "slk")]
    pub fn slk_attr_on(&mut self, attrs: AttrT) -> Result<()> {
        self.slk_attron(attrs)
    }

    /// Set the attributes for soft labels (X/Open style).
    #[cfg(feature = "slk")]
    pub fn slk_attr_set(&mut self, attrs: AttrT, pair: i16) -> Result<()> {
        let combined = attrs | attr::color_pair(pair);
        self.slk_attrset(combined)
    }

    /// Set a soft label key using a wide string.
    #[cfg(all(feature = "slk", feature = "wide"))]
    pub fn slk_wset(&mut self, labnum: i32, label: &str, justify: i32) -> Result<()> {
        self.slk_set(labnum, label, justify)
    }

    /// Check if soft labels are initialized.
    #[cfg(feature = "slk")]
    pub fn slk_is_initialized(&self) -> bool {
        self.slk.as_ref().is_some_and(|slk| slk.is_initialized())
    }

    // ========================================================================
    // Terminfo query functions
    // ========================================================================

    /// Get a boolean capability value from terminfo.
    ///
    /// Returns:
    /// - 1 if the capability is present
    /// - 0 if the capability is absent
    /// - -1 if the capability name is not recognized
    ///
    /// Common boolean capabilities:
    /// - "am" - automatic margins
    /// - "bce" - background color erase
    /// - "km" - has a meta key
    /// - "mc5i" - printer won't echo on screen
    /// - "mir" - safe to move while in insert mode
    /// - "msgr" - safe to move while in standout mode
    /// - "xenl" - newline ignored after 80 cols
    /// - "xon" - terminal uses XON/XOFF handshaking
    pub fn tigetflag(&self, capname: &str) -> i32 {
        let term_type = self.terminal.term_type();
        let is_modern = matches!(
            term_type,
            "xterm"
                | "xterm-256color"
                | "screen"
                | "screen-256color"
                | "tmux"
                | "tmux-256color"
                | "rxvt"
                | "rxvt-unicode"
                | "kitty"
                | "alacritty"
                | "wezterm"
                | "iterm2"
                | "vte"
        );

        match capname {
            // Automatic margins (most terminals have this)
            "am" => 1,
            // Background color erase (modern terminals)
            "bce" => {
                if is_modern {
                    1
                } else {
                    0
                }
            }
            // Has a meta key
            "km" => 1,
            // Terminal uses XON/XOFF
            "xon" => 0,
            // Move in insert mode safe
            "mir" => {
                if is_modern {
                    1
                } else {
                    0
                }
            }
            // Move in standout mode safe
            "msgr" => {
                if is_modern {
                    1
                } else {
                    0
                }
            }
            // Newline ignored after 80 cols (xenl bug)
            "xenl" => {
                if is_modern {
                    1
                } else {
                    0
                }
            }
            // Has hardware tabs
            "ht" | "it" => 1,
            // Can change color
            "ccc" => {
                if self.terminal.can_change_color() {
                    1
                } else {
                    0
                }
            }
            // Has insert character
            "ich" => {
                if self.terminal.has_ic() {
                    1
                } else {
                    0
                }
            }
            // Has insert line
            "il" => {
                if self.terminal.has_il() {
                    1
                } else {
                    0
                }
            }
            // Unknown capability
            _ => -1,
        }
    }

    /// Get a numeric capability value from terminfo.
    ///
    /// Returns:
    /// - The capability value if present
    /// - -2 if the capability is absent
    /// - -1 if the capability name is not recognized
    ///
    /// Common numeric capabilities:
    /// - "cols" - number of columns
    /// - "lines" - number of lines
    /// - "colors" - max number of colors
    /// - "pairs" - max number of color pairs
    pub fn tigetnum(&self, capname: &str) -> i32 {
        match capname {
            "cols" | "co" => self.terminal.columns(),
            "lines" | "li" => self.terminal.lines(),
            "colors" => self.terminal.colors(),
            "pairs" => self.terminal.color_pairs(),
            "it" => self.tabsize, // initial tab size
            // Unknown capability
            _ => -1,
        }
    }

    /// Get a string capability value from terminfo.
    ///
    /// Returns:
    /// - Some(string) if the capability is present
    /// - None if the capability is absent or not recognized
    ///
    /// Common string capabilities:
    /// - "clear" - clear screen
    /// - "cup" - cursor position
    /// - "cuf1" - cursor forward one
    /// - "cub1" - cursor back one
    /// - "cuu1" - cursor up one
    /// - "cud1" - cursor down one
    /// - "smcup" - enter cursor-addressing mode
    /// - "rmcup" - exit cursor-addressing mode
    /// - "smso" - enter standout mode
    /// - "rmso" - exit standout mode
    /// - "setaf" - set ANSI foreground color
    /// - "setab" - set ANSI background color
    pub fn tigetstr(&self, capname: &str) -> Option<String> {
        match capname {
            // Clear screen
            "clear" | "cl" => Some("\x1b[H\x1b[J".to_string()),
            // Cursor position (template: \x1b[%d;%dH)
            "cup" | "cm" => Some("\x1b[%i%p1%d;%p2%dH".to_string()),
            // Cursor movements
            "cuf1" => Some("\x1b[C".to_string()),
            "cub1" => Some("\x08".to_string()), // backspace
            "cuu1" => Some("\x1b[A".to_string()),
            "cud1" => Some("\x1b[B".to_string()),
            "home" => Some("\x1b[H".to_string()),
            // Cursor n positions
            "cuf" => Some("\x1b[%p1%dC".to_string()),
            "cub" => Some("\x1b[%p1%dD".to_string()),
            "cuu" => Some("\x1b[%p1%dA".to_string()),
            "cud" => Some("\x1b[%p1%dB".to_string()),
            // Enter/exit cursor addressing mode (alternate screen)
            "smcup" => Some("\x1b[?1049h".to_string()),
            "rmcup" => Some("\x1b[?1049l".to_string()),
            // Enter/exit standout mode
            "smso" => Some("\x1b[7m".to_string()),
            "rmso" => Some("\x1b[27m".to_string()),
            // Enter/exit underline mode
            "smul" => Some("\x1b[4m".to_string()),
            "rmul" => Some("\x1b[24m".to_string()),
            // Bold mode
            "bold" => Some("\x1b[1m".to_string()),
            // Dim mode
            "dim" => Some("\x1b[2m".to_string()),
            // Blink mode
            "blink" => Some("\x1b[5m".to_string()),
            // Reverse mode
            "rev" => Some("\x1b[7m".to_string()),
            // Reset all attributes
            "sgr0" => Some("\x1b[0m".to_string()),
            // Set foreground color (ANSI)
            "setaf" => Some("\x1b[3%p1%dm".to_string()),
            // Set background color (ANSI)
            "setab" => Some("\x1b[4%p1%dm".to_string()),
            // Original colors
            "op" => Some("\x1b[39;49m".to_string()),
            // Invisible cursor
            "civis" => Some("\x1b[?25l".to_string()),
            // Normal cursor
            "cnorm" => Some("\x1b[?25h".to_string()),
            // Very visible cursor
            "cvvis" => Some("\x1b[?25h\x1b[?12h".to_string()),
            // Insert/delete character
            "ich1" => Some("\x1b[@".to_string()),
            "dch1" => Some("\x1b[P".to_string()),
            // Insert/delete line
            "il1" => Some("\x1b[L".to_string()),
            "dl1" => Some("\x1b[M".to_string()),
            // Scroll forward/reverse
            "ind" => Some("\x1b[S".to_string()),
            "ri" => Some("\x1b[T".to_string()),
            // Clear to end of line/screen
            "el" => Some("\x1b[K".to_string()),
            "ed" => Some("\x1b[J".to_string()),
            // Key sequences
            "kcuu1" => Some("\x1b[A".to_string()),
            "kcud1" => Some("\x1b[B".to_string()),
            "kcuf1" => Some("\x1b[C".to_string()),
            "kcub1" => Some("\x1b[D".to_string()),
            "khome" => Some("\x1b[H".to_string()),
            "kend" => Some("\x1b[F".to_string()),
            "kpp" => Some("\x1b[5~".to_string()),   // page up
            "knp" => Some("\x1b[6~".to_string()),   // page down
            "kich1" => Some("\x1b[2~".to_string()), // insert
            "kdch1" => Some("\x1b[3~".to_string()), // delete
            "kbs" => Some("\x7f".to_string()),      // backspace
            // Function keys
            "kf1" => Some("\x1bOP".to_string()),
            "kf2" => Some("\x1bOQ".to_string()),
            "kf3" => Some("\x1bOR".to_string()),
            "kf4" => Some("\x1bOS".to_string()),
            "kf5" => Some("\x1b[15~".to_string()),
            "kf6" => Some("\x1b[17~".to_string()),
            "kf7" => Some("\x1b[18~".to_string()),
            "kf8" => Some("\x1b[19~".to_string()),
            "kf9" => Some("\x1b[20~".to_string()),
            "kf10" => Some("\x1b[21~".to_string()),
            "kf11" => Some("\x1b[23~".to_string()),
            "kf12" => Some("\x1b[24~".to_string()),
            // Unknown capability
            _ => None,
        }
    }

    /// Output a terminfo string to the terminal.
    ///
    /// This is equivalent to tputs() in ncurses.
    pub fn putp(&mut self, s: &str) -> Result<()> {
        self.terminal.write(s.as_bytes())?;
        self.terminal.flush()
    }

    /// Parameterized terminal string.
    ///
    /// This is a simplified version of tparm() that handles basic parameter
    /// substitution in terminfo strings.
    ///
    /// Supports:
    /// - %p1 through %p9: parameter values
    /// - %d: output parameter as decimal
    /// - %i: increment first two parameters by 1
    /// - %%: literal %
    pub fn tparm(&self, s: &str, params: &[i32]) -> String {
        let mut result = String::new();
        let mut chars = s.chars().peekable();
        let mut params = params.to_vec();
        // Extend params to at least 9 elements
        params.resize(9, 0);

        while let Some(ch) = chars.next() {
            if ch == '%' {
                match chars.next() {
                    Some('%') => result.push('%'),
                    Some('i') => {
                        // Increment first two parameters
                        if !params.is_empty() {
                            params[0] += 1;
                        }
                        if params.len() > 1 {
                            params[1] += 1;
                        }
                    }
                    Some('p') => {
                        // Parameter reference %p1 through %p9
                        if let Some(digit) = chars.next() {
                            if let Some(n) = digit.to_digit(10) {
                                if (1..=9).contains(&n) {
                                    let idx = (n - 1) as usize;
                                    // Look for the format specifier
                                    if chars.peek() == Some(&'%') {
                                        chars.next();
                                        if chars.peek() == Some(&'d') {
                                            chars.next();
                                            result.push_str(&params[idx].to_string());
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Some('d') => {
                        // Direct decimal output (legacy format)
                        if !params.is_empty() {
                            result.push_str(&params[0].to_string());
                        }
                    }
                    Some(c) => {
                        // Unknown format, preserve it
                        result.push('%');
                        result.push(c);
                    }
                    None => result.push('%'),
                }
            } else {
                result.push(ch);
            }
        }

        result
    }

    // ========================================================================
    // Screen dump/restore functions
    // ========================================================================

    /// Dump the screen contents to a file.
    ///
    /// This saves the virtual screen (newscr) contents to a file that can be
    /// restored later with `scr_restore()`.
    pub fn scr_dump(&self, filename: &str) -> Result<()> {
        use std::fs::File;
        use std::io::Write;

        let mut file = File::create(filename)?;

        // Write header: magic number, version, dimensions
        let lines = self.newscr.getmaxy();
        let cols = self.newscr.getmaxx();

        // Simple binary format: NCDUMP + version(1) + lines(4) + cols(4) + data
        file.write_all(b"NCDUMP")?;
        file.write_all(&[1u8]) // version
            ?;
        file.write_all(&(lines as u32).to_le_bytes())?;
        file.write_all(&(cols as u32).to_le_bytes())?;

        // Write screen data
        for y in 0..lines {
            if let Some(line) = self.newscr.line(y as usize) {
                for x in 0..cols {
                    let ch = line.get(x as usize);
                    #[cfg(not(feature = "wide"))]
                    {
                        file.write_all(&(ch as u32).to_le_bytes())?;
                    }
                    #[cfg(feature = "wide")]
                    {
                        // For wide characters, serialize the primary char and attrs
                        let c = ch.chars[0] as u32;
                        let a = ch.attr;
                        file.write_all(&c.to_le_bytes())?;
                        file.write_all(&a.to_le_bytes())?;
                    }
                }
            }
        }

        file.flush()?;
        Ok(())
    }

    /// Restore screen contents from a file.
    ///
    /// This restores screen contents saved by `scr_dump()`. The screen should
    /// be refreshed after calling this to display the restored contents.
    pub fn scr_restore(&mut self, filename: &str) -> Result<()> {
        use std::fs::File;
        use std::io::Read;

        let mut file = File::open(filename)?;

        // Read and verify header
        let mut magic = [0u8; 6];
        file.read_exact(&mut magic)?;
        if &magic != b"NCDUMP" {
            return Err(Error::InvalidArgument(
                "Invalid screen dump file".to_string(),
            ));
        }

        let mut version = [0u8; 1];
        file.read_exact(&mut version)?;
        if version[0] != 1 {
            return Err(Error::InvalidArgument(
                "Unsupported dump file version".to_string(),
            ));
        }

        let mut lines_bytes = [0u8; 4];
        let mut cols_bytes = [0u8; 4];
        file.read_exact(&mut lines_bytes)?;
        file.read_exact(&mut cols_bytes)?;

        let file_lines = u32::from_le_bytes(lines_bytes) as i32;
        let file_cols = u32::from_le_bytes(cols_bytes) as i32;

        // Read screen data (limited to current screen size)
        let screen_lines = self.newscr.getmaxy().min(file_lines);
        let screen_cols = self.newscr.getmaxx().min(file_cols);

        for y in 0..file_lines {
            for x in 0..file_cols {
                #[cfg(not(feature = "wide"))]
                {
                    let mut ch_bytes = [0u8; 4];
                    file.read_exact(&mut ch_bytes)?;
                    let ch = u32::from_le_bytes(ch_bytes) as ChType;

                    if y < screen_lines && x < screen_cols {
                        if let Some(line) = self.newscr.line_mut(y as usize) {
                            line.set(x as usize, ch);
                        }
                    }
                }
                #[cfg(feature = "wide")]
                {
                    let mut c_bytes = [0u8; 4];
                    let mut a_bytes = [0u8; 4];
                    file.read_exact(&mut c_bytes)?;
                    file.read_exact(&mut a_bytes)?;
                    let c = u32::from_le_bytes(c_bytes);
                    let a = u32::from_le_bytes(a_bytes);

                    if y < screen_lines && x < screen_cols {
                        if let Some(line) = self.newscr.line_mut(y as usize) {
                            let mut cchar = crate::wide::CCharT::default();
                            cchar.chars[0] = char::from_u32(c).unwrap_or(' ');
                            cchar.attr = a;
                            line.set(x as usize, cchar);
                        }
                    }
                }
            }
        }

        // Mark screen as needing refresh
        self.newscr.touchwin();
        Ok(())
    }

    /// Initialize screen from a dump file.
    ///
    /// This is similar to `scr_restore()` but is intended to be called before
    /// the first refresh to pre-populate the screen.
    pub fn scr_init(&mut self, filename: &str) -> Result<()> {
        self.scr_restore(filename)
    }

    /// Set the screen contents from a dump file.
    ///
    /// This combines the functionality of `scr_init()` and `scr_restore()`.
    pub fn scr_set(&mut self, filename: &str) -> Result<()> {
        self.scr_restore(filename)?;
        // Also update curscr to match
        let lines = self.newscr.getmaxy();
        let cols = self.newscr.getmaxx();
        for y in 0..lines {
            if let (Some(src_line), Some(dst_line)) = (
                self.newscr.line(y as usize),
                self.curscr.line_mut(y as usize),
            ) {
                for x in 0..cols {
                    dst_line.set(x as usize, src_line.get(x as usize));
                }
            }
        }
        Ok(())
    }

    /// Save a window to a file.
    ///
    /// This saves the window contents in a format that can be restored
    /// with `getwin()`.
    pub fn putwin(&self, win: &Window, filename: &str) -> Result<()> {
        use std::fs::File;
        use std::io::Write;

        let mut file = File::create(filename)?;

        // Write header
        let lines = win.getmaxy();
        let cols = win.getmaxx();
        let begy = win.getbegy();
        let begx = win.getbegx();

        // Format: NCWIN + version(1) + lines(4) + cols(4) + begy(4) + begx(4) + data
        file.write_all(b"NCWIN")?;
        file.write_all(&[1u8])?;
        file.write_all(&(lines as u32).to_le_bytes())?;
        file.write_all(&(cols as u32).to_le_bytes())?;
        file.write_all(&begy.to_le_bytes())?;
        file.write_all(&begx.to_le_bytes())?;

        // Write window data
        for y in 0..lines {
            if let Some(line) = win.line(y as usize) {
                for x in 0..cols {
                    let ch = line.get(x as usize);
                    #[cfg(not(feature = "wide"))]
                    {
                        file.write_all(&(ch as u32).to_le_bytes())?;
                    }
                    #[cfg(feature = "wide")]
                    {
                        let c = ch.chars[0] as u32;
                        let a = ch.attr;
                        file.write_all(&c.to_le_bytes())?;
                        file.write_all(&a.to_le_bytes())?;
                    }
                }
            }
        }

        file.flush()?;
        Ok(())
    }

    /// Restore a window from a file.
    ///
    /// This creates a new window with the contents saved by `putwin()`.
    pub fn getwin(&self, filename: &str) -> Result<Window> {
        use std::fs::File;
        use std::io::Read;

        let mut file = File::open(filename)?;

        // Read and verify header
        let mut magic = [0u8; 5];
        file.read_exact(&mut magic)?;
        if &magic != b"NCWIN" {
            return Err(Error::InvalidArgument(
                "Invalid window dump file".to_string(),
            ));
        }

        let mut version = [0u8; 1];
        file.read_exact(&mut version)?;
        if version[0] != 1 {
            return Err(Error::InvalidArgument(
                "Unsupported dump file version".to_string(),
            ));
        }

        let mut lines_bytes = [0u8; 4];
        let mut cols_bytes = [0u8; 4];
        let mut begy_bytes = [0u8; 4];
        let mut begx_bytes = [0u8; 4];
        file.read_exact(&mut lines_bytes)?;
        file.read_exact(&mut cols_bytes)?;
        file.read_exact(&mut begy_bytes)?;
        file.read_exact(&mut begx_bytes)?;

        let lines = u32::from_le_bytes(lines_bytes) as i32;
        let cols = u32::from_le_bytes(cols_bytes) as i32;
        let begy = i32::from_le_bytes(begy_bytes);
        let begx = i32::from_le_bytes(begx_bytes);

        // Create the window
        let mut win = Window::new(lines, cols, begy, begx)?;

        // Read window data
        for y in 0..lines {
            for x in 0..cols {
                #[cfg(not(feature = "wide"))]
                {
                    let mut ch_bytes = [0u8; 4];
                    file.read_exact(&mut ch_bytes)?;
                    let ch = u32::from_le_bytes(ch_bytes) as ChType;

                    if let Some(line) = win.line_mut(y as usize) {
                        line.set(x as usize, ch);
                    }
                }
                #[cfg(feature = "wide")]
                {
                    let mut c_bytes = [0u8; 4];
                    let mut a_bytes = [0u8; 4];
                    file.read_exact(&mut c_bytes)?;
                    file.read_exact(&mut a_bytes)?;
                    let c = u32::from_le_bytes(c_bytes);
                    let a = u32::from_le_bytes(a_bytes);

                    if let Some(line) = win.line_mut(y as usize) {
                        let mut cchar = crate::wide::CCharT::default();
                        cchar.chars[0] = char::from_u32(c).unwrap_or(' ');
                        cchar.attr = a;
                        line.set(x as usize, cchar);
                    }
                }
            }
        }

        Ok(win)
    }

    // ========================================================================
    // Deprecated/Unavailable C-style functions
    // ========================================================================
    // These functions exist in C ncurses but don't translate well to Rust.
    // The printw family is implemented above with working alternatives.
    // The scanw family below provides compile-time errors with guidance.

    /// **Not available in Rust** - Use `getstr()` and parse manually instead.
    ///
    /// In C ncurses, `scanw` reads formatted input:
    /// ```c
    /// int value;
    /// scanw("%d", &value);
    /// ```
    ///
    /// In Rust, use `getstr` and parse:
    /// ```rust,ignore
    /// let input = screen.getstr(100)?;
    /// let value: i32 = input.trim().parse().unwrap_or(0);
    /// ```
    ///
    /// # Panics
    /// Always panics - this function is not implemented.
    #[deprecated(since = "0.1.0", note = "Use getstr() and parse the result instead")]
    pub fn scanw(&mut self, _fmt: &str) -> Result<()> {
        panic!(
            "scanw is not available in Rust. Use getstr() and parse the result instead.\n\
             Example:\n\
             let input = screen.getstr(100)?;\n\
             let value: i32 = input.trim().parse().unwrap_or(0);"
        );
    }

    /// **Not available in Rust** - Use `wgetstr()` and parse manually instead.
    ///
    /// In C ncurses, `wscanw` reads formatted input from a window:
    /// ```c
    /// int value;
    /// wscanw(win, "%d", &value);
    /// ```
    ///
    /// In Rust, use `wgetstr` and parse:
    /// ```rust,ignore
    /// let input = screen.wgetstr(win, 100)?;
    /// let value: i32 = input.trim().parse().unwrap_or(0);
    /// ```
    ///
    /// # Panics
    /// Always panics - this function is not implemented.
    #[deprecated(since = "0.1.0", note = "Use wgetstr() and parse the result instead")]
    pub fn wscanw(&mut self, _win: &mut Window, _fmt: &str) -> Result<()> {
        panic!(
            "wscanw is not available in Rust. Use wgetstr() and parse the result instead.\n\
             Example:\n\
             let input = screen.wgetstr(win, 100)?;\n\
             let value: i32 = input.trim().parse().unwrap_or(0);"
        );
    }

    /// **Not available in Rust** - Use `mvgetstr()` and parse manually instead.
    ///
    /// # Panics
    /// Always panics - this function is not implemented.
    #[deprecated(
        since = "0.1.0",
        note = "Use mv() + getstr() and parse the result instead"
    )]
    pub fn mvscanw(&mut self, _y: i32, _x: i32, _fmt: &str) -> Result<()> {
        panic!(
            "mvscanw is not available in Rust. Use mv() + getstr() and parse instead.\n\
             Example:\n\
             screen.mv(y, x)?;\n\
             let input = screen.getstr(100)?;\n\
             let value: i32 = input.trim().parse().unwrap_or(0);"
        );
    }

    /// **Not available in Rust** - Use `wmove()` + `wgetstr()` and parse manually instead.
    ///
    /// # Panics
    /// Always panics - this function is not implemented.
    #[deprecated(
        since = "0.1.0",
        note = "Use wmove() + wgetstr() and parse the result instead"
    )]
    pub fn mvwscanw(&mut self, _win: &mut Window, _y: i32, _x: i32, _fmt: &str) -> Result<()> {
        panic!(
            "mvwscanw is not available in Rust. Use wmove() + wgetstr() and parse instead.\n\
             Example:\n\
             screen.wmove(win, y, x)?;\n\
             let input = screen.wgetstr(win, 100)?;\n\
             let value: i32 = input.trim().parse().unwrap_or(0);"
        );
    }

    // ========================================================================
    // Alias functions for ncurses API compatibility
    // ========================================================================

    /// Move the cursor on stdscr.
    ///
    /// This is equivalent to `wmove(stdscr, y, x)`.
    pub fn r#move(&mut self, y: i32, x: i32) -> Result<()> {
        self.stdscr.mv(y, x)
    }

    /// Turn off standout mode on stdscr.
    pub fn standend(&mut self) -> Result<()> {
        self.stdscr.standend()
    }

    /// Turn on standout mode on stdscr.
    pub fn standout(&mut self) -> Result<()> {
        self.stdscr.standout()
    }

    /// Turn off standout mode on a window.
    pub fn wstandend(&mut self, win: &mut Window) -> Result<()> {
        win.standend()
    }

    /// Turn on standout mode on a window.
    pub fn wstandout(&mut self, win: &mut Window) -> Result<()> {
        win.standout()
    }

    // ========================================================================
    // Attribute get/set functions (attr_* family)
    // ========================================================================

    /// Get the current attributes and color pair from stdscr.
    pub fn attr_get(&self, attrs: &mut AttrT, pair: &mut i16) -> Result<()> {
        *attrs = self.stdscr.getattrs();
        *pair = attr::pair_number(*attrs);
        Ok(())
    }

    /// Turn off attributes on stdscr.
    pub fn attr_off(&mut self, attrs: AttrT) -> Result<()> {
        self.stdscr.attroff(attrs)
    }

    /// Turn on attributes on stdscr.
    pub fn attr_on(&mut self, attrs: AttrT) -> Result<()> {
        self.stdscr.attron(attrs)
    }

    /// Set attributes on stdscr.
    pub fn attr_set(&mut self, attrs: AttrT, pair: i16) -> Result<()> {
        let combined = attrs | attr::color_pair(pair);
        self.stdscr.attrset(combined)
    }

    /// Get the current attributes and color pair from a window.
    pub fn wattr_get(&self, win: &Window, attrs: &mut AttrT, pair: &mut i16) -> Result<()> {
        *attrs = win.getattrs();
        *pair = attr::pair_number(*attrs);
        Ok(())
    }

    /// Turn off attributes on a window.
    pub fn wattr_off(&mut self, win: &mut Window, attrs: AttrT) -> Result<()> {
        win.attroff(attrs)
    }

    /// Turn on attributes on a window.
    pub fn wattr_on(&mut self, win: &mut Window, attrs: AttrT) -> Result<()> {
        win.attron(attrs)
    }

    /// Set attributes on a window.
    pub fn wattr_set(&mut self, win: &mut Window, attrs: AttrT, pair: i16) -> Result<()> {
        let combined = attrs | attr::color_pair(pair);
        win.attrset(combined)
    }

    // ========================================================================
    // Move + operation wrappers (mv* family)
    // ========================================================================

    /// Move cursor and delete character on stdscr.
    pub fn mvdelch(&mut self, y: i32, x: i32) -> Result<()> {
        self.stdscr.mv(y, x)?;
        self.stdscr.delch()
    }

    /// Move cursor and draw horizontal line on stdscr.
    pub fn mvhline(&mut self, y: i32, x: i32, ch: ChType, n: i32) -> Result<()> {
        self.stdscr.mv(y, x)?;
        self.stdscr.hline(ch, n)
    }

    /// Move cursor and draw vertical line on stdscr.
    pub fn mvvline(&mut self, y: i32, x: i32, ch: ChType, n: i32) -> Result<()> {
        self.stdscr.mv(y, x)?;
        self.stdscr.vline(ch, n)
    }

    /// Move cursor and insert character on stdscr.
    pub fn mvinsch(&mut self, y: i32, x: i32, ch: ChType) -> Result<()> {
        self.stdscr.mv(y, x)?;
        self.stdscr.insch(ch)
    }

    /// Move cursor and add a string of chtype on stdscr.
    pub fn mvaddchstr(&mut self, y: i32, x: i32, chstr: &[ChType]) -> Result<()> {
        self.stdscr.mv(y, x)?;
        self.stdscr.addchnstr(chstr, -1)
    }

    /// Move cursor and add at most n chtype on stdscr.
    pub fn mvaddchnstr(&mut self, y: i32, x: i32, chstr: &[ChType], n: i32) -> Result<()> {
        self.stdscr.mv(y, x)?;
        self.stdscr.addchnstr(chstr, n)
    }

    /// Move cursor and add a string of chtype on a window.
    pub fn mvwaddchstr(
        &mut self,
        win: &mut Window,
        y: i32,
        x: i32,
        chstr: &[ChType],
    ) -> Result<()> {
        win.mv(y, x)?;
        win.addchnstr(chstr, -1)
    }

    /// Move cursor and add at most n chtype on a window.
    pub fn mvwaddchnstr(
        &mut self,
        win: &mut Window,
        y: i32,
        x: i32,
        chstr: &[ChType],
        n: i32,
    ) -> Result<()> {
        win.mv(y, x)?;
        win.addchnstr(chstr, n)
    }

    /// Move a derived window relative to its parent.
    pub fn mvderwin(&mut self, win: &mut Window, y: i32, x: i32) -> Result<()> {
        win.mvderwin(y, x)
    }

    // ========================================================================
    // Unbounded *chstr variants (read to end of line)
    // ========================================================================

    /// Get a string of characters with attributes from stdscr (unbounded).
    pub fn inchstr(&self, chstr: &mut [ChType]) -> i32 {
        self.stdscr.inchnstr(chstr, -1)
    }

    /// Move and get a string of characters with attributes from stdscr (unbounded).
    pub fn mvinchstr(&mut self, y: i32, x: i32, chstr: &mut [ChType]) -> Result<i32> {
        self.stdscr.mv(y, x)?;
        Ok(self.stdscr.inchnstr(chstr, -1))
    }

    /// Get a string of characters with attributes from a window (unbounded).
    pub fn winchstr(&self, win: &Window, chstr: &mut [ChType]) -> i32 {
        win.inchnstr(chstr, -1)
    }

    /// Move and get a string of characters with attributes from a window (unbounded).
    pub fn mvwinchstr(
        &mut self,
        win: &mut Window,
        y: i32,
        x: i32,
        chstr: &mut [ChType],
    ) -> Result<i32> {
        win.mv(y, x)?;
        Ok(win.inchnstr(chstr, -1))
    }

    // ========================================================================
    // Additional string input functions (getstr family variants)
    // ========================================================================

    /// Get a string from stdscr with a specific length limit.
    ///
    /// This is an alias for `getstr()`.
    pub fn getnstr(&mut self, maxlen: usize) -> Result<String> {
        self.getstr(maxlen)
    }

    /// Move cursor and get a string from stdscr.
    pub fn mvgetstr(&mut self, y: i32, x: i32, maxlen: usize) -> Result<String> {
        self.stdscr.mv(y, x)?;
        self.getstr(maxlen)
    }

    /// Move cursor and get a string with length limit from stdscr.
    pub fn mvgetnstr(&mut self, y: i32, x: i32, maxlen: usize) -> Result<String> {
        self.mvgetstr(y, x, maxlen)
    }

    /// Get a string from a window with a specific length limit.
    ///
    /// This is an alias for `wgetstr()`.
    pub fn wgetnstr(&mut self, win: &mut Window, maxlen: usize) -> Result<String> {
        self.wgetstr(win, maxlen)
    }

    /// Move cursor and get a string from a window.
    pub fn mvwgetstr(&mut self, win: &mut Window, y: i32, x: i32, maxlen: usize) -> Result<String> {
        win.mv(y, x)?;
        self.wgetstr(win, maxlen)
    }

    /// Move cursor and get a string with length limit from a window.
    pub fn mvwgetnstr(
        &mut self,
        win: &mut Window,
        y: i32,
        x: i32,
        maxlen: usize,
    ) -> Result<String> {
        self.mvwgetstr(win, y, x, maxlen)
    }

    // ========================================================================
    // Unbounded instr variants
    // ========================================================================

    /// Get a string from a window at the current position (unbounded).
    pub fn winstr(&self, win: &Window) -> String {
        win.instr(-1)
    }

    /// Move and get a string from a window (unbounded).
    pub fn mvwinstr(&mut self, win: &mut Window, y: i32, x: i32) -> Result<String> {
        win.mv(y, x)?;
        Ok(win.instr(-1))
    }

    // ========================================================================
    // Sync functions
    // ========================================================================

    /// Synchronize cursor position with ancestors.
    pub fn wcursyncup(&mut self, _win: &mut Window) {
        // In our implementation, windows don't share storage with parents,
        // so this is a no-op. The cursor position is local to each window.
    }

    /// Synchronize window with descendants.
    pub fn wsyncdown(&mut self, _win: &mut Window) {
        // In our implementation, windows don't share storage with parents,
        // so this is a no-op.
    }

    /// Synchronize window with ancestors.
    pub fn wsyncup(&mut self, _win: &mut Window) {
        // In our implementation, windows don't share storage with parents,
        // so this is a no-op.
    }

    /// Touch a line range in a window.
    pub fn touchline(&mut self, win: &mut Window, start: i32, count: i32) -> Result<()> {
        win.touchln(start, count, true);
        Ok(())
    }

    // ========================================================================
    // Mouse transformation
    // ========================================================================

    /// Transform mouse coordinates from screen to window coordinates.
    ///
    /// If `to_screen` is true, transforms from window to screen coordinates.
    /// Returns true if the coordinates are within the window.
    #[cfg(feature = "mouse")]
    pub fn mouse_trafo(&self, y: &mut i32, x: &mut i32, to_screen: bool) -> bool {
        self.wmouse_trafo(&self.stdscr, y, x, to_screen)
    }

    /// Transform mouse coordinates for a specific window.
    ///
    /// If `to_screen` is true, transforms from window to screen coordinates.
    /// Returns true if the coordinates are within the window.
    #[cfg(feature = "mouse")]
    pub fn wmouse_trafo(&self, win: &Window, y: &mut i32, x: &mut i32, to_screen: bool) -> bool {
        if to_screen {
            // Window to screen
            *y += win.getbegy();
            *x += win.getbegx();
            true
        } else {
            // Screen to window
            let wy = *y - win.getbegy();
            let wx = *x - win.getbegx();
            if wy >= 0 && wy <= win.getmaxy() && wx >= 0 && wx <= win.getmaxx() {
                *y = wy;
                *x = wx;
                true
            } else {
                false
            }
        }
    }

    // ========================================================================
    // Wide character functions
    // ========================================================================

    /// Set the background character (wide) on stdscr.
    #[cfg(feature = "wide")]
    pub fn bkgrnd(&mut self, wch: &crate::wide::CCharT) -> Result<()> {
        self.stdscr.wbkgrnd(wch)
    }

    /// Get a wide character and attributes at the current position.
    #[cfg(feature = "wide")]
    pub fn getcchar(
        wch: &crate::wide::CCharT,
        wc: &mut [char],
        attrs: &mut AttrT,
        color_pair: &mut i16,
    ) -> Result<()> {
        if !wc.is_empty() {
            wc[0] = wch.spacing_char();
        }
        *attrs = wch.attrs();
        *color_pair = attr::pair_number(*attrs);
        Ok(())
    }

    /// Create a complex character from components.
    #[cfg(feature = "wide")]
    pub fn setcchar(
        wch: &mut crate::wide::CCharT,
        wc: &[char],
        attrs: AttrT,
        color_pair: i16,
    ) -> Result<()> {
        let c = wc.first().copied().unwrap_or(' ');
        *wch = crate::wide::CCharT::from_char_attr(c, attrs | attr::color_pair(color_pair));
        Ok(())
    }

    /// Move and get a wide character from stdscr.
    #[cfg(feature = "wide")]
    pub fn mvin_wch(&mut self, y: i32, x: i32, wcval: &mut crate::wide::CCharT) -> Result<()> {
        self.stdscr.mv(y, x)?;
        *wcval = self.stdscr.in_wch();
        Ok(())
    }

    /// Move and get a wide character from a window.
    #[cfg(feature = "wide")]
    pub fn mvwin_wch(
        &mut self,
        win: &mut Window,
        y: i32,
        x: i32,
        wcval: &mut crate::wide::CCharT,
    ) -> Result<()> {
        win.mv(y, x)?;
        *wcval = win.in_wch();
        Ok(())
    }

    // ========================================================================
    // Wide string input/output functions
    // ========================================================================

    /// Get a wide string from stdscr.
    #[cfg(feature = "wide")]
    pub fn get_wstr(&mut self, maxlen: i32) -> Result<String> {
        use crate::wide::WideInput;

        let mut result = String::new();
        let max = if maxlen < 0 { 1024 } else { maxlen as usize };

        // Get delay and keypad settings from stdscr
        let delay = Delay::from_raw(self.stdscr.getdelay());
        let use_keypad = self.stdscr.is_keypad();

        loop {
            if result.len() >= max {
                break;
            }

            match self.get_wch_internal(delay, use_keypad)? {
                WideInput::Char(c) => {
                    if c == '\n' {
                        break;
                    }
                    if c == '\x7f' || c == '\x08' {
                        result.pop();
                        continue;
                    }
                    result.push(c);
                }
                WideInput::Key(k) => {
                    if k == crate::key::KEY_ENTER {
                        break;
                    }
                }
                WideInput::None | WideInput::Eof | WideInput::Error => break,
            }
        }

        Ok(result)
    }

    /// Get a wide string with length limit from stdscr.
    #[cfg(feature = "wide")]
    pub fn getn_wstr(&mut self, maxlen: i32) -> Result<String> {
        self.get_wstr(maxlen)
    }

    /// Move and get a wide string from stdscr.
    #[cfg(feature = "wide")]
    pub fn mvget_wstr(&mut self, y: i32, x: i32, maxlen: i32) -> Result<String> {
        self.stdscr.mv(y, x)?;
        self.get_wstr(maxlen)
    }

    /// Move and get a wide string with length limit from stdscr.
    #[cfg(feature = "wide")]
    pub fn mvgetn_wstr(&mut self, y: i32, x: i32, maxlen: i32) -> Result<String> {
        self.mvget_wstr(y, x, maxlen)
    }

    /// Get a wide string from a window.
    #[cfg(feature = "wide")]
    pub fn wget_wstr(&mut self, win: &mut Window, maxlen: i32) -> Result<String> {
        use crate::wide::WideInput;

        let mut result = String::new();
        let max = if maxlen < 0 { 1024 } else { maxlen as usize };

        loop {
            if result.len() >= max {
                break;
            }

            match self.wget_wch(win)? {
                WideInput::Char(c) => {
                    if c == '\n' {
                        break;
                    }
                    if c == '\x7f' || c == '\x08' {
                        result.pop();
                        continue;
                    }
                    result.push(c);
                }
                WideInput::Key(k) => {
                    if k == crate::key::KEY_ENTER {
                        break;
                    }
                }
                WideInput::None | WideInput::Eof | WideInput::Error => break,
            }
        }

        Ok(result)
    }

    /// Get a wide string with length limit from a window.
    #[cfg(feature = "wide")]
    pub fn wgetn_wstr(&mut self, win: &mut Window, maxlen: i32) -> Result<String> {
        self.wget_wstr(win, maxlen)
    }

    /// Move and get a wide string from a window.
    #[cfg(feature = "wide")]
    pub fn mvwget_wstr(&mut self, win: &mut Window, y: i32, x: i32, maxlen: i32) -> Result<String> {
        win.mv(y, x)?;
        self.wget_wstr(win, maxlen)
    }

    /// Move and get a wide string with length limit from a window.
    #[cfg(feature = "wide")]
    pub fn mvwgetn_wstr(
        &mut self,
        win: &mut Window,
        y: i32,
        x: i32,
        maxlen: i32,
    ) -> Result<String> {
        self.mvwget_wstr(win, y, x, maxlen)
    }

    /// Get a wide character string from stdscr (unbounded).
    #[cfg(feature = "wide")]
    pub fn inwstr(&self) -> String {
        self.winnwstr(&self.stdscr, -1)
    }

    /// Get a wide character string from stdscr with limit.
    #[cfg(feature = "wide")]
    pub fn innwstr(&self, n: i32) -> String {
        self.winnwstr(&self.stdscr, n)
    }

    /// Move and get a wide character string from stdscr.
    #[cfg(feature = "wide")]
    pub fn mvinwstr(&mut self, y: i32, x: i32) -> Result<String> {
        self.stdscr.mv(y, x)?;
        Ok(self.stdscr.instr(-1))
    }

    /// Move and get a wide character string from stdscr with limit.
    #[cfg(feature = "wide")]
    pub fn mvinnwstr(&mut self, y: i32, x: i32, n: i32) -> Result<String> {
        self.stdscr.mv(y, x)?;
        Ok(self.stdscr.instr(n))
    }

    /// Get a wide character string from a window (unbounded).
    #[cfg(feature = "wide")]
    pub fn winwstr(&self, win: &Window) -> String {
        self.winnwstr(win, -1)
    }

    /// Get a wide character string from a window with limit.
    #[cfg(feature = "wide")]
    pub fn winnwstr(&self, win: &Window, n: i32) -> String {
        win.instr(n)
    }

    /// Move and get a wide character string from a window.
    #[cfg(feature = "wide")]
    pub fn mvwinwstr(&mut self, win: &mut Window, y: i32, x: i32) -> Result<String> {
        self.mvwinnwstr(win, y, x, -1)
    }

    /// Move and get a wide character string from a window with limit.
    #[cfg(feature = "wide")]
    pub fn mvwinnwstr(&mut self, win: &mut Window, y: i32, x: i32, n: i32) -> Result<String> {
        win.mv(y, x)?;
        Ok(win.instr(n))
    }

    /// Insert a wide string on stdscr.
    #[cfg(feature = "wide")]
    pub fn ins_wstr(&mut self, wstr: &str) -> Result<()> {
        self.ins_nwstr(wstr, -1)
    }

    /// Insert at most n characters of a wide string on stdscr.
    #[cfg(feature = "wide")]
    pub fn ins_nwstr(&mut self, wstr: &str, n: i32) -> Result<()> {
        let chars: Vec<char> = wstr.chars().collect();
        let limit = if n < 0 {
            chars.len()
        } else {
            (n as usize).min(chars.len())
        };
        for c in chars.into_iter().take(limit).rev() {
            let wch = crate::wide::CCharT::from_char(c);
            self.stdscr.ins_wch(&wch)?;
        }
        Ok(())
    }

    /// Move and insert a wide string on stdscr.
    #[cfg(feature = "wide")]
    pub fn mvins_wstr(&mut self, y: i32, x: i32, wstr: &str) -> Result<()> {
        self.stdscr.mv(y, x)?;
        self.ins_wstr(wstr)
    }

    /// Move and insert at most n characters of a wide string on stdscr.
    #[cfg(feature = "wide")]
    pub fn mvins_nwstr(&mut self, y: i32, x: i32, wstr: &str, n: i32) -> Result<()> {
        self.stdscr.mv(y, x)?;
        self.ins_nwstr(wstr, n)
    }

    /// Insert a wide string on a window.
    #[cfg(feature = "wide")]
    pub fn wins_wstr(&mut self, win: &mut Window, wstr: &str) -> Result<()> {
        self.wins_nwstr(win, wstr, -1)
    }

    /// Insert at most n characters of a wide string on a window.
    #[cfg(feature = "wide")]
    pub fn wins_nwstr(&mut self, win: &mut Window, wstr: &str, n: i32) -> Result<()> {
        let chars: Vec<char> = wstr.chars().collect();
        let limit = if n < 0 {
            chars.len()
        } else {
            (n as usize).min(chars.len())
        };
        for c in chars.into_iter().take(limit).rev() {
            let wch = crate::wide::CCharT::from_char(c);
            win.ins_wch(&wch)?;
        }
        Ok(())
    }

    /// Move and insert a wide string on a window.
    #[cfg(feature = "wide")]
    pub fn mvwins_wstr(&mut self, win: &mut Window, y: i32, x: i32, wstr: &str) -> Result<()> {
        win.mv(y, x)?;
        self.wins_wstr(win, wstr)
    }

    /// Move and insert at most n characters of a wide string on a window.
    #[cfg(feature = "wide")]
    pub fn mvwins_nwstr(
        &mut self,
        win: &mut Window,
        y: i32,
        x: i32,
        wstr: &str,
        n: i32,
    ) -> Result<()> {
        win.mv(y, x)?;
        self.wins_nwstr(win, wstr, n)
    }

    // ========================================================================
    // Wide character array functions (in_wch* family)
    // ========================================================================

    /// Get a wide character array from stdscr (unbounded).
    #[cfg(feature = "wide")]
    pub fn in_wchstr(&self, wchstr: &mut [crate::wide::CCharT]) -> i32 {
        self.win_wchnstr(&self.stdscr, wchstr, -1)
    }

    /// Get a wide character array from stdscr with limit.
    #[cfg(feature = "wide")]
    pub fn in_wchnstr(&self, wchstr: &mut [crate::wide::CCharT], n: i32) -> i32 {
        self.win_wchnstr(&self.stdscr, wchstr, n)
    }

    /// Move and get a wide character array from stdscr (unbounded).
    #[cfg(feature = "wide")]
    pub fn mvin_wchstr(
        &mut self,
        y: i32,
        x: i32,
        wchstr: &mut [crate::wide::CCharT],
    ) -> Result<i32> {
        self.stdscr.mv(y, x)?;
        Ok(self.stdscr.in_wchnstr(wchstr, -1))
    }

    /// Move and get a wide character array from stdscr with limit.
    #[cfg(feature = "wide")]
    pub fn mvin_wchnstr(
        &mut self,
        y: i32,
        x: i32,
        wchstr: &mut [crate::wide::CCharT],
        n: i32,
    ) -> Result<i32> {
        self.stdscr.mv(y, x)?;
        Ok(self.stdscr.in_wchnstr(wchstr, n))
    }

    /// Get a wide character array from a window (unbounded).
    #[cfg(feature = "wide")]
    pub fn win_wchstr(&self, win: &Window, wchstr: &mut [crate::wide::CCharT]) -> i32 {
        self.win_wchnstr(win, wchstr, -1)
    }

    /// Get a wide character array from a window with limit.
    #[cfg(feature = "wide")]
    pub fn win_wchnstr(&self, win: &Window, wchstr: &mut [crate::wide::CCharT], n: i32) -> i32 {
        win.in_wchnstr(wchstr, n)
    }

    /// Move and get a wide character array from a window (unbounded).
    #[cfg(feature = "wide")]
    pub fn mvwin_wchstr(
        &mut self,
        win: &mut Window,
        y: i32,
        x: i32,
        wchstr: &mut [crate::wide::CCharT],
    ) -> Result<i32> {
        self.mvwin_wchnstr(win, y, x, wchstr, -1)
    }

    /// Move and get a wide character array from a window with limit.
    #[cfg(feature = "wide")]
    pub fn mvwin_wchnstr(
        &mut self,
        win: &mut Window,
        y: i32,
        x: i32,
        wchstr: &mut [crate::wide::CCharT],
        n: i32,
    ) -> Result<i32> {
        win.mv(y, x)?;
        Ok(win.in_wchnstr(wchstr, n))
    }

    // ========================================================================
    // Wide border/line functions with complex characters
    // ========================================================================

    /// Draw a horizontal line with a complex character on stdscr.
    #[cfg(feature = "wide")]
    pub fn hline_set(&mut self, wch: &crate::wide::CCharT, n: i32) -> Result<()> {
        self.stdscr.hline_set(wch, n)
    }

    /// Move and draw a horizontal line with a complex character on stdscr.
    #[cfg(feature = "wide")]
    pub fn mvhline_set(&mut self, y: i32, x: i32, wch: &crate::wide::CCharT, n: i32) -> Result<()> {
        self.stdscr.mv(y, x)?;
        self.hline_set(wch, n)
    }

    /// Draw a horizontal line with a complex character on a window.
    #[cfg(feature = "wide")]
    pub fn whline_set(
        &mut self,
        win: &mut Window,
        wch: &crate::wide::CCharT,
        n: i32,
    ) -> Result<()> {
        win.hline_set(wch, n)
    }

    /// Move and draw a horizontal line with a complex character on a window.
    #[cfg(feature = "wide")]
    pub fn mvwhline_set(
        &mut self,
        win: &mut Window,
        y: i32,
        x: i32,
        wch: &crate::wide::CCharT,
        n: i32,
    ) -> Result<()> {
        win.mv(y, x)?;
        win.hline_set(wch, n)
    }

    /// Draw a vertical line with a complex character on stdscr.
    #[cfg(feature = "wide")]
    pub fn vline_set(&mut self, wch: &crate::wide::CCharT, n: i32) -> Result<()> {
        self.stdscr.vline_set(wch, n)
    }

    /// Move and draw a vertical line with a complex character on stdscr.
    #[cfg(feature = "wide")]
    pub fn mvvline_set(&mut self, y: i32, x: i32, wch: &crate::wide::CCharT, n: i32) -> Result<()> {
        self.stdscr.mv(y, x)?;
        self.vline_set(wch, n)
    }

    /// Draw a vertical line with a complex character on a window.
    #[cfg(feature = "wide")]
    pub fn wvline_set(
        &mut self,
        win: &mut Window,
        wch: &crate::wide::CCharT,
        n: i32,
    ) -> Result<()> {
        win.vline_set(wch, n)
    }

    /// Move and draw a vertical line with a complex character on a window.
    #[cfg(feature = "wide")]
    pub fn mvwvline_set(
        &mut self,
        win: &mut Window,
        y: i32,
        x: i32,
        wch: &crate::wide::CCharT,
        n: i32,
    ) -> Result<()> {
        win.mv(y, x)?;
        win.vline_set(wch, n)
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        // Try to restore terminal state
        let _ = self.endwin();
    }
}

// ============================================================================
// Global screen dimensions (thread-local storage)
// ============================================================================

use std::cell::Cell;
use std::cell::RefCell;

thread_local! {
    /// Thread-local storage for screen lines (rows).
    static SCREEN_LINES: Cell<i32> = const { Cell::new(24) };
    /// Thread-local storage for screen columns.
    static SCREEN_COLS: Cell<i32> = const { Cell::new(80) };
    /// Whether to use environment variables (LINES, COLUMNS) for screen size.
    static USE_ENV: Cell<bool> = const { Cell::new(true) };
    /// Whether to use ioctl for screen size detection.
    static USE_TIOCTL: Cell<bool> = const { Cell::new(true) };
    /// Ripoff line specifications (called before initscr).
    static RIPOFF_LINES: RefCell<Vec<RipoffSpec>> = const { RefCell::new(Vec::new()) };
}

/// Specification for a ripped-off line.
#[derive(Clone)]
struct RipoffSpec {
    /// Positive for top, negative for bottom.
    line: i32,
    /// Initialization callback (not used in Rust, but kept for API compatibility).
    #[allow(dead_code)]
    callback: Option<fn(&mut Window, i32) -> i32>,
}

/// Global-style functions for ncurses compatibility.
///
/// These provide access to screen dimensions without needing a Screen reference.
/// Note that in Rust, using the Screen instance methods (`screen.lines()` and
/// `screen.cols()`) is preferred over these globals.
///
/// # Thread Safety
///
/// These values are stored in thread-local storage, so each thread has its own
/// copy. If you're using multiple screens in different threads, each thread
/// will have the dimensions of the last Screen initialized in that thread.
///
/// # Example
///
/// ```rust,ignore
/// use ncurses::screen::*;
///
/// let screen = Screen::init()?;
/// // After init, LINES() and COLS() return the screen dimensions
/// let lines = LINES();
/// let cols = COLS();
/// ```
pub mod globals {
    use super::*;

    /// Get the number of screen lines (rows).
    ///
    /// This returns the value set by the last Screen initialization in this thread.
    /// Prefer using `screen.lines()` when you have a Screen reference.
    #[allow(non_snake_case)]
    pub fn LINES() -> i32 {
        SCREEN_LINES.with(|l| l.get())
    }

    /// Get the number of screen columns.
    ///
    /// This returns the value set by the last Screen initialization in this thread.
    /// Prefer using `screen.cols()` when you have a Screen reference.
    #[allow(non_snake_case)]
    pub fn COLS() -> i32 {
        SCREEN_COLS.with(|c| c.get())
    }

    /// Set the screen dimensions (called internally by Screen).
    #[doc(hidden)]
    pub fn set_dimensions(lines: i32, cols: i32) {
        SCREEN_LINES.with(|l| l.set(lines));
        SCREEN_COLS.with(|c| c.set(cols));
    }

    /// Control whether environment variables are used for screen size.
    ///
    /// This function must be called **before** `Screen::init()` to have an effect.
    ///
    /// When `f` is `true` (the default), the LINES and COLUMNS environment
    /// variables are used to determine the screen size. When `false`, only
    /// the terminfo/termcap values and ioctl are used.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ncurses::screen::globals::use_env;
    ///
    /// // Don't use LINES/COLUMNS environment variables
    /// use_env(false);
    ///
    /// let screen = Screen::init()?;
    /// ```
    pub fn use_env(f: bool) {
        USE_ENV.with(|v| v.set(f));
    }

    /// Get the current use_env setting.
    pub fn get_use_env() -> bool {
        USE_ENV.with(|v| v.get())
    }

    /// Control whether ioctl is used for screen size detection.
    ///
    /// This function must be called **before** `Screen::init()` to have an effect.
    ///
    /// When `f` is `true` (the default), the terminal ioctl (TIOCGWINSZ) is
    /// used to get the actual terminal size. When `false`, only the terminfo
    /// values and environment variables (if enabled) are used.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ncurses::screen::globals::use_tioctl;
    ///
    /// // Don't use ioctl for size detection
    /// use_tioctl(false);
    ///
    /// let screen = Screen::init()?;
    /// ```
    pub fn use_tioctl(f: bool) {
        USE_TIOCTL.with(|v| v.set(f));
    }

    /// Get the current use_tioctl setting.
    pub fn get_use_tioctl() -> bool {
        USE_TIOCTL.with(|v| v.get())
    }

    /// Rip off a line from the top or bottom of the screen.
    ///
    /// This function must be called **before** `Screen::init()` to have an effect.
    /// It reduces the size of stdscr by one line and creates a separate window
    /// for that line.
    ///
    /// # Arguments
    ///
    /// * `line` - If positive, rip off a line from the top of the screen.
    ///   If negative, rip off a line from the bottom.
    /// * `init` - An optional initialization callback that will be called with
    ///   the window and its width. In Rust, you typically don't need
    ///   this callback; you can access ripped-off windows after init.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the ripoff was registered, `Err` if the maximum
    /// number of ripoffs (5) has been reached.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ncurses::screen::globals::ripoffline;
    ///
    /// // Reserve a line at the top for a header
    /// ripoffline(1, None)?;
    ///
    /// // Reserve a line at the bottom for a status bar
    /// ripoffline(-1, None)?;
    ///
    /// let screen = Screen::init()?;
    /// ```
    ///
    /// # Note
    ///
    /// In ncurses-pure, the callback argument is optional and mostly for API
    /// compatibility. The recommended pattern is to create your own windows
    /// after initialization for header/footer lines.
    pub fn ripoffline(line: i32, init: Option<fn(&mut Window, i32) -> i32>) -> Result<()> {
        const MAX_RIPS: usize = 5;

        if line == 0 {
            return Ok(());
        }

        RIPOFF_LINES.with(|rips| {
            let mut rips = rips.borrow_mut();
            if rips.len() >= MAX_RIPS {
                return Err(Error::InvalidArgument(
                    "maximum ripoff lines reached".to_string(),
                ));
            }
            rips.push(RipoffSpec {
                line: if line > 0 { 1 } else { -1 },
                callback: init,
            });
            Ok(())
        })
    }

    /// Get the number of ripped-off lines.
    #[doc(hidden)]
    pub fn get_ripoff_count() -> (i32, i32) {
        RIPOFF_LINES.with(|rips| {
            let rips = rips.borrow();
            let top = rips.iter().filter(|r| r.line > 0).count() as i32;
            let bottom = rips.iter().filter(|r| r.line < 0).count() as i32;
            (top, bottom)
        })
    }

    /// Clear ripoff specifications (called after Screen::init).
    #[doc(hidden)]
    pub fn clear_ripoffs() {
        RIPOFF_LINES.with(|rips| {
            rips.borrow_mut().clear();
        });
    }
}

// ============================================================================
// Terminfo low-level functions (stubs)
// ============================================================================

/// Initialize the terminal description.
///
/// This is a stub function for ncurses API compatibility.
/// In this implementation, terminal handling is done internally by the Screen.
///
/// # Arguments
///
/// * `_term` - Terminal type name (ignored in this implementation)
/// * `_filedes` - File descriptor for the terminal (ignored in this implementation)
///
/// # Returns
///
/// Always returns `Ok(())` since terminal setup is handled by Screen::init().
pub fn setupterm(_term: Option<&str>, _filedes: i32) -> Result<()> {
    // In our implementation, terminal setup is handled by Screen::init()
    Ok(())
}

/// Set the current terminal.
///
/// This is a stub function for ncurses API compatibility.
/// In this implementation, terminal handling is done internally by the Screen.
///
/// # Returns
///
/// Always returns `Ok(())`.
pub fn set_curterm() -> Result<()> {
    // No-op in our implementation
    Ok(())
}

/// Delete a terminal description.
///
/// This is a stub function for ncurses API compatibility.
/// In this implementation, terminal descriptions are managed automatically.
///
/// # Returns
///
/// Always returns `Ok(())`.
pub fn del_curterm() -> Result<()> {
    // No-op in our implementation - memory is managed automatically
    Ok(())
}

/// Restart terminal setup.
///
/// This is a stub function for ncurses API compatibility.
/// In this implementation, terminal restart is handled by re-initializing Screen.
///
/// # Arguments
///
/// * `_term` - Terminal type name (ignored)
/// * `_filedes` - File descriptor (ignored)
/// * `_errret` - Error return location (ignored)
///
/// # Returns
///
/// Always returns `Ok(())`.
pub fn restartterm(_term: Option<&str>, _filedes: i32, _errret: Option<&mut i32>) -> Result<()> {
    // No-op in our implementation
    Ok(())
}

/// Output video attributes to the terminal.
///
/// This is a stub function for ncurses API compatibility.
/// In our implementation, attribute handling is done through the Screen methods.
///
/// # Arguments
///
/// * `_attrs` - The attributes to set
///
/// # Returns
///
/// Always returns `Ok(())`.
pub fn vidattr(_attrs: AttrT) -> Result<()> {
    // In our implementation, use Screen's attribute methods
    Ok(())
}

/// Output video attributes using a custom output function.
///
/// This is a stub function for ncurses API compatibility.
/// Modern terminals don't need this low-level control.
///
/// # Arguments
///
/// * `_attrs` - The attributes to set
/// * `_putc` - Custom output function (ignored)
///
/// # Returns
///
/// Always returns `Ok(())`.
pub fn vidputs<F>(_attrs: AttrT, _putc: F) -> Result<()>
where
    F: FnMut(i32) -> i32,
{
    // No-op in our implementation
    Ok(())
}

/// Move the physical cursor.
///
/// This is a low-level function that moves the physical terminal cursor
/// from one position to another without going through the curses window system.
///
/// This is a stub that always returns Ok - use Screen's cursor positioning
/// methods for actual cursor control.
///
/// # Arguments
///
/// * `_oldrow` - Current row position
/// * `_oldcol` - Current column position  
/// * `_newrow` - Target row position
/// * `_newcol` - Target column position
///
/// # Returns
///
/// Always returns `Ok(())`.
pub fn mvcur(_oldrow: i32, _oldcol: i32, _newrow: i32, _newcol: i32) -> Result<()> {
    // In our implementation, cursor movement is handled through Screen
    Ok(())
}

// ============================================================================
// Termcap compatibility functions (stubs)
// ============================================================================

/// Get a termcap entry.
///
/// This is a stub for ncurses termcap compatibility.
/// Returns 1 (success) always since our implementation doesn't use termcap.
///
/// # Arguments
///
/// * `_bp` - Buffer for the termcap entry (unused)
/// * `_name` - Terminal name (unused)
///
/// # Returns
///
/// Always returns 1 (success).
pub fn tgetent(_bp: &mut [u8], _name: &str) -> i32 {
    1 // Success
}

/// Get a boolean termcap capability.
///
/// This is a stub for ncurses termcap compatibility.
///
/// # Arguments
///
/// * `_id` - The capability ID
///
/// # Returns
///
/// Always returns 0 (capability not present).
pub fn tgetflag(_id: &str) -> i32 {
    0 // Not present
}

/// Get a numeric termcap capability.
///
/// This is a stub for ncurses termcap compatibility.
///
/// # Arguments
///
/// * `_id` - The capability ID
///
/// # Returns
///
/// Always returns -1 (capability not present).
pub fn tgetnum(_id: &str) -> i32 {
    -1 // Not present
}

/// Get a string termcap capability.
///
/// This is a stub for ncurses termcap compatibility.
///
/// # Arguments
///
/// * `_id` - The capability ID
///
/// # Returns
///
/// Always returns `None` (capability not present).
pub fn tgetstr(_id: &str) -> Option<String> {
    None
}

/// Apply parameters to a termcap string capability.
///
/// This is a stub for ncurses termcap compatibility.
///
/// # Arguments
///
/// * `cap` - The capability string
/// * `_col` - Column parameter
/// * `_row` - Row parameter
///
/// # Returns
///
/// Returns the capability string unchanged.
pub fn tgoto(cap: &str, _col: i32, _row: i32) -> String {
    cap.to_string()
}

/// Output a termcap string with padding.
///
/// This is a stub for ncurses termcap compatibility.
/// Since modern terminals don't need padding, this just outputs the string.
///
/// # Arguments
///
/// * `_str` - The string to output
/// * `_affcnt` - Affected lines count (for padding calculation)
/// * `_putc` - Output function
///
/// # Returns
///
/// Always returns `Ok(())`.
pub fn tputs<F>(_str: &str, _affcnt: i32, _putc: F) -> Result<()>
where
    F: FnMut(i32) -> i32,
{
    // No-op - modern terminals don't need padding
    Ok(())
}

// ============================================================================
// Additional ncurses compatibility functions
// ============================================================================

/// Initialize curses and return the standard screen.
///
/// This is the traditional ncurses initialization function. In ncurses-pure,
/// it's an alias for `Screen::init()`.
///
/// For new code, prefer using `Screen::init()` directly.
pub fn initscr() -> Result<Screen> {
    Screen::init()
}

/// Initialize a new terminal.
///
/// This function creates a new Screen for a specific terminal type and output.
/// In this implementation, it's an alias for `Screen::init()` as we only
/// support the default terminal.
///
/// # Arguments
///
/// * `_type` - Terminal type (ignored, uses $TERM)
/// * `_outf` - Output file (ignored, uses stdout)
/// * `_inf` - Input file (ignored, uses stdin)
///
/// # Returns
///
/// A new Screen instance.
pub fn newterm(_type: Option<&str>, _outf: Option<()>, _inf: Option<()>) -> Result<Screen> {
    Screen::init()
}

/// Set the current terminal (screen).
///
/// In ncurses-pure, there is no global current screen, so this is a no-op stub.
/// Each Screen instance manages its own state.
///
/// # Arguments
///
/// * `_new` - The new screen to set as current (ignored)
///
/// # Returns
///
/// Always returns `Ok(())`.
pub fn set_term(_new: &Screen) -> Result<()> {
    // In our implementation, each Screen is independent
    Ok(())
}

/// Delete a screen.
///
/// In ncurses-pure, screens are automatically cleaned up when dropped.
/// This function is a no-op stub for API compatibility.
///
/// # Arguments
///
/// * `_sp` - The screen to delete (ignored)
pub fn delscreen(_sp: Screen) {
    // Screen is dropped automatically when it goes out of scope
}

/// Delete a window.
///
/// In ncurses-pure, windows are automatically cleaned up when dropped.
/// This function is provided for API compatibility.
///
/// # Arguments
///
/// * `_win` - The window to delete (ignored, will be dropped)
pub fn delwin(_win: Window) {
    // Window is dropped automatically when it goes out of scope
}

/// Get the ncurses version string.
///
/// Returns the version of ncurses-pure.
#[must_use]
pub fn curses_version() -> &'static str {
    concat!("ncurses-pure ", env!("CARGO_PKG_VERSION"))
}

/// Get terminal attributes.
///
/// Returns the video attributes supported by the terminal.
/// In our implementation, we support all standard attributes.
#[must_use]
pub fn termattrs() -> AttrT {
    use crate::attr::*;
    A_STANDOUT | A_UNDERLINE | A_REVERSE | A_BLINK | A_DIM | A_BOLD | A_INVIS | A_ITALIC
}

/// Get terminal attributes (X/Open style).
///
/// Returns the video attributes supported by the terminal.
#[must_use]
pub fn term_attrs() -> AttrT {
    termattrs()
}

/// Get the binding for a key.
///
/// Returns the key definition bound to a specific keycode.
/// In our implementation, key bindings are fixed.
///
/// # Arguments
///
/// * `_keycode` - The key code to query
/// * `_count` - Which binding to get (for keys with multiple bindings)
///
/// # Returns
///
/// Always returns `None` as custom key bindings aren't supported.
pub fn keybound(_keycode: i32, _count: i32) -> Option<String> {
    None
}

/// Get the erase character as a wide character.
///
/// Returns the terminal's erase character (typically backspace).
#[cfg(feature = "wide")]
pub fn killwchar() -> char {
    '\x15' // Ctrl-U
}

/// Enable or disable extended names in terminfo.
///
/// This is a stub for API compatibility. Extended names are not used
/// in this implementation.
///
/// # Arguments
///
/// * `_enable` - Whether to enable extended names
///
/// # Returns
///
/// Always returns 0.
pub fn use_extended_names(_enable: bool) -> i32 {
    0
}

/// Set legacy coding mode.
///
/// This is a stub for API compatibility. Legacy coding is not used
/// in this implementation.
///
/// # Arguments
///
/// * `_level` - The legacy coding level
///
/// # Returns
///
/// Always returns 0.
pub fn use_legacy_coding(_level: i32) -> i32 {
    0
}

/// Output video attributes (X/Open style).
///
/// This is a stub for API compatibility.
///
/// # Arguments
///
/// * `_attrs` - The attributes to output
/// * `_pair` - The color pair
/// * `_opts` - Options (ignored)
///
/// # Returns
///
/// Always returns `Ok(())`.
pub fn vid_attr(_attrs: AttrT, _pair: i16, _opts: Option<()>) -> Result<()> {
    Ok(())
}

/// Output video attributes using a custom function (X/Open style).
///
/// This is a stub for API compatibility.
///
/// # Arguments
///
/// * `_attrs` - The attributes to output
/// * `_pair` - The color pair
/// * `_opts` - Options (ignored)
/// * `_putc` - Output function
///
/// # Returns
///
/// Always returns `Ok(())`.
pub fn vid_puts<F>(_attrs: AttrT, _pair: i16, _opts: Option<()>, _putc: F) -> Result<()>
where
    F: FnMut(i32) -> i32,
{
    Ok(())
}

/// **Not available in Rust** - Use `wprintw()` instead.
///
/// The variadic printw functions cannot be directly implemented in Rust.
/// Use the format! macro and wprintw instead.
///
/// # Panics
/// Always panics - this function is not implemented.
#[deprecated(since = "0.1.0", note = "Use wprintw with format! instead")]
pub fn vwprintw(_win: &mut Window, _fmt: &str) -> Result<()> {
    panic!(
        "vwprintw is not available in Rust. Use wprintw with format! instead.\n\
         Example:\n\
         screen.wprintw(win, &format!(\"Value: {{}}\", value))?;"
    );
}

/// **Not available in Rust** - Use `wprintw()` instead.
///
/// This is an alias for `vwprintw`.
///
/// # Panics
/// Always panics - this function is not implemented.
#[deprecated(since = "0.1.0", note = "Use wprintw with format! instead")]
#[allow(deprecated)]
pub fn vw_printw(_win: &mut Window, _fmt: &str) -> Result<()> {
    vwprintw(_win, _fmt)
}

/// **Not available in Rust** - Use `wgetstr()` and parse instead.
///
/// The variadic scanw functions cannot be directly implemented in Rust.
/// Use wgetstr and parse the input manually.
///
/// # Panics
/// Always panics - this function is not implemented.
#[deprecated(since = "0.1.0", note = "Use wgetstr and parse the result instead")]
pub fn vwscanw(_win: &mut Window, _fmt: &str) -> Result<()> {
    panic!(
        "vwscanw is not available in Rust. Use wgetstr and parse instead.\n\
         Example:\n\
         let input = screen.wgetstr(win, 100)?;\n\
         let value: i32 = input.trim().parse().unwrap_or(0);"
    );
}

/// **Not available in Rust** - Use `wgetstr()` and parse instead.
///
/// This is an alias for `vwscanw`.
///
/// # Panics
/// Always panics - this function is not implemented.
#[deprecated(since = "0.1.0", note = "Use wgetstr and parse the result instead")]
#[allow(deprecated)]
pub fn vw_scanw(_win: &mut Window, _fmt: &str) -> Result<()> {
    vwscanw(_win, _fmt)
}

/// Get printable representation of a character.
///
/// This is the window-less version that operates on characters directly.
pub fn unctrl(c: ChType) -> String {
    let ch = (c & crate::attr::A_CHARTEXT) as u8;
    if ch < 32 {
        format!("^{}", (ch + 64) as char)
    } else if ch == 127 {
        "^?".to_string()
    } else {
        (ch as char).to_string()
    }
}

/// Get printable representation of a wide character.
#[cfg(feature = "wide")]
pub fn wunctrl(wc: &crate::wide::CCharT) -> String {
    let c = wc.spacing_char();
    if c < ' ' {
        format!("^{}", ((c as u8) + 64) as char)
    } else if c as u8 == 127 {
        "^?".to_string()
    } else {
        c.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Most Screen tests require a terminal, so they're integration tests

    #[test]
    fn test_delay_conversion() {
        assert_eq!(Delay::from_raw(0), Delay::NoDelay);
        assert_eq!(Delay::from_raw(-1), Delay::Blocking);
        assert_eq!(Delay::from_raw(100), Delay::Timeout(100));

        assert_eq!(Delay::NoDelay.to_raw(), 0);
        assert_eq!(Delay::Blocking.to_raw(), -1);
        assert_eq!(Delay::Timeout(100).to_raw(), 100);
    }
}
