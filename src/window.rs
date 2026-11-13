//! Window management for ncurses-pure.
//!
//! This module implements the WINDOW structure and all window-related
//! operations as defined by the X/Open XSI Curses standard.

#[cfg(not(feature = "wide"))]
use crate::attr::A_CHARTEXT;
use crate::attr::{self, A_NORMAL, color_pair};
use crate::error::{Error, Result};
use crate::line::LineData;
use crate::types::{AttrT, ChType, NcursesSize, WindowFlags};

#[cfg(feature = "wide")]
use crate::wide::CCharT;

/// Pad-specific data for pad windows.
#[derive(Clone, Copy, Debug, Default)]
pub struct PadData {
    /// Pad Y position for refresh.
    pub pad_y: NcursesSize,
    /// Pad X position for refresh.
    pub pad_x: NcursesSize,
    /// Screen top line for refresh.
    pub pad_top: NcursesSize,
    /// Screen left column for refresh.
    pub pad_left: NcursesSize,
    /// Screen bottom line for refresh.
    pub pad_bottom: NcursesSize,
    /// Screen right column for refresh.
    pub pad_right: NcursesSize,
}

/// A curses window.
///
/// Windows are the fundamental abstraction in curses. They represent a
/// rectangular area of the screen that can be written to and refreshed
/// independently.
pub struct Window {
    // ========================================================================
    // Cursor position
    // ========================================================================
    /// Current cursor Y position (row).
    cury: NcursesSize,
    /// Current cursor X position (column).
    curx: NcursesSize,

    // ========================================================================
    // Window dimensions and location
    // ========================================================================
    /// Maximum Y coordinate (rows - 1).
    maxy: NcursesSize,
    /// Maximum X coordinate (columns - 1).
    maxx: NcursesSize,
    /// Screen Y coordinate of upper-left corner.
    begy: NcursesSize,
    /// Screen X coordinate of upper-left corner.
    begx: NcursesSize,

    // ========================================================================
    // Window state
    // ========================================================================
    /// Window state flags.
    flags: WindowFlags,
    /// Current attributes for non-space characters.
    attrs: AttrT,
    /// Current background character/attribute pair.
    #[cfg(not(feature = "wide"))]
    bkgd: ChType,
    #[cfg(feature = "wide")]
    bkgrnd: CCharT,

    // ========================================================================
    // Option values
    // ========================================================================
    /// No timeout on function-key entry.
    notimeout: bool,
    /// Consider all data invalid (clear screen on refresh).
    clear: bool,
    /// OK to not reset cursor on exit.
    leaveok: bool,
    /// OK to scroll this window.
    scroll: bool,
    /// OK to use insert/delete line.
    idlok: bool,
    /// OK to use insert/delete char.
    idcok: bool,
    /// Immediate mode (refresh after each output).
    immed: bool,
    /// Sync mode (sync with parent on change).
    sync: bool,
    /// Process function keys into KEY_ symbols.
    use_keypad: bool,
    /// Input delay: 0=nodelay, <0=blocking, >0=delay ms.
    delay: i32,

    // ========================================================================
    // Line data
    // ========================================================================
    /// The actual line data.
    lines: Vec<LineData>,

    // ========================================================================
    // Scrolling region
    // ========================================================================
    /// Top line of scrolling region.
    regtop: NcursesSize,
    /// Bottom line of scrolling region.
    regbottom: NcursesSize,

    // ========================================================================
    // Sub-window data
    // ========================================================================
    /// X coordinate of this window in parent.
    parx: i32,
    /// Y coordinate of this window in parent.
    pary: i32,
    // Note: parent pointer not stored directly due to ownership rules

    // ========================================================================
    // Pad data
    // ========================================================================
    /// Pad-specific data for pad refresh operations.
    pad: PadData,

    // ========================================================================
    // Other
    // ========================================================================
    /// Y offset (real begy = begy + yoffset, reserved for future use).
    #[allow(dead_code)]
    yoffset: NcursesSize,

    /// Extended color pair (when ext-colors feature is enabled).
    #[cfg(feature = "ext-colors")]
    color: i32,
}

/// Builder for creating windows with a fluent API.
///
/// This provides a more ergonomic way to create windows compared to
/// passing multiple positional parameters.
///
/// # Example
///
/// ```rust,ignore
/// use ncurses::window::WindowBuilder;
///
/// let win = WindowBuilder::new()
///     .size(24, 80)
///     .position(0, 0)
///     .scrollok(true)
///     .keypad(true)
///     .build()?;
/// ```
#[derive(Clone, Debug, Default)]
pub struct WindowBuilder {
    rows: i32,
    cols: i32,
    y: i32,
    x: i32,
    scrollok: bool,
    keypad: bool,
    leaveok: bool,
    nodelay: bool,
    notimeout: bool,
    idlok: bool,
    idcok: bool,
    immedok: bool,
}

impl WindowBuilder {
    /// Create a new window builder with default values.
    #[must_use]
    pub fn new() -> Self {
        Self {
            rows: 0,
            cols: 0,
            y: 0,
            x: 0,
            scrollok: false,
            keypad: false,
            leaveok: false,
            nodelay: false,
            notimeout: false,
            idlok: false,
            idcok: true, // default is true
            immedok: false,
        }
    }

    /// Set the window size (rows and columns).
    ///
    /// If either dimension is 0, defaults to full screen size.
    #[must_use]
    pub const fn size(mut self, rows: i32, cols: i32) -> Self {
        self.rows = rows;
        self.cols = cols;
        self
    }

    /// Set the window position (y, x).
    #[must_use]
    pub const fn position(mut self, y: i32, x: i32) -> Self {
        self.y = y;
        self.x = x;
        self
    }

    /// Enable or disable scrolling.
    #[must_use]
    pub const fn scrollok(mut self, enabled: bool) -> Self {
        self.scrollok = enabled;
        self
    }

    /// Enable or disable keypad mode.
    #[must_use]
    pub const fn keypad(mut self, enabled: bool) -> Self {
        self.keypad = enabled;
        self
    }

    /// Enable or disable leaveok mode.
    #[must_use]
    pub const fn leaveok(mut self, enabled: bool) -> Self {
        self.leaveok = enabled;
        self
    }

    /// Enable or disable nodelay mode.
    #[must_use]
    pub const fn nodelay(mut self, enabled: bool) -> Self {
        self.nodelay = enabled;
        self
    }

    /// Enable or disable notimeout mode.
    #[must_use]
    pub const fn notimeout(mut self, enabled: bool) -> Self {
        self.notimeout = enabled;
        self
    }

    /// Enable or disable hardware insert/delete line.
    #[must_use]
    pub const fn idlok(mut self, enabled: bool) -> Self {
        self.idlok = enabled;
        self
    }

    /// Enable or disable hardware insert/delete character.
    #[must_use]
    pub const fn idcok(mut self, enabled: bool) -> Self {
        self.idcok = enabled;
        self
    }

    /// Enable or disable immediate refresh mode.
    #[must_use]
    pub const fn immedok(mut self, enabled: bool) -> Self {
        self.immedok = enabled;
        self
    }

    /// Build the window with the configured options.
    pub fn build(self) -> Result<Window> {
        let mut win = Window::new(self.rows, self.cols, self.y, self.x)?;
        win.scrollok(self.scrollok);
        win.keypad(self.keypad);
        win.leaveok(self.leaveok);
        win.nodelay(self.nodelay);
        win.notimeout = self.notimeout;
        win.idlok(self.idlok);
        win.idcok(self.idcok);
        win.immedok(self.immedok);
        Ok(win)
    }
}

impl Window {
    /// Create a window builder for ergonomic window creation.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let win = Window::builder()
    ///     .size(24, 80)
    ///     .scrollok(true)
    ///     .build()?;
    /// ```
    #[must_use]
    pub fn builder() -> WindowBuilder {
        WindowBuilder::new()
    }

    /// Create a new window.
    ///
    /// # Arguments
    ///
    /// * `nlines` - Number of lines (height). If 0, uses screen height - begy.
    /// * `ncols` - Number of columns (width). If 0, uses screen width - begx.
    /// * `begy` - Y coordinate of upper-left corner on screen.
    /// * `begx` - X coordinate of upper-left corner on screen.
    pub fn new(nlines: i32, ncols: i32, begy: i32, begx: i32) -> Result<Self> {
        if nlines < 0 || ncols < 0 || begy < 0 || begx < 0 {
            return Err(Error::InvalidArgument("negative window dimensions".into()));
        }

        let height = if nlines == 0 { 24 } else { nlines } as usize;
        let width = if ncols == 0 { 80 } else { ncols } as usize;

        let mut lines = Vec::with_capacity(height);
        for _ in 0..height {
            lines.push(LineData::new(width));
        }

        Ok(Self {
            cury: 0,
            curx: 0,
            maxy: (height - 1) as NcursesSize,
            maxx: (width - 1) as NcursesSize,
            begy: begy as NcursesSize,
            begx: begx as NcursesSize,
            flags: WindowFlags::empty(),
            attrs: A_NORMAL,
            #[cfg(not(feature = "wide"))]
            bkgd: b' ' as ChType,
            #[cfg(feature = "wide")]
            bkgrnd: CCharT::from_char(' '),
            notimeout: false,
            clear: false,
            leaveok: false,
            scroll: false,
            idlok: false,
            idcok: true,
            immed: false,
            sync: false,
            use_keypad: false,
            delay: -1,
            lines,
            regtop: 0,
            regbottom: (height - 1) as NcursesSize,
            parx: 0,
            pary: 0,
            pad: PadData::default(),
            yoffset: 0,
            #[cfg(feature = "ext-colors")]
            color: 0,
        })
    }

    /// Create a new pad.
    ///
    /// Pads are windows that are not constrained to the screen size.
    pub fn new_pad(nlines: i32, ncols: i32) -> Result<Self> {
        let mut win = Self::new(nlines, ncols, 0, 0)?;
        win.flags |= WindowFlags::ISPAD;
        Ok(win)
    }

    /// Move a window to a new position on the screen.
    ///
    /// This changes the window's origin (begy, begx) to the new position.
    /// The contents of the window are preserved.
    ///
    /// # Arguments
    ///
    /// * `y` - New Y coordinate of upper-left corner.
    /// * `x` - New X coordinate of upper-left corner.
    pub fn mvwin(&mut self, y: i32, x: i32) -> Result<()> {
        if y < 0 || x < 0 {
            return Err(Error::InvalidArgument("negative window position".into()));
        }

        // Can't move a pad - pads don't have screen positions
        if self.flags.contains(WindowFlags::ISPAD) {
            return Err(Error::InvalidArgument("cannot move a pad".into()));
        }

        // Can't move a subwindow - subwindows share memory with parent
        if self.flags.contains(WindowFlags::SUBWIN) {
            return Err(Error::InvalidArgument("cannot move a subwindow".into()));
        }

        self.begy = y as NcursesSize;
        self.begx = x as NcursesSize;
        self.touchwin();

        Ok(())
    }

    /// Resize a window.
    ///
    /// This changes the window's dimensions. Content is preserved where possible.
    /// If the window shrinks, content outside the new bounds is lost.
    /// If the window grows, new areas are filled with the background character.
    ///
    /// # Arguments
    ///
    /// * `lines` - New number of lines (height).
    /// * `cols` - New number of columns (width).
    pub fn resize(&mut self, lines: i32, cols: i32) -> Result<()> {
        if lines <= 0 || cols <= 0 {
            return Err(Error::InvalidArgument(
                "window dimensions must be positive".into(),
            ));
        }

        let new_height = lines as usize;
        let new_width = cols as usize;

        #[cfg(not(feature = "wide"))]
        let fill = self.bkgd;
        #[cfg(feature = "wide")]
        let fill = self.bkgrnd;

        // Resize existing lines or add new ones
        if new_height > self.lines.len() {
            // Add new lines
            for _ in self.lines.len()..new_height {
                self.lines.push(LineData::new(new_width));
            }
        } else if new_height < self.lines.len() {
            // Truncate lines
            self.lines.truncate(new_height);
        }

        // Resize each line's width
        for line in &mut self.lines {
            line.resize(new_width, fill);
        }

        // A scroll region covering the whole window keeps covering it.
        let region_was_whole_window = self.regtop == 0 && self.regbottom == self.maxy;

        // Update dimensions
        self.maxy = (new_height - 1) as NcursesSize;
        self.maxx = (new_width - 1) as NcursesSize;

        // Adjust cursor if outside new bounds
        if self.cury > self.maxy {
            self.cury = self.maxy;
        }
        if self.curx > self.maxx {
            self.curx = self.maxx;
        }

        // Update scroll region: it must stay inside the window.
        if region_was_whole_window {
            self.regbottom = self.maxy;
        } else {
            self.regtop = self.regtop.min(self.maxy);
            self.regbottom = self.regbottom.min(self.maxy);
        }

        // Touch entire window to ensure refresh
        self.touchwin();

        Ok(())
    }

    /// Create a subwindow within this window.
    ///
    /// A subwindow shares the character storage of the parent window.
    /// Changes to the subwindow affect the parent and vice versa.
    ///
    /// The subwindow coordinates are relative to the screen, not the parent.
    ///
    /// # Arguments
    ///
    /// * `nlines` - Number of lines (height) of the subwindow.
    /// * `ncols` - Number of columns (width) of the subwindow.
    /// * `begy` - Y coordinate of upper-left corner (screen-relative).
    /// * `begx` - X coordinate of upper-left corner (screen-relative).
    ///
    /// # Note
    ///
    /// Due to Rust's ownership model, this creates a new window with its own
    /// storage. For true shared storage, use derwin and coordinate updates
    /// between parent and child.
    pub fn subwin(&self, nlines: i32, ncols: i32, begy: i32, begx: i32) -> Result<Self> {
        if nlines < 0 || ncols < 0 || begy < 0 || begx < 0 {
            return Err(Error::InvalidArgument(
                "negative subwindow dimensions".into(),
            ));
        }

        // Check if subwindow fits within parent (screen coordinates)
        let parent_begy = self.begy as i32;
        let parent_begx = self.begx as i32;
        let parent_maxy = parent_begy + self.getmaxy();
        let parent_maxx = parent_begx + self.getmaxx();

        let sub_maxy = if nlines == 0 {
            parent_maxy
        } else {
            begy + nlines
        };
        let sub_maxx = if ncols == 0 {
            parent_maxx
        } else {
            begx + ncols
        };

        if begy < parent_begy
            || begx < parent_begx
            || sub_maxy > parent_maxy
            || sub_maxx > parent_maxx
        {
            return Err(Error::InvalidArgument(
                "subwindow extends beyond parent boundaries".into(),
            ));
        }

        let height = if nlines == 0 {
            parent_maxy - begy
        } else {
            nlines
        };
        let width = if ncols == 0 {
            parent_maxx - begx
        } else {
            ncols
        };

        let mut win = Self::new(height, width, begy, begx)?;
        win.flags |= WindowFlags::SUBWIN;

        // Store parent offset for coordinate translation
        win.pary = begy - parent_begy;
        win.parx = begx - parent_begx;

        Ok(win)
    }

    /// Create a derived window (subwindow with parent-relative coordinates).
    ///
    /// Like subwin, but the coordinates are relative to the parent window
    /// rather than the screen.
    ///
    /// # Arguments
    ///
    /// * `nlines` - Number of lines (height) of the derived window.
    /// * `ncols` - Number of columns (width) of the derived window.
    /// * `begy` - Y coordinate relative to parent window.
    /// * `begx` - X coordinate relative to parent window.
    ///
    /// # Note
    ///
    /// Due to Rust's ownership model, this creates a new window with its own
    /// storage. For true shared storage, coordinate updates between parent
    /// and child manually.
    pub fn derwin(&self, nlines: i32, ncols: i32, begy: i32, begx: i32) -> Result<Self> {
        if nlines < 0 || ncols < 0 || begy < 0 || begx < 0 {
            return Err(Error::InvalidArgument("negative derwin dimensions".into()));
        }

        // Convert parent-relative to screen coordinates
        let screen_begy = self.begy as i32 + begy;
        let screen_begx = self.begx as i32 + begx;

        // Check bounds
        let height = if nlines == 0 {
            self.getmaxy() - begy
        } else {
            nlines
        };
        let width = if ncols == 0 {
            self.getmaxx() - begx
        } else {
            ncols
        };

        if begy + height > self.getmaxy() || begx + width > self.getmaxx() {
            return Err(Error::InvalidArgument(
                "derived window extends beyond parent boundaries".into(),
            ));
        }

        let mut win = Self::new(height, width, screen_begy, screen_begx)?;
        win.flags |= WindowFlags::SUBWIN;

        // Store parent-relative offset
        win.pary = begy;
        win.parx = begx;

        Ok(win)
    }

    /// Move a derived window relative to its parent.
    ///
    /// This function moves a derived window (created by `derwin`) to a new position
    /// relative to its parent window. The window's contents are preserved.
    ///
    /// # Arguments
    ///
    /// * `y` - New Y position relative to parent
    /// * `x` - New X position relative to parent
    pub fn mvderwin(&mut self, y: i32, x: i32) -> Result<()> {
        if !self.flags.contains(WindowFlags::SUBWIN) {
            return Err(Error::InvalidArgument(
                "mvderwin can only be used on derived windows".into(),
            ));
        }

        if y < 0 || x < 0 {
            return Err(Error::InvalidArgument(
                "negative derived window position".into(),
            ));
        }

        // Update parent-relative position
        self.pary = y;
        self.parx = x;

        // Touch the window to ensure it gets refreshed
        self.touchwin();

        Ok(())
    }

    /// Duplicate this window.
    ///
    /// Creates an exact copy of this window with its own storage.
    pub fn dupwin(&self) -> Result<Self> {
        let mut win = Self::new(
            self.getmaxy(),
            self.getmaxx(),
            self.begy as i32,
            self.begx as i32,
        )?;

        // Copy all state
        win.cury = self.cury;
        win.curx = self.curx;
        win.flags = self.flags;
        win.flags.remove(WindowFlags::SUBWIN); // Duplicated window is not a subwindow
        win.attrs = self.attrs;
        #[cfg(not(feature = "wide"))]
        {
            win.bkgd = self.bkgd;
        }
        #[cfg(feature = "wide")]
        {
            win.bkgrnd = self.bkgrnd;
        }
        win.notimeout = self.notimeout;
        win.clear = self.clear;
        win.leaveok = self.leaveok;
        win.scroll = self.scroll;
        win.idlok = self.idlok;
        win.idcok = self.idcok;
        win.immed = self.immed;
        win.sync = self.sync;
        win.use_keypad = self.use_keypad;
        win.delay = self.delay;
        win.regtop = self.regtop;
        win.regbottom = self.regbottom;
        win.parx = 0;
        win.pary = 0;
        win.yoffset = self.yoffset;
        #[cfg(feature = "ext-colors")]
        {
            win.color = self.color;
        }

        // Copy line data
        for (i, line) in self.lines.iter().enumerate() {
            if i < win.lines.len() {
                win.lines[i] = line.clone();
            }
        }

        Ok(win)
    }

    // ========================================================================
    // Dimension getters
    // ========================================================================

    /// Get the number of lines (height) in the window.
    #[inline]
    #[must_use]
    pub fn getmaxy(&self) -> i32 {
        (self.maxy + 1) as i32
    }

    /// Get the number of columns (width) in the window.
    #[inline]
    #[must_use]
    pub fn getmaxx(&self) -> i32 {
        (self.maxx + 1) as i32
    }

    /// Get the Y coordinate of the upper-left corner.
    #[inline]
    #[must_use]
    pub fn getbegy(&self) -> i32 {
        self.begy as i32
    }

    /// Get the X coordinate of the upper-left corner.
    #[inline]
    #[must_use]
    pub fn getbegx(&self) -> i32 {
        self.begx as i32
    }

    /// Get the current cursor Y position.
    #[inline]
    #[must_use]
    pub fn getcury(&self) -> i32 {
        self.cury as i32
    }

    /// Get the current cursor X position.
    #[inline]
    #[must_use]
    pub fn getcurx(&self) -> i32 {
        self.curx as i32
    }

    /// Get the parent X coordinate (for subwindows).
    #[inline]
    #[must_use]
    pub fn getparx(&self) -> i32 {
        self.parx
    }

    /// Get the parent Y coordinate (for subwindows).
    #[inline]
    #[must_use]
    pub fn getpary(&self) -> i32 {
        self.pary
    }

    // ========================================================================
    // Cursor movement
    // ========================================================================

    /// Move the cursor to a new position.
    ///
    /// # Arguments
    ///
    /// * `y` - New Y coordinate (row).
    /// * `x` - New X coordinate (column).
    pub fn mv(&mut self, y: i32, x: i32) -> Result<()> {
        if y < 0 || y > self.maxy as i32 || x < 0 || x > self.maxx as i32 {
            return Err(Error::OutOfBounds {
                y,
                x,
                max_y: self.maxy as i32,
                max_x: self.maxx as i32,
            });
        }
        self.cury = y as NcursesSize;
        self.curx = x as NcursesSize;
        self.flags |= WindowFlags::HASMOVED;
        Ok(())
    }

    // ========================================================================
    // Character output
    // ========================================================================

    /// Add a character at the current position.
    #[cfg(not(feature = "wide"))]
    pub fn addch(&mut self, ch: ChType) -> Result<()> {
        let render_ch = ch | self.attrs;
        self.add_ch_internal(render_ch)
    }

    /// Add a character at the current cursor position (wide character version).
    #[cfg(feature = "wide")]
    pub fn addch(&mut self, ch: ChType) -> Result<()> {
        self.add_wch_internal(CCharT::from_chtype(ch | self.attrs))
    }

    /// Add a wide character at the current position.
    #[cfg(feature = "wide")]
    pub fn add_wch(&mut self, wch: &CCharT) -> Result<()> {
        let mut cchar = *wch;
        cchar.attr |= self.attrs;
        self.add_wch_internal(cchar)
    }

    /// Move to position and add a character.
    pub fn mvaddch(&mut self, y: i32, x: i32, ch: ChType) -> Result<()> {
        self.mv(y, x)?;
        self.addch(ch)
    }

    /// Internal character addition for non-wide mode.
    #[cfg(not(feature = "wide"))]
    fn add_ch_internal(&mut self, ch: ChType) -> Result<()> {
        let x = self.curx as usize;
        let y = self.cury as usize;

        // Handle special characters
        let c = (ch & A_CHARTEXT) as u8;
        match c {
            b'\n' => {
                // Newline: clear to end of line and move to next line
                self.clrtoeol()?;
                if self.next_line()? {
                    self.curx = 0;
                }
                return Ok(());
            }
            b'\r' => {
                self.curx = 0;
                return Ok(());
            }
            b'\t' => {
                // Tab: move to next tab stop (every 8 columns)
                let next_tab = ((self.curx / 8) + 1) * 8;
                let spaces = (next_tab - self.curx) as usize;
                for _ in 0..spaces {
                    self.add_ch_internal(b' ' as ChType | (ch & !A_CHARTEXT))?;
                }
                return Ok(());
            }
            b'\x08' => {
                // Backspace
                if self.curx > 0 {
                    self.curx -= 1;
                }
                return Ok(());
            }
            _ => {}
        }

        // Check bounds
        if y > self.maxy as usize {
            return Ok(());
        }

        // Write the character
        if x <= self.maxx as usize {
            self.lines[y].set(x, ch);
        }

        // Advance cursor
        self.advance_cursor()?;

        Ok(())
    }

    /// Internal wide character addition.
    #[cfg(feature = "wide")]
    fn add_wch_internal(&mut self, ch: CCharT) -> Result<()> {
        let x = self.curx as usize;
        let y = self.cury as usize;

        // Handle special characters
        let c = ch.spacing_char();
        match c {
            '\n' => {
                self.clrtoeol()?;
                if self.next_line()? {
                    self.curx = 0;
                }
                return Ok(());
            }
            '\r' => {
                self.curx = 0;
                return Ok(());
            }
            '\t' => {
                let next_tab = ((self.curx / 8) + 1) * 8;
                let spaces = (next_tab - self.curx) as usize;
                for _ in 0..spaces {
                    let space = CCharT::from_char_attr(' ', ch.attr);
                    self.add_wch_internal(space)?;
                }
                return Ok(());
            }
            '\x08' => {
                if self.curx > 0 {
                    self.curx -= 1;
                }
                return Ok(());
            }
            _ => {}
        }

        if y > self.maxy as usize {
            return Ok(());
        }

        // Handle wide characters (2-column)
        let width = ch.width();
        if x + width > (self.maxx + 1) as usize {
            // Character doesn't fit, wrap or don't draw
            if self.scroll && y == self.maxy as usize {
                // At bottom, need to scroll
            }
            // For now, just don't draw if it doesn't fit
            self.advance_cursor()?;
            return Ok(());
        }

        // Write the character
        self.lines[y].set(x, ch);

        // For wide characters, fill the second cell with a placeholder
        if width > 1 && x < self.maxx as usize {
            // Use a special marker for the second cell
            let placeholder = CCharT::new();
            self.lines[y].set(x + 1, placeholder);
        }

        // Advance cursor by character width
        for _ in 0..width {
            self.advance_cursor()?;
        }

        Ok(())
    }

    /// Advance the cursor after character output.
    fn advance_cursor(&mut self) -> Result<()> {
        self.curx += 1;
        if self.curx > self.maxx {
            self.flags |= WindowFlags::WRAPPED;
            if self.next_line()? {
                self.curx = 0;
            } else {
                // Stay at the right edge
                self.curx = self.maxx;
            }
        }
        Ok(())
    }

    /// Move the cursor down a line, as a newline or a wrap at the right edge
    /// does, and say whether it moved.
    ///
    /// On the last line of the scroll region the region scrolls up instead
    /// and the cursor stays on that line, if scrolling is enabled. Without
    /// scrolling the cursor does not move from there, nor from the last line
    /// of the window.
    fn next_line(&mut self) -> Result<bool> {
        if self.cury == self.regbottom {
            if !self.scroll {
                return Ok(false);
            }
            self.scroll_up(1)?;
        } else if self.cury < self.maxy {
            self.cury += 1;
        } else {
            return Ok(false);
        }
        Ok(true)
    }

    /// Add a string at the current position.
    pub fn addstr(&mut self, s: &str) -> Result<()> {
        self.addnstr(s, -1)
    }

    /// Move to position and add a string.
    pub fn mvaddstr(&mut self, y: i32, x: i32, s: &str) -> Result<()> {
        self.mv(y, x)?;
        self.addstr(s)
    }

    /// Move to position and add a string with a maximum length.
    pub fn mvaddnstr(&mut self, y: i32, x: i32, s: &str, n: i32) -> Result<()> {
        self.mv(y, x)?;
        self.addnstr(s, n)
    }

    /// Add a string with a maximum length.
    pub fn addnstr(&mut self, s: &str, n: i32) -> Result<()> {
        let max_chars = if n < 0 { usize::MAX } else { n as usize };
        for (i, c) in s.chars().enumerate() {
            if i >= max_chars {
                break;
            }
            #[cfg(not(feature = "wide"))]
            {
                if c.is_ascii() {
                    self.addch(c as ChType)?;
                } else {
                    self.addch(b'?' as ChType)?;
                }
            }
            #[cfg(feature = "wide")]
            {
                let cchar = CCharT::from_char_attr(c, self.attrs);
                self.add_wch_internal(cchar)?;
            }
        }
        Ok(())
    }

    /// Add a chtype string at the current position.
    pub fn addchstr(&mut self, chstr: &[ChType]) -> Result<()> {
        self.addchnstr(chstr, -1)
    }

    /// Add a chtype string with a maximum length.
    pub fn addchnstr(&mut self, chstr: &[ChType], n: i32) -> Result<()> {
        let max_chars = if n < 0 { chstr.len() } else { n as usize };
        let y = self.cury as usize;
        let first_x = self.curx as usize;

        for (x, &ch) in (first_x..).zip(chstr.iter().take(max_chars)) {
            if x > self.maxx as usize {
                break;
            }
            #[cfg(not(feature = "wide"))]
            self.lines[y].set(x, ch | self.attrs);
            #[cfg(feature = "wide")]
            self.lines[y].set(x, CCharT::from_chtype(ch | self.attrs));
        }

        Ok(())
    }

    // ========================================================================
    // Wide character string output
    // ========================================================================

    /// Add a wide string at the current position.
    ///
    /// This is the Rust equivalent of `addwstr()` / `waddwstr()`.
    /// The string is converted to wide characters and output to the window.
    #[cfg(feature = "wide")]
    pub fn addwstr(&mut self, s: &str) -> Result<()> {
        self.addnwstr(s, -1)
    }

    /// Add a wide string with a maximum length.
    ///
    /// This is the Rust equivalent of `addnwstr()` / `waddnwstr()`.
    /// At most n characters are written. If n is negative, the entire string is written.
    #[cfg(feature = "wide")]
    pub fn addnwstr(&mut self, s: &str, n: i32) -> Result<()> {
        let max_chars = if n < 0 { usize::MAX } else { n as usize };
        for (i, c) in s.chars().enumerate() {
            if i >= max_chars {
                break;
            }
            let cchar = CCharT::from_char_attr(c, self.attrs);
            self.add_wch_internal(cchar)?;
        }
        Ok(())
    }

    /// Move cursor and add a wide string.
    #[cfg(feature = "wide")]
    pub fn mvaddwstr(&mut self, y: i32, x: i32, s: &str) -> Result<()> {
        self.mv(y, x)?;
        self.addwstr(s)
    }

    /// Move cursor and add a wide string with a maximum length.
    #[cfg(feature = "wide")]
    pub fn mvaddnwstr(&mut self, y: i32, x: i32, s: &str, n: i32) -> Result<()> {
        self.mv(y, x)?;
        self.addnwstr(s, n)
    }

    // ========================================================================
    // Wide character array output (add_wchstr family)
    // ========================================================================

    /// Add a wide character string (array of cchar_t) at the current position.
    ///
    /// This is the ncurses `add_wchstr()` function.
    /// Characters are written to the current line, stopping at the right margin.
    /// Unlike `addwstr`, this function preserves the attributes in each cchar_t.
    #[cfg(feature = "wide")]
    pub fn add_wchstr(&mut self, wchstr: &[CCharT]) -> Result<()> {
        self.add_wchnstr(wchstr, -1)
    }

    /// Add at most n wide characters from a string at the current position.
    ///
    /// This is the ncurses `add_wchnstr()` function.
    /// At most n characters are written. If n is negative, the entire array is written.
    /// Writing stops at the right margin without wrapping.
    #[cfg(feature = "wide")]
    pub fn add_wchnstr(&mut self, wchstr: &[CCharT], n: i32) -> Result<()> {
        let y = self.cury as usize;
        let first_x = self.curx as usize;
        let max_chars = if n < 0 { wchstr.len() } else { n as usize };

        for (x, wch) in (first_x..).zip(wchstr.iter().take(max_chars)) {
            if x > self.maxx as usize {
                break;
            }
            self.lines[y].set(x, *wch);
        }

        Ok(())
    }

    /// Move cursor and add a wide character string.
    ///
    /// This is the ncurses `mvadd_wchstr()` function.
    #[cfg(feature = "wide")]
    pub fn mvadd_wchstr(&mut self, y: i32, x: i32, wchstr: &[CCharT]) -> Result<()> {
        self.mv(y, x)?;
        self.add_wchstr(wchstr)
    }

    /// Move cursor and add at most n wide characters from a string.
    ///
    /// This is the ncurses `mvadd_wchnstr()` function.
    #[cfg(feature = "wide")]
    pub fn mvadd_wchnstr(&mut self, y: i32, x: i32, wchstr: &[CCharT], n: i32) -> Result<()> {
        self.mv(y, x)?;
        self.add_wchnstr(wchstr, n)
    }

    /// Move cursor and add a wide character.
    ///
    /// This is the ncurses `mvadd_wch()` function.
    #[cfg(feature = "wide")]
    pub fn mvadd_wch(&mut self, y: i32, x: i32, wch: &CCharT) -> Result<()> {
        self.mv(y, x)?;
        self.add_wch(wch)
    }

    // ========================================================================
    // Character input (reading from window)
    // ========================================================================

    /// Get the character at the current position.
    #[cfg(not(feature = "wide"))]
    #[must_use]
    pub fn inch(&self) -> ChType {
        let y = self.cury as usize;
        let x = self.curx as usize;
        if y <= self.maxy as usize && x <= self.maxx as usize {
            self.lines[y].get(x)
        } else {
            0
        }
    }

    /// Get the character at the current cursor position (wide character version).
    #[cfg(feature = "wide")]
    #[must_use]
    pub fn inch(&self) -> ChType {
        let y = self.cury as usize;
        let x = self.curx as usize;
        if y <= self.maxy as usize && x <= self.maxx as usize {
            self.lines[y].get(x).to_chtype()
        } else {
            0
        }
    }

    /// Get a string of characters from the current position.
    #[must_use]
    pub fn instr(&self, n: i32) -> String {
        let mut result = String::new();
        let y = self.cury as usize;
        let max_x = if n < 0 {
            self.maxx as usize + 1
        } else {
            (self.curx as usize + n as usize).min(self.maxx as usize + 1)
        };

        for x in (self.curx as usize)..max_x {
            #[cfg(not(feature = "wide"))]
            {
                let ch = self.lines[y].get(x);
                let c = (ch & A_CHARTEXT) as u8;
                if c == 0 {
                    break;
                }
                result.push(c as char);
            }
            #[cfg(feature = "wide")]
            {
                let cchar = self.lines[y].get(x);
                let c = cchar.spacing_char();
                if c == '\0' {
                    break;
                }
                result.push(c);
            }
        }

        result
    }

    /// Move to position and get the character at that position.
    pub fn mvinch(&mut self, y: i32, x: i32) -> Result<ChType> {
        self.mv(y, x)?;
        Ok(self.inch())
    }

    /// Get a string of characters with attributes from the current position.
    ///
    /// This reads characters (with attributes) into a slice of ChType values.
    /// Returns the number of characters read.
    ///
    /// If `n` is negative, reads to the end of the line.
    pub fn inchnstr(&self, chstr: &mut [ChType], n: i32) -> i32 {
        let y = self.cury as usize;
        let max_chars = if n < 0 {
            chstr
                .len()
                .min((self.maxx as usize + 1).saturating_sub(self.curx as usize))
        } else {
            chstr
                .len()
                .min(n as usize)
                .min((self.maxx as usize + 1).saturating_sub(self.curx as usize))
        };

        let mut count = 0;
        for (i, ch) in chstr.iter_mut().take(max_chars).enumerate() {
            let x = self.curx as usize + i;
            if x > self.maxx as usize {
                break;
            }
            #[cfg(not(feature = "wide"))]
            {
                *ch = self.lines[y].get(x);
            }
            #[cfg(feature = "wide")]
            {
                *ch = self.lines[y].get(x).to_chtype();
            }
            count += 1;
        }

        count
    }

    /// Move to position and get a string of characters with attributes.
    pub fn mvinchnstr(&mut self, y: i32, x: i32, chstr: &mut [ChType], n: i32) -> Result<i32> {
        self.mv(y, x)?;
        Ok(self.inchnstr(chstr, n))
    }

    /// Get a string of characters from the current position with a limit.
    ///
    /// This is equivalent to `instr(n)` but with an explicit limit parameter
    /// matching the ncurses `innstr()` API.
    #[must_use]
    pub fn innstr(&self, n: i32) -> String {
        self.instr(n)
    }

    /// Move to position and get a string of characters with a limit.
    pub fn mvinnstr(&mut self, y: i32, x: i32, n: i32) -> Result<String> {
        self.mv(y, x)?;
        Ok(self.instr(n))
    }

    // ========================================================================
    // Clearing
    // ========================================================================

    /// Clear the entire window.
    pub fn clear(&mut self) -> Result<()> {
        self.erase()?;
        self.clear = true;
        Ok(())
    }

    /// Erase the entire window (fill with background).
    pub fn erase(&mut self) -> Result<()> {
        #[cfg(not(feature = "wide"))]
        let fill = self.bkgd;
        #[cfg(feature = "wide")]
        let fill = self.bkgrnd;

        for line in &mut self.lines {
            line.fill(fill);
        }
        self.cury = 0;
        self.curx = 0;
        Ok(())
    }

    /// Clear to end of line.
    pub fn clrtoeol(&mut self) -> Result<()> {
        let y = self.cury as usize;
        let x = self.curx as usize;

        #[cfg(not(feature = "wide"))]
        let fill = self.bkgd;
        #[cfg(feature = "wide")]
        let fill = self.bkgrnd;

        if y <= self.maxy as usize {
            self.lines[y].fill_range(x, (self.maxx + 1) as usize, fill);
        }
        Ok(())
    }

    /// Clear to end of window.
    pub fn clrtobot(&mut self) -> Result<()> {
        self.clrtoeol()?;

        #[cfg(not(feature = "wide"))]
        let fill = self.bkgd;
        #[cfg(feature = "wide")]
        let fill = self.bkgrnd;

        for y in (self.cury as usize + 1)..=(self.maxy as usize) {
            self.lines[y].fill(fill);
        }
        Ok(())
    }

    // ========================================================================
    // Scrolling
    // ========================================================================

    /// Scroll the window up by n lines.
    pub fn scroll_up(&mut self, n: i32) -> Result<()> {
        if n > 0 {
            self.shift_lines_up(n, self.regtop as usize, self.regbottom as usize);
        }
        Ok(())
    }

    /// Move the lines `top..=bottom` up by `n` (which must be positive),
    /// dropping the first `n` of them and clearing the last `n`.
    fn shift_lines_up(&mut self, n: i32, top: usize, bottom: usize) {
        let n = (n as usize).min(bottom + 1 - top);

        #[cfg(not(feature = "wide"))]
        let fill = self.bkgd;
        #[cfg(feature = "wide")]
        let fill = self.bkgrnd;

        // The first line that is cleared rather than shifted into. When the
        // whole region scrolls away this is `top`, and nothing is shifted.
        let first_cleared = bottom + 1 - n;

        // Shift lines up - clone source lines first to avoid borrow issues
        for y in top..first_cleared {
            let src_line = self.lines[y + n].clone();
            self.lines[y].copy_from(&src_line);
        }

        // Clear the bottom lines
        for line in &mut self.lines[first_cleared..=bottom] {
            line.fill(fill);
        }
    }

    /// Scroll the window down by n lines.
    pub fn scroll_down(&mut self, n: i32) -> Result<()> {
        if n > 0 {
            self.shift_lines_down(n, self.regtop as usize, self.regbottom as usize);
        }
        Ok(())
    }

    /// Move the lines `top..=bottom` down by `n` (which must be positive),
    /// dropping the last `n` of them and clearing the first `n`.
    fn shift_lines_down(&mut self, n: i32, top: usize, bottom: usize) {
        let n = (n as usize).min(bottom + 1 - top);

        #[cfg(not(feature = "wide"))]
        let fill = self.bkgd;
        #[cfg(feature = "wide")]
        let fill = self.bkgrnd;

        // Shift lines down - clone source lines first to avoid borrow issues
        for y in ((top + n)..=bottom).rev() {
            let src_line = self.lines[y - n].clone();
            self.lines[y].copy_from(&src_line);
        }

        // Clear the top lines
        for line in &mut self.lines[top..(top + n)] {
            line.fill(fill);
        }
    }

    /// Scroll the scrolling region (wscrl).
    pub fn scrl(&mut self, n: i32) -> Result<()> {
        if !self.scroll {
            return Err(Error::WindowError("scrolling not enabled".into()));
        }
        if n > 0 {
            self.scroll_up(n)
        } else if n < 0 {
            self.scroll_down(-n)
        } else {
            Ok(())
        }
    }

    // ========================================================================
    // Attributes
    // ========================================================================

    /// Turn on attributes.
    pub fn attron(&mut self, attr: AttrT) -> Result<()> {
        self.attrs |= attr;
        Ok(())
    }

    /// Turn off attributes.
    pub fn attroff(&mut self, attr: AttrT) -> Result<()> {
        self.attrs &= !attr;
        Ok(())
    }

    /// Set attributes.
    pub fn attrset(&mut self, attr: AttrT) -> Result<()> {
        self.attrs = attr;
        Ok(())
    }

    /// Get current attributes.
    #[must_use]
    pub fn getattrs(&self) -> AttrT {
        self.attrs
    }

    /// Turn on standout mode (typically reverse video).
    pub fn standout(&mut self) -> Result<()> {
        self.attron(crate::attr::A_STANDOUT)
    }

    /// Turn off standout mode.
    pub fn standend(&mut self) -> Result<()> {
        self.attrset(A_NORMAL)
    }

    /// Set color pair.
    pub fn color_set(&mut self, pair: i16) -> Result<()> {
        self.attrs = (self.attrs & !attr::A_COLOR) | color_pair(pair);
        #[cfg(feature = "ext-colors")]
        {
            self.color = pair as i32;
        }
        Ok(())
    }

    // ========================================================================
    // Background
    // ========================================================================

    /// Set the background character.
    #[cfg(not(feature = "wide"))]
    pub fn bkgdset(&mut self, ch: ChType) {
        self.bkgd = ch;
    }

    /// Set the background character and attribute (wide character version).
    #[cfg(feature = "wide")]
    pub fn bkgdset(&mut self, ch: ChType) {
        self.bkgrnd = CCharT::from_chtype(ch);
    }

    /// Get the background character.
    #[cfg(not(feature = "wide"))]
    #[must_use]
    pub fn getbkgd(&self) -> ChType {
        self.bkgd
    }

    /// Get the background character (wide character version).
    #[cfg(feature = "wide")]
    #[must_use]
    pub fn getbkgd(&self) -> ChType {
        self.bkgrnd.to_chtype()
    }

    /// Set the background and apply to entire window.
    #[cfg(not(feature = "wide"))]
    pub fn bkgd(&mut self, ch: ChType) -> Result<()> {
        let old_bkgd = self.bkgd;
        self.bkgd = ch;

        let old_char = (old_bkgd & A_CHARTEXT) as u8;
        let old_attr = old_bkgd & !A_CHARTEXT;
        let new_char = (ch & A_CHARTEXT) as u8;
        let new_attr = ch & !A_CHARTEXT;

        // Update all cells
        for line in &mut self.lines {
            for x in 0..line.width() {
                let cell = line.get(x);
                let cell_char = (cell & A_CHARTEXT) as u8;
                let cell_attr = cell & !A_CHARTEXT;

                // Replace old background char with new
                let updated_char = if cell_char == old_char || cell_char == b' ' {
                    new_char
                } else {
                    cell_char
                };

                // Update attributes
                let updated_attr = (cell_attr & !old_attr) | new_attr;
                line.set(x, updated_char as ChType | updated_attr);
            }
        }

        Ok(())
    }

    /// Set the background and apply to entire window (wide character version).
    #[cfg(feature = "wide")]
    pub fn bkgd(&mut self, ch: ChType) -> Result<()> {
        self.bkgrnd = CCharT::from_chtype(ch);
        // For simplicity, just set background without full update
        Ok(())
    }

    // ========================================================================
    // Change attributes
    // ========================================================================

    /// Change attributes of characters starting at current position.
    ///
    /// This changes the attributes of `n` characters starting at the current
    /// cursor position without moving the cursor or changing the characters.
    /// If `n` is -1, it changes attributes to the end of the line.
    ///
    /// # Arguments
    /// * `n` - Number of characters to change (-1 for rest of line)
    /// * `attr` - Attributes to apply
    /// * `color` - Color pair number (0 to use current color)
    pub fn chgat(&mut self, n: i32, attr: AttrT, color: i16) -> Result<()> {
        self.wchgat(n, attr, color)
    }

    /// Change attributes of characters starting at current position (window version).
    pub fn wchgat(&mut self, n: i32, attr: AttrT, color: i16) -> Result<()> {
        let y = self.cury as usize;
        let x = self.curx as usize;

        if y > self.maxy as usize {
            return Ok(());
        }

        // Determine end position
        let end_x = if n < 0 {
            self.maxx as usize + 1
        } else {
            (x + n as usize).min(self.maxx as usize + 1)
        };

        // Build the combined attribute with color
        let combined_attr = if color != 0 {
            attr | color_pair(color)
        } else {
            attr
        };

        // Change attributes for each character
        #[cfg(not(feature = "wide"))]
        {
            for cx in x..end_x {
                let cell = self.lines[y].get(cx);
                let ch = cell & A_CHARTEXT;
                self.lines[y].set(cx, ch | combined_attr);
            }
        }

        #[cfg(feature = "wide")]
        {
            for cx in x..end_x {
                let mut cell = self.lines[y].get(cx);
                cell.set_attrs(combined_attr);
                self.lines[y].set(cx, cell);
            }
        }

        Ok(())
    }

    /// Move cursor and change attributes.
    pub fn mvchgat(&mut self, y: i32, x: i32, n: i32, attr: AttrT, color: i16) -> Result<()> {
        self.mv(y, x)?;
        self.chgat(n, attr, color)
    }

    /// Move cursor and change attributes (window version).
    pub fn mvwchgat(&mut self, y: i32, x: i32, n: i32, attr: AttrT, color: i16) -> Result<()> {
        self.mv(y, x)?;
        self.wchgat(n, attr, color)
    }

    // ========================================================================
    // Borders and lines
    // ========================================================================

    /// Draw a box around the window.
    pub fn box_(&mut self, verch: ChType, horch: ChType) -> Result<()> {
        self.border(verch, verch, horch, horch, 0, 0, 0, 0)
    }

    /// Draw a border around the window using a BorderChars specification.
    ///
    /// This is a more ergonomic alternative to the 8-parameter `border()` method.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ncurses::types::BorderChars;
    ///
    /// // Use default line-drawing characters
    /// win.draw_border(BorderChars::default())?;
    ///
    /// // Use custom characters
    /// win.draw_border(BorderChars::simple('|' as u32, '-' as u32))?;
    /// ```
    pub fn draw_border(&mut self, chars: crate::types::BorderChars) -> Result<()> {
        self.border(
            chars.left,
            chars.right,
            chars.top,
            chars.bottom,
            chars.top_left,
            chars.top_right,
            chars.bottom_left,
            chars.bottom_right,
        )
    }

    /// Draw a border around the window.
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
        // The default line-drawing characters do not fit in a `ChType`, so
        // the border is drawn from complex characters.
        #[cfg(feature = "wide")]
        {
            use crate::wide::wacs;

            let ls = Self::chtype_or(ls, &wacs::VLINE);
            let rs = Self::chtype_or(rs, &wacs::VLINE);
            let ts = Self::chtype_or(ts, &wacs::HLINE);
            let bs = Self::chtype_or(bs, &wacs::HLINE);
            let tl = Self::chtype_or(tl, &wacs::ULCORNER);
            let tr = Self::chtype_or(tr, &wacs::URCORNER);
            let bl = Self::chtype_or(bl, &wacs::LLCORNER);
            let br = Self::chtype_or(br, &wacs::LRCORNER);

            self.wborder_set(
                Some(&ls),
                Some(&rs),
                Some(&ts),
                Some(&bs),
                Some(&tl),
                Some(&tr),
                Some(&bl),
                Some(&br),
            )
        }
        #[cfg(not(feature = "wide"))]
        {
            // Default characters if 0
            let ls = if ls == 0 { self.acs_vline() } else { ls };
            let rs = if rs == 0 { self.acs_vline() } else { rs };
            let ts = if ts == 0 { self.acs_hline() } else { ts };
            let bs = if bs == 0 { self.acs_hline() } else { bs };
            let tl = if tl == 0 { self.acs_ulcorner() } else { tl };
            let tr = if tr == 0 { self.acs_urcorner() } else { tr };
            let bl = if bl == 0 { self.acs_llcorner() } else { bl };
            let br = if br == 0 { self.acs_lrcorner() } else { br };

            let maxy = self.maxy as usize;
            let maxx = self.maxx as usize;

            // Corners
            self.lines[0].set(0, tl);
            self.lines[0].set(maxx, tr);
            self.lines[maxy].set(0, bl);
            self.lines[maxy].set(maxx, br);

            // Top and bottom edges
            for x in 1..maxx {
                self.lines[0].set(x, ts);
                self.lines[maxy].set(x, bs);
            }

            // Left and right edges
            for line in self.lines.iter_mut().take(maxy).skip(1) {
                line.set(0, ls);
                line.set(maxx, rs);
            }

            Ok(())
        }
    }

    /// The cell a `ChType` border or line argument stands for: `default`
    /// when it is 0, otherwise the character and attributes it carries.
    #[cfg(feature = "wide")]
    fn chtype_or(ch: ChType, default: &CCharT) -> CCharT {
        if ch == 0 {
            *default
        } else {
            CCharT::from_chtype(ch)
        }
    }

    /// How many cells a line of `n` draws when `room` cells are left before
    /// the window's edge: none for a zero or negative `n`.
    fn line_len(n: i32, room: i32) -> usize {
        n.min(room).max(0) as usize
    }

    /// Draw a horizontal line.
    ///
    /// The line starts at the cursor and is at most `n` cells long, cut off
    /// at the window's edge. Nothing is drawn when `n` is zero or negative.
    pub fn hline(&mut self, ch: ChType, n: i32) -> Result<()> {
        #[cfg(not(feature = "wide"))]
        let ch = if ch == 0 { self.acs_hline() } else { ch };
        let n = Self::line_len(n, (self.maxx - self.curx + 1) as i32);
        let y = self.cury as usize;
        let x = self.curx as usize;

        #[cfg(not(feature = "wide"))]
        {
            // Combine character with window attributes (like addch does)
            for i in 0..n {
                self.lines[y].set(x + i, ch | self.attrs);
            }
        }
        #[cfg(feature = "wide")]
        {
            // The character and its attributes, combined with the window's
            let mut cchar = Self::chtype_or(ch, &crate::wide::wacs::HLINE);
            cchar.attr |= self.attrs;
            for i in 0..n {
                self.lines[y].set(x + i, cchar);
            }
        }

        Ok(())
    }

    /// Draw a vertical line.
    ///
    /// The line starts at the cursor and is at most `n` cells long, cut off
    /// at the window's edge. Nothing is drawn when `n` is zero or negative.
    pub fn vline(&mut self, ch: ChType, n: i32) -> Result<()> {
        #[cfg(not(feature = "wide"))]
        let ch = if ch == 0 { self.acs_vline() } else { ch };
        let n = Self::line_len(n, (self.maxy - self.cury + 1) as i32);
        let y = self.cury as usize;
        let x = self.curx as usize;

        #[cfg(not(feature = "wide"))]
        {
            // Combine character with window attributes (like addch does)
            for i in 0..n {
                self.lines[y + i].set(x, ch | self.attrs);
            }
        }
        #[cfg(feature = "wide")]
        {
            // The character and its attributes, combined with the window's
            let mut cchar = Self::chtype_or(ch, &crate::wide::wacs::VLINE);
            cchar.attr |= self.attrs;
            for i in 0..n {
                self.lines[y + i].set(x, cchar);
            }
        }

        Ok(())
    }

    /// Draw a horizontal line using a complex character.
    #[cfg(feature = "wide")]
    pub fn hline_set(&mut self, wch: &CCharT, n: i32) -> Result<()> {
        let wch = if wch.spacing_char() == '\0' {
            CCharT::from_char('─')
        } else {
            *wch
        };
        let n = Self::line_len(n, (self.maxx - self.curx + 1) as i32);
        let y = self.cury as usize;
        let x = self.curx as usize;

        for i in 0..n {
            self.lines[y].set(x + i, wch);
        }

        Ok(())
    }

    /// Draw a vertical line using a complex character.
    #[cfg(feature = "wide")]
    pub fn vline_set(&mut self, wch: &CCharT, n: i32) -> Result<()> {
        let wch = if wch.spacing_char() == '\0' {
            CCharT::from_char('│')
        } else {
            *wch
        };
        let n = Self::line_len(n, (self.maxy - self.cury + 1) as i32);
        let y = self.cury as usize;
        let x = self.curx as usize;

        for i in 0..n {
            self.lines[y + i].set(x, wch);
        }

        Ok(())
    }

    // ACS character helpers: the VT100 line-drawing codes with A_ALTCHARSET
    #[cfg(not(feature = "wide"))]
    fn acs_hline(&self) -> ChType {
        crate::acs::acs_char(crate::acs::ACS_HLINE)
    }
    #[cfg(not(feature = "wide"))]
    fn acs_vline(&self) -> ChType {
        crate::acs::acs_char(crate::acs::ACS_VLINE)
    }
    #[cfg(not(feature = "wide"))]
    fn acs_ulcorner(&self) -> ChType {
        crate::acs::acs_char(crate::acs::ACS_ULCORNER)
    }
    #[cfg(not(feature = "wide"))]
    fn acs_urcorner(&self) -> ChType {
        crate::acs::acs_char(crate::acs::ACS_URCORNER)
    }
    #[cfg(not(feature = "wide"))]
    fn acs_llcorner(&self) -> ChType {
        crate::acs::acs_char(crate::acs::ACS_LLCORNER)
    }
    #[cfg(not(feature = "wide"))]
    fn acs_lrcorner(&self) -> ChType {
        crate::acs::acs_char(crate::acs::ACS_LRCORNER)
    }

    // ========================================================================
    // Insert/Delete
    // ========================================================================

    /// Insert a character at the current position.
    pub fn insch(&mut self, ch: ChType) -> Result<()> {
        let y = self.cury as usize;
        let x = self.curx as usize;

        #[cfg(not(feature = "wide"))]
        self.lines[y].insert(x, ch | self.attrs, 1);
        #[cfg(feature = "wide")]
        self.lines[y].insert(x, CCharT::from_chtype(ch | self.attrs), 1);

        Ok(())
    }

    /// Insert a string at the current cursor position.
    ///
    /// Characters to the right of the cursor are shifted right.
    /// Characters that are shifted off the right edge of the window are lost.
    /// The cursor position does not change.
    pub fn insstr(&mut self, s: &str) -> Result<()> {
        self.insnstr(s, -1)
    }

    /// Insert at most n characters of a string at the current cursor position.
    ///
    /// Characters to the right of the cursor are shifted right.
    /// Characters that are shifted off the right edge of the window are lost.
    /// The cursor position does not change.
    ///
    /// If n is negative, the entire string is inserted.
    pub fn insnstr(&mut self, s: &str, n: i32) -> Result<()> {
        let y = self.cury as usize;
        let x = self.curx as usize;

        // Determine how many characters to insert
        let chars: Vec<char> = s.chars().collect();
        let count = if n < 0 {
            chars.len()
        } else {
            chars.len().min(n as usize)
        };

        // Insert characters one at a time from right to left
        // so the first character ends up at the cursor position
        for i in (0..count).rev() {
            #[cfg(not(feature = "wide"))]
            {
                let ch = chars[i] as ChType | self.attrs;
                self.lines[y].insert(x, ch, 1);
            }
            #[cfg(feature = "wide")]
            {
                let attr = self.attrs;
                self.lines[y].insert(x, CCharT::from_char_attr(chars[i], attr), 1);
            }
        }

        Ok(())
    }

    /// Move cursor and insert a string.
    pub fn mvinsstr(&mut self, y: i32, x: i32, s: &str) -> Result<()> {
        self.mv(y, x)?;
        self.insstr(s)
    }

    /// Move cursor and insert at most n characters of a string.
    pub fn mvinsnstr(&mut self, y: i32, x: i32, s: &str, n: i32) -> Result<()> {
        self.mv(y, x)?;
        self.insnstr(s, n)
    }

    /// Delete the character at the current position.
    pub fn delch(&mut self) -> Result<()> {
        let y = self.cury as usize;
        let x = self.curx as usize;

        #[cfg(not(feature = "wide"))]
        self.lines[y].delete(x, 1, self.bkgd);
        #[cfg(feature = "wide")]
        self.lines[y].delete(x, 1, self.bkgrnd);

        Ok(())
    }

    /// Insert a blank line above the current line.
    pub fn insertln(&mut self) -> Result<()> {
        self.insdelln(1)
    }

    /// Delete the current line.
    pub fn deleteln(&mut self) -> Result<()> {
        self.insdelln(-1)
    }

    /// Insert or delete lines at the cursor.
    ///
    /// A positive `n` inserts `n` blank lines above the current line: it and
    /// the lines below move down, and the last `n` lines of the window are
    /// lost. A negative `n` deletes that many lines starting with the current
    /// one: the lines below move up, and blank lines fill the bottom. Lines
    /// above the cursor are not touched, and the scroll region plays no part.
    pub fn insdelln(&mut self, n: i32) -> Result<()> {
        let (top, bottom) = (self.cury as usize, self.maxy as usize);
        if n > 0 {
            self.shift_lines_down(n, top, bottom);
        } else if n < 0 {
            self.shift_lines_up(n.saturating_neg(), top, bottom);
        }
        Ok(())
    }

    // ========================================================================
    // Wide character insert/read operations
    // ========================================================================

    /// Insert a wide character at the current position.
    ///
    /// Characters to the right of the cursor are shifted right.
    /// Characters shifted off the right edge are lost.
    /// The cursor position does not change.
    #[cfg(feature = "wide")]
    pub fn ins_wch(&mut self, wch: &CCharT) -> Result<()> {
        self.wins_wch(wch)
    }

    /// Insert a wide character at the current position (window version).
    #[cfg(feature = "wide")]
    pub fn wins_wch(&mut self, wch: &CCharT) -> Result<()> {
        let y = self.cury as usize;
        let x = self.curx as usize;

        if y > self.maxy as usize || x > self.maxx as usize {
            return Err(Error::OutOfBounds {
                y: y as i32,
                x: x as i32,
                max_y: self.maxy as i32,
                max_x: self.maxx as i32,
            });
        }

        // Merge attributes: wch attributes take precedence, but apply window attrs if wch has none
        let mut cchar = *wch;
        if cchar.attrs() == A_NORMAL {
            cchar.attr |= self.attrs;
        }

        let width = cchar.width().max(1);
        self.lines[y].insert(x, cchar, width);

        Ok(())
    }

    /// Move cursor and insert a wide character.
    #[cfg(feature = "wide")]
    pub fn mvins_wch(&mut self, y: i32, x: i32, wch: &CCharT) -> Result<()> {
        self.mv(y, x)?;
        self.ins_wch(wch)
    }

    /// Move cursor and insert a wide character (window version).
    #[cfg(feature = "wide")]
    pub fn mvwins_wch(&mut self, y: i32, x: i32, wch: &CCharT) -> Result<()> {
        self.mv(y, x)?;
        self.wins_wch(wch)
    }

    /// Read the wide character at the current position.
    ///
    /// Returns the complex character including attributes at the cursor position.
    #[cfg(feature = "wide")]
    #[must_use]
    pub fn in_wch(&self) -> CCharT {
        self.win_wch()
    }

    /// Read the wide character at the current position (window version).
    #[cfg(feature = "wide")]
    #[must_use]
    pub fn win_wch(&self) -> CCharT {
        let y = self.cury as usize;
        let x = self.curx as usize;

        if y <= self.maxy as usize && x <= self.maxx as usize {
            self.lines[y].get(x)
        } else {
            CCharT::new()
        }
    }

    /// Move cursor and read a wide character.
    #[cfg(feature = "wide")]
    pub fn mvin_wch(&mut self, y: i32, x: i32) -> Result<CCharT> {
        self.mv(y, x)?;
        Ok(self.in_wch())
    }

    /// Move cursor and read a wide character (window version).
    #[cfg(feature = "wide")]
    pub fn mvwin_wch(&mut self, y: i32, x: i32) -> Result<CCharT> {
        self.mv(y, x)?;
        Ok(self.win_wch())
    }

    /// Get a string of wide characters from the current position.
    ///
    /// This reads complex characters into a slice of CCharT values.
    /// Returns the number of characters read.
    ///
    /// If `n` is negative, reads to the end of the line.
    #[cfg(feature = "wide")]
    pub fn in_wchnstr(&self, wchstr: &mut [CCharT], n: i32) -> i32 {
        let y = self.cury as usize;
        let max_chars = if n < 0 {
            wchstr
                .len()
                .min((self.maxx as usize + 1).saturating_sub(self.curx as usize))
        } else {
            wchstr
                .len()
                .min(n as usize)
                .min((self.maxx as usize + 1).saturating_sub(self.curx as usize))
        };

        let mut count = 0;
        for (i, wch) in wchstr.iter_mut().take(max_chars).enumerate() {
            let x = self.curx as usize + i;
            if x > self.maxx as usize {
                break;
            }
            *wch = self.lines[y].get(x);
            count += 1;
        }

        count
    }

    /// Move to position and get a string of wide characters.
    #[cfg(feature = "wide")]
    pub fn mvin_wchnstr(&mut self, y: i32, x: i32, wchstr: &mut [CCharT], n: i32) -> Result<i32> {
        self.mv(y, x)?;
        Ok(self.in_wchnstr(wchstr, n))
    }

    // ========================================================================
    // Wide character borders and boxes
    // ========================================================================

    /// Draw a border using wide characters.
    ///
    /// Arguments are: left, right, top, bottom sides, and the four corners
    /// (top-left, top-right, bottom-left, bottom-right).
    /// If any argument is None, the default line-drawing character is used.
    #[cfg(feature = "wide")]
    #[allow(clippy::too_many_arguments)]
    pub fn border_set(
        &mut self,
        ls: Option<&CCharT>,
        rs: Option<&CCharT>,
        ts: Option<&CCharT>,
        bs: Option<&CCharT>,
        tl: Option<&CCharT>,
        tr: Option<&CCharT>,
        bl: Option<&CCharT>,
        br: Option<&CCharT>,
    ) -> Result<()> {
        self.wborder_set(ls, rs, ts, bs, tl, tr, bl, br)
    }

    /// Draw a border using wide characters (window version).
    #[cfg(feature = "wide")]
    #[allow(clippy::too_many_arguments)]
    pub fn wborder_set(
        &mut self,
        ls: Option<&CCharT>,
        rs: Option<&CCharT>,
        ts: Option<&CCharT>,
        bs: Option<&CCharT>,
        tl: Option<&CCharT>,
        tr: Option<&CCharT>,
        bl: Option<&CCharT>,
        br: Option<&CCharT>,
    ) -> Result<()> {
        use crate::wide::wacs;

        // Use defaults if None
        let ls = ls.unwrap_or(&wacs::VLINE);
        let rs = rs.unwrap_or(&wacs::VLINE);
        let ts = ts.unwrap_or(&wacs::HLINE);
        let bs = bs.unwrap_or(&wacs::HLINE);
        let tl = tl.unwrap_or(&wacs::ULCORNER);
        let tr = tr.unwrap_or(&wacs::URCORNER);
        let bl = bl.unwrap_or(&wacs::LLCORNER);
        let br = br.unwrap_or(&wacs::LRCORNER);

        let maxy = self.maxy as usize;
        let maxx = self.maxx as usize;

        // Corners
        self.lines[0].set(0, *tl);
        self.lines[0].set(maxx, *tr);
        self.lines[maxy].set(0, *bl);
        self.lines[maxy].set(maxx, *br);

        // Top and bottom edges
        for x in 1..maxx {
            self.lines[0].set(x, *ts);
            self.lines[maxy].set(x, *bs);
        }

        // Left and right edges
        for line in self.lines.iter_mut().take(maxy).skip(1) {
            line.set(0, *ls);
            line.set(maxx, *rs);
        }

        self.touchwin();
        Ok(())
    }

    /// Draw a box using wide characters.
    ///
    /// This is a shorthand for border_set with the same character for
    /// vertical sides and the same character for horizontal sides.
    #[cfg(feature = "wide")]
    pub fn box_set(&mut self, verch: Option<&CCharT>, horch: Option<&CCharT>) -> Result<()> {
        self.border_set(verch, verch, horch, horch, None, None, None, None)
    }

    // ========================================================================
    // Wide character background
    // ========================================================================

    /// Set the background character using a wide character (no window update).
    ///
    /// Sets the background property of the window. The background character
    /// is used for clearing and as the fill character for insert operations.
    #[cfg(feature = "wide")]
    pub fn bkgrndset(&mut self, wch: &CCharT) {
        self.wbkgrndset(wch);
    }

    /// Set the background character using a wide character (window version, no update).
    #[cfg(feature = "wide")]
    pub fn wbkgrndset(&mut self, wch: &CCharT) {
        self.bkgrnd = *wch;
    }

    /// Set the background and apply to the entire window.
    ///
    /// This sets the background property and also updates all character cells
    /// in the window, replacing the old background character with the new one.
    #[cfg(feature = "wide")]
    pub fn bkgrnd_set(&mut self, wch: &CCharT) -> Result<()> {
        self.wbkgrnd(wch)
    }

    /// Set the background and apply to the entire window (window version).
    #[cfg(feature = "wide")]
    pub fn wbkgrnd(&mut self, wch: &CCharT) -> Result<()> {
        let old_bkgrnd = self.bkgrnd;
        self.bkgrnd = *wch;

        let old_char = old_bkgrnd.spacing_char();
        let old_attr = old_bkgrnd.attrs();
        let new_char = wch.spacing_char();
        let new_attr = wch.attrs();

        // Update all cells
        for line in &mut self.lines {
            for x in 0..line.width() {
                let cell = line.get(x);
                let cell_char = cell.spacing_char();
                let cell_attr = cell.attrs();

                // Replace old background char with new
                let updated_char = if cell_char == old_char || cell_char == ' ' {
                    new_char
                } else {
                    cell_char
                };

                // Update attributes: remove old background attrs, add new ones
                let updated_attr = (cell_attr & !old_attr) | new_attr;

                line.set(x, CCharT::from_char_attr(updated_char, updated_attr));
            }
        }

        self.touchwin();
        Ok(())
    }

    /// Get the background wide character.
    #[cfg(feature = "wide")]
    #[must_use]
    pub fn getbkgrnd(&self) -> CCharT {
        self.wgetbkgrnd()
    }

    /// Get the background wide character (window version).
    #[cfg(feature = "wide")]
    #[must_use]
    pub fn wgetbkgrnd(&self) -> CCharT {
        self.bkgrnd
    }

    // ========================================================================
    // Touch/change tracking
    // ========================================================================

    /// Mark the entire window as changed.
    pub fn touchwin(&mut self) {
        for line in &mut self.lines {
            line.touch();
        }
    }

    /// Mark the entire window as unchanged.
    pub fn untouchwin(&mut self) {
        for line in &mut self.lines {
            line.untouch();
        }
    }

    /// Mark `count` lines from `start` as changed or unchanged.
    ///
    /// Lines past the end of the window are ignored. Nothing happens when
    /// `start` or `count` is negative, or `start` is past the last line.
    pub fn touchln(&mut self, start: i32, count: i32, changed: bool) {
        let (Ok(start), Ok(count)) = (usize::try_from(start), usize::try_from(count)) else {
            return;
        };
        let end = start.saturating_add(count).min(self.lines.len());
        let Some(lines) = self.lines.get_mut(start..end) else {
            return;
        };

        for line in lines {
            if changed {
                line.touch();
            } else {
                line.untouch();
            }
        }
    }

    /// Check if a line has been touched.
    #[must_use]
    pub fn is_linetouched(&self, line: i32) -> bool {
        if line < 0 || line > self.maxy as i32 {
            false
        } else {
            self.lines[line as usize].is_touched()
        }
    }

    /// Check if any line in the window has been touched.
    #[must_use]
    pub fn is_wintouched(&self) -> bool {
        self.lines.iter().any(|l| l.is_touched())
    }

    // ========================================================================
    // Window options
    // ========================================================================

    /// Enable/disable scrolling.
    pub fn scrollok(&mut self, bf: bool) {
        self.scroll = bf;
    }

    /// Check if scrolling is enabled.
    #[must_use]
    pub fn is_scrollok(&self) -> bool {
        self.scroll
    }

    /// Enable/disable keypad mode.
    pub fn keypad(&mut self, bf: bool) {
        self.use_keypad = bf;
    }

    /// Check if keypad mode is enabled.
    #[must_use]
    pub fn is_keypad(&self) -> bool {
        self.use_keypad
    }

    /// Enable/disable nodelay mode.
    pub fn nodelay(&mut self, bf: bool) {
        self.delay = if bf { 0 } else { -1 };
    }

    /// Check if nodelay mode is enabled.
    #[must_use]
    pub fn is_nodelay(&self) -> bool {
        self.delay == 0
    }

    /// Set the input timeout.
    pub fn timeout(&mut self, delay: i32) {
        self.delay = delay;
    }

    /// Get the input delay.
    #[must_use]
    pub fn getdelay(&self) -> i32 {
        self.delay
    }

    /// Enable/disable leaveok mode.
    pub fn leaveok(&mut self, bf: bool) {
        self.leaveok = bf;
    }

    /// Check if leaveok mode is enabled.
    #[must_use]
    pub fn is_leaveok(&self) -> bool {
        self.leaveok
    }

    /// Enable/disable clearok mode.
    pub fn clearok(&mut self, bf: bool) {
        self.clear = bf;
    }

    /// Check if clearok mode is enabled.
    #[must_use]
    pub fn is_cleared(&self) -> bool {
        self.clear
    }

    /// Enable/disable idlok mode.
    pub fn idlok(&mut self, bf: bool) {
        self.idlok = bf;
    }

    /// Check if idlok mode is enabled.
    #[must_use]
    pub fn is_idlok(&self) -> bool {
        self.idlok
    }

    /// Enable/disable idcok mode.
    pub fn idcok(&mut self, bf: bool) {
        self.idcok = bf;
    }

    /// Check if idcok mode is enabled.
    #[must_use]
    pub fn is_idcok(&self) -> bool {
        self.idcok
    }

    /// Enable/disable immedok mode.
    pub fn immedok(&mut self, bf: bool) {
        self.immed = bf;
    }

    /// Check if immedok mode is enabled.
    #[must_use]
    pub fn is_immedok(&self) -> bool {
        self.immed
    }

    /// Enable/disable syncok mode.
    pub fn syncok(&mut self, bf: bool) {
        self.sync = bf;
    }

    /// Check if syncok mode is enabled.
    #[must_use]
    pub fn is_syncok(&self) -> bool {
        self.sync
    }

    /// Enable/disable notimeout mode.
    pub fn notimeout(&mut self, bf: bool) {
        self.notimeout = bf;
    }

    /// Check if notimeout mode is enabled.
    #[must_use]
    pub fn is_notimeout(&self) -> bool {
        self.notimeout
    }

    /// Set the scrolling region.
    pub fn setscrreg(&mut self, top: i32, bot: i32) -> Result<()> {
        if top < 0 || bot > self.maxy as i32 || top > bot {
            return Err(Error::InvalidArgument("invalid scrolling region".into()));
        }
        self.regtop = top as NcursesSize;
        self.regbottom = bot as NcursesSize;
        Ok(())
    }

    /// Get the scrolling region.
    #[must_use]
    pub fn getscrreg(&self) -> (i32, i32) {
        (self.regtop as i32, self.regbottom as i32)
    }

    // ========================================================================
    // Window flags
    // ========================================================================

    /// Check if this is a pad.
    #[must_use]
    pub fn is_pad(&self) -> bool {
        self.flags.contains(WindowFlags::ISPAD)
    }

    /// Check if this is a subwindow.
    #[must_use]
    pub fn is_subwin(&self) -> bool {
        self.flags.contains(WindowFlags::SUBWIN)
    }

    // ========================================================================
    // Pad support
    // ========================================================================

    /// Get the pad data for this window.
    ///
    /// Returns `None` if this is not a pad.
    pub fn pad_data(&self) -> Option<&PadData> {
        if self.is_pad() { Some(&self.pad) } else { None }
    }

    /// Get mutable pad data for this window.
    ///
    /// Returns `None` if this is not a pad.
    pub fn pad_data_mut(&mut self) -> Option<&mut PadData> {
        if self.is_pad() {
            Some(&mut self.pad)
        } else {
            None
        }
    }

    /// Set the pad refresh parameters.
    ///
    /// These parameters are used by `prefresh` and `pnoutrefresh` to determine
    /// which portion of the pad to display and where on the screen.
    ///
    /// # Arguments
    ///
    /// * `pminrow` - Row in pad to start display from
    /// * `pmincol` - Column in pad to start display from
    /// * `sminrow` - Top row on screen for display
    /// * `smincol` - Left column on screen for display
    /// * `smaxrow` - Bottom row on screen for display
    /// * `smaxcol` - Right column on screen for display
    pub fn set_pad_params(
        &mut self,
        pminrow: i32,
        pmincol: i32,
        sminrow: i32,
        smincol: i32,
        smaxrow: i32,
        smaxcol: i32,
    ) -> Result<()> {
        if !self.is_pad() {
            return Err(Error::InvalidArgument("not a pad".into()));
        }

        if pminrow < 0 || pmincol < 0 || sminrow < 0 || smincol < 0 {
            return Err(Error::InvalidArgument("negative pad parameters".into()));
        }

        if smaxrow < sminrow || smaxcol < smincol {
            return Err(Error::InvalidArgument("invalid screen region".into()));
        }

        self.pad.pad_y = pminrow as NcursesSize;
        self.pad.pad_x = pmincol as NcursesSize;
        self.pad.pad_top = sminrow as NcursesSize;
        self.pad.pad_left = smincol as NcursesSize;
        self.pad.pad_bottom = smaxrow as NcursesSize;
        self.pad.pad_right = smaxcol as NcursesSize;

        Ok(())
    }

    /// Create a subpad within this pad.
    ///
    /// Similar to `subwin` but for pads. The subpad shares the parent pad's
    /// character storage (conceptually - Rust implementation uses separate storage).
    ///
    /// # Arguments
    ///
    /// * `nlines` - Number of lines (height) of the subpad
    /// * `ncols` - Number of columns (width) of the subpad
    /// * `begy` - Y coordinate relative to parent pad
    /// * `begx` - X coordinate relative to parent pad
    pub fn subpad(&self, nlines: i32, ncols: i32, begy: i32, begx: i32) -> Result<Self> {
        if !self.is_pad() {
            return Err(Error::InvalidArgument("not a pad".into()));
        }

        if nlines < 0 || ncols < 0 || begy < 0 || begx < 0 {
            return Err(Error::InvalidArgument("negative subpad dimensions".into()));
        }

        // Check bounds
        let height = if nlines == 0 {
            self.getmaxy() - begy
        } else {
            nlines
        };
        let width = if ncols == 0 {
            self.getmaxx() - begx
        } else {
            ncols
        };

        if begy + height > self.getmaxy() || begx + width > self.getmaxx() {
            return Err(Error::InvalidArgument(
                "subpad extends beyond parent boundaries".into(),
            ));
        }

        let mut win = Self::new_pad(height, width)?;
        win.flags |= WindowFlags::SUBWIN;
        win.pary = begy;
        win.parx = begx;

        Ok(win)
    }

    // ========================================================================
    // Line access (for refresh)
    // ========================================================================

    /// Get a reference to a line.
    pub fn line(&self, y: usize) -> Option<&LineData> {
        self.lines.get(y)
    }

    /// Get a mutable reference to a line.
    pub fn line_mut(&mut self, y: usize) -> Option<&mut LineData> {
        self.lines.get_mut(y)
    }

    /// Get all lines.
    pub fn lines(&self) -> &[LineData] {
        &self.lines
    }

    /// Clear the "clear screen" flag and return its previous value.
    pub fn take_clear_flag(&mut self) -> bool {
        let was_clear = self.clear;
        self.clear = false;
        was_clear
    }
}

impl std::fmt::Debug for Window {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Window")
            .field("size", &(self.getmaxy(), self.getmaxx()))
            .field("position", &(self.begy, self.begx))
            .field("cursor", &(self.cury, self.curx))
            .field("flags", &self.flags)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attr::{A_BOLD, A_UNDERLINE};

    #[test]
    fn test_window_creation() {
        let win = Window::new(24, 80, 0, 0).unwrap();
        assert_eq!(win.getmaxy(), 24);
        assert_eq!(win.getmaxx(), 80);
        assert_eq!(win.getcury(), 0);
        assert_eq!(win.getcurx(), 0);
    }

    #[test]
    fn test_cursor_movement() {
        let mut win = Window::new(24, 80, 0, 0).unwrap();
        win.mv(10, 20).unwrap();
        assert_eq!(win.getcury(), 10);
        assert_eq!(win.getcurx(), 20);
    }

    #[test]
    fn test_addstr() {
        let mut win = Window::new(24, 80, 0, 0).unwrap();
        win.addstr("Hello").unwrap();
        assert_eq!(win.getcurx(), 5);
    }

    #[test]
    fn test_border() {
        let mut win = Window::new(10, 20, 0, 0).unwrap();
        win.box_(0, 0).unwrap();
        // Check corners are set
        assert!(win.is_wintouched());
    }

    #[test]
    fn test_scrolling() {
        let mut win = Window::new(5, 10, 0, 0).unwrap();
        win.scrollok(true);
        win.scroll_up(1).unwrap();
        assert!(win.is_wintouched());
    }

    /// The text of every line of a three-column window.
    fn rows(win: &mut Window) -> Vec<String> {
        rows_of(win, 3)
    }

    /// The first `width` columns of every line of a window.
    fn rows_of(win: &mut Window, width: i32) -> Vec<String> {
        (0..win.getmaxy())
            .map(|y| {
                win.mv(y, 0).unwrap();
                win.instr(width)
            })
            .collect()
    }

    #[cfg(feature = "wide")]
    #[test]
    fn test_default_border_uses_line_drawing_characters() {
        let mut win = Window::new(3, 3, 0, 0).unwrap();
        win.box_(0, 0).unwrap();
        assert_eq!(rows(&mut win), ["┌─┐", "│ │", "└─┘"]);

        let mut win = Window::new(3, 3, 0, 0).unwrap();
        win.border(0, 0, 0, 0, 0, 0, 0, 0).unwrap();
        assert_eq!(rows(&mut win), ["┌─┐", "│ │", "└─┘"]);

        let mut win = Window::new(3, 3, 0, 0).unwrap();
        win.draw_border(crate::types::BorderChars::default())
            .unwrap();
        assert_eq!(rows(&mut win), ["┌─┐", "│ │", "└─┘"]);
    }

    #[cfg(feature = "wide")]
    #[test]
    fn test_border_mixes_given_and_default_characters() {
        let mut win = Window::new(3, 3, 0, 0).unwrap();
        win.box_('|' as ChType, '-' as ChType).unwrap();
        assert_eq!(rows(&mut win), ["┌-┐", "| |", "└-┘"]);

        let mut win = Window::new(3, 3, 0, 0).unwrap();
        let (l, r, t, b) = ('l' as ChType, 'r' as ChType, 't' as ChType, 'b' as ChType);
        win.border(
            l,
            r,
            t,
            b,
            '1' as ChType,
            '2' as ChType,
            '3' as ChType,
            '4' as ChType,
        )
        .unwrap();
        assert_eq!(rows(&mut win), ["1t2", "l r", "3b4"]);
    }

    #[cfg(feature = "wide")]
    #[test]
    fn test_border_keeps_the_attributes_of_a_given_character() {
        let mut win = Window::new(3, 3, 0, 0).unwrap();
        win.box_('|' as ChType | A_BOLD, 0).unwrap();
        let side = win.lines[1].get(0);
        assert_eq!((side.spacing_char(), side.attr), ('|', A_BOLD));
        // A default character carries no attributes of its own.
        let top = win.lines[0].get(1);
        assert_eq!((top.spacing_char(), top.attr), ('─', A_NORMAL));
    }

    #[cfg(feature = "wide")]
    #[test]
    fn test_default_hline_and_vline_use_line_drawing_characters() {
        let mut win = Window::new(3, 3, 0, 0).unwrap();
        win.hline(0, 3).unwrap();
        win.mv(1, 1).unwrap();
        win.vline(0, 2).unwrap();
        assert_eq!(rows(&mut win), ["───", " │ ", " │ "]);

        // The window's attributes apply to the default character too.
        let mut win = Window::new(1, 3, 0, 0).unwrap();
        win.attron(A_BOLD).unwrap();
        win.hline(0, 1).unwrap();
        win.vline('!' as ChType | A_UNDERLINE, 1).unwrap();
        let cell = win.lines[0].get(0);
        assert_eq!(
            (cell.spacing_char(), cell.attr),
            ('!', A_BOLD | A_UNDERLINE)
        );
        win.hline(0, 1).unwrap();
        let cell = win.lines[0].get(0);
        assert_eq!((cell.spacing_char(), cell.attr), ('─', A_BOLD));
    }

    #[cfg(feature = "wide")]
    #[test]
    fn test_acs_characters_can_be_drawn_through_a_chtype() {
        use crate::acs::*;

        let mut win = Window::new(3, 3, 0, 0).unwrap();
        win.addch(acs_char(ACS_ULCORNER)).unwrap();
        win.addch(acs_char(ACS_HLINE)).unwrap();
        win.addch(acs_char(ACS_URCORNER)).unwrap();
        win.mv(1, 0).unwrap();
        win.insch(acs_char(ACS_VLINE)).unwrap();
        win.mv(2, 0).unwrap();
        win.addchstr(&[
            acs_char(ACS_LLCORNER),
            acs_char(ACS_DIAMOND),
            acs_char(ACS_PI),
        ])
        .unwrap();
        assert_eq!(rows(&mut win), ["┌─┐", "│  ", "└◆π"]);

        // The cell holds the character itself, without A_ALTCHARSET.
        let cell = win.lines[0].get(1);
        assert_eq!((cell.spacing_char(), cell.attr), ('─', A_NORMAL));
    }

    #[cfg(feature = "wide")]
    #[test]
    fn test_acs_characters_keep_their_other_attributes() {
        use crate::acs::*;

        let mut win = Window::new(1, 3, 0, 0).unwrap();
        win.addch(acs_char(ACS_HLINE) | A_BOLD).unwrap();
        win.attron(A_UNDERLINE).unwrap();
        win.addch(acs_char(ACS_VLINE)).unwrap();
        let cells = [win.lines[0].get(0), win.lines[0].get(1)];
        assert_eq!((cells[0].spacing_char(), cells[0].attr), ('─', A_BOLD));
        assert_eq!((cells[1].spacing_char(), cells[1].attr), ('│', A_UNDERLINE));
    }

    #[cfg(feature = "wide")]
    #[test]
    fn test_altcharset_attribute_turns_codes_into_line_drawing() {
        use crate::attr::A_ALTCHARSET;

        let mut win = Window::new(1, 3, 0, 0).unwrap();
        win.attron(A_ALTCHARSET).unwrap();
        win.addch('l' as ChType).unwrap();
        win.addch('q' as ChType).unwrap();
        // A character that is not a line-drawing code stays as it is.
        win.addch('A' as ChType).unwrap();
        assert_eq!(rows(&mut win), ["┌─A"]);
    }

    #[cfg(feature = "wide")]
    #[test]
    fn test_acs_characters_in_borders_and_lines() {
        use crate::acs::*;

        let mut win = Window::new(3, 3, 0, 0).unwrap();
        win.box_(acs_char(ACS_VLINE), acs_char(ACS_HLINE)).unwrap();
        assert_eq!(rows(&mut win), ["┌─┐", "│ │", "└─┘"]);

        let mut win = Window::new(3, 3, 0, 0).unwrap();
        let (tee, plus) = (acs_char(ACS_TTEE), acs_char(ACS_PLUS));
        win.border(0, 0, 0, 0, tee, tee, plus, plus).unwrap();
        assert_eq!(rows(&mut win), ["┬─┬", "│ │", "┼─┼"]);

        let mut win = Window::new(2, 3, 0, 0).unwrap();
        win.hline(acs_char(ACS_S9), 3).unwrap();
        win.mv(0, 1).unwrap();
        win.vline(acs_char(ACS_CKBOARD), 2).unwrap();
        assert_eq!(rows(&mut win), ["⎽▒⎽", " ▒ "]);
    }

    #[cfg(feature = "wide")]
    #[test]
    fn test_acs_characters_read_back_as_the_chtype_that_drew_them() {
        use crate::acs::*;

        let mut win = Window::new(1, 4, 0, 0).unwrap();
        let drawn = [
            acs_char(ACS_HLINE) | A_BOLD,
            acs_char(ACS_LARROW),
            'x' as ChType | A_UNDERLINE,
            0xE9, // é fits in a ChType as it is
        ];
        win.addchstr(&drawn).unwrap();

        let mut read = [0; 4];
        assert_eq!(win.inchnstr(&mut read, 4), 4);
        assert_eq!(read, drawn);
        win.mv(0, 1).unwrap();
        assert_eq!(win.inch(), acs_char(ACS_LARROW));

        // Drawing what was read gives the same characters again.
        let mut copy = Window::new(1, 4, 0, 0).unwrap();
        copy.addchstr(&read).unwrap();
        assert_eq!(rows_of(&mut copy, 4), rows_of(&mut win, 4));
        assert_eq!(rows_of(&mut copy, 4), ["─←xé"]);
    }

    #[cfg(feature = "wide")]
    #[test]
    fn test_acs_background_character() {
        use crate::acs::*;

        let mut win = Window::new(1, 2, 0, 0).unwrap();
        win.bkgdset(acs_char(ACS_BOARD));
        assert_eq!(win.getbkgd(), acs_char(ACS_BOARD));
        win.erase().unwrap();
        assert_eq!(rows_of(&mut win, 2), ["▓▓"]);
    }

    #[test]
    fn test_border_of_a_one_cell_window_does_not_panic() {
        for (lines, cols) in [(1, 1), (1, 4), (4, 1), (2, 2)] {
            let mut win = Window::new(lines, cols, 0, 0).unwrap();
            win.box_(0, 0).unwrap();
        }
    }

    #[test]
    fn test_hline_and_vline_draw_nothing_for_a_negative_count() {
        let mut win = Window::new(3, 3, 0, 0).unwrap();
        win.mv(1, 1).unwrap();
        for n in [0, -1, i32::MIN] {
            win.hline('-' as ChType, n).unwrap();
            win.vline('|' as ChType, n).unwrap();
        }
        assert_eq!(rows(&mut win), ["   ", "   ", "   "]);
    }

    #[cfg(feature = "wide")]
    #[test]
    fn test_hline_set_and_vline_set_draw_nothing_for_a_negative_count() {
        let mut win = Window::new(3, 3, 0, 0).unwrap();
        win.mv(1, 1).unwrap();
        let ch = CCharT::from_char('x');
        for n in [0, -1, i32::MIN] {
            win.hline_set(&ch, n).unwrap();
            win.vline_set(&ch, n).unwrap();
        }
        assert_eq!(rows(&mut win), ["   ", "   ", "   "]);
    }

    #[test]
    fn test_hline_and_vline_stop_at_the_window_edge() {
        let mut win = Window::new(3, 3, 0, 0).unwrap();
        win.mv(1, 1).unwrap();
        win.hline('-' as ChType, 10).unwrap();
        win.vline('|' as ChType, 10).unwrap();
        assert_eq!(rows(&mut win), ["   ", " |-", " | "]);
    }

    /// Which lines of the window are marked as changed.
    fn touched(win: &Window) -> Vec<bool> {
        (0..win.getmaxy()).map(|y| win.is_linetouched(y)).collect()
    }

    #[test]
    fn test_touchln_marks_the_lines_inside_the_window() {
        let mut win = Window::new(5, 3, 0, 0).unwrap();
        win.untouchwin();
        win.touchln(1, 2, true);
        assert_eq!(touched(&win), [false, true, true, false, false]);

        // A range that runs past the last line is cut off there.
        win.untouchwin();
        win.touchln(3, 10, true);
        assert_eq!(touched(&win), [false, false, false, true, true]);
        win.touchln(3, i32::MAX, false);
        assert_eq!(touched(&win), [false; 5]);
    }

    #[test]
    fn test_touchln_ignores_out_of_range_arguments() {
        let mut win = Window::new(5, 3, 0, 0).unwrap();
        win.untouchwin();
        for (start, count) in [
            (5, 1),
            (10, 1),
            (i32::MAX, i32::MAX),
            (1, -1),
            (-1, 2),
            (0, 0),
        ] {
            win.touchln(start, count, true);
            assert_eq!(touched(&win), [false; 5], "touchln({start}, {count})");
        }
    }

    #[test]
    fn test_resize_keeps_the_scroll_region_inside_the_window() {
        let mut win = Window::new(10, 3, 0, 0).unwrap();
        win.scrollok(true);
        win.setscrreg(5, 9).unwrap();
        win.resize(3, 3).unwrap();
        assert_eq!(win.getscrreg(), (2, 2));

        // Scrolling the shrunken region used to panic.
        for (y, text) in ["aaa", "bbb", "ccc"].into_iter().enumerate() {
            win.mvaddstr(y as i32, 0, text).unwrap();
        }
        win.scrl(1).unwrap();
        assert_eq!(rows(&mut win), ["aaa", "bbb", "   "]);
        win.scrl(-1).unwrap();
        win.scroll_up(1).unwrap();
        win.scroll_down(1).unwrap();
    }

    #[test]
    fn test_resize_keeps_a_scroll_region_that_still_fits() {
        let mut win = Window::new(10, 3, 0, 0).unwrap();
        win.setscrreg(1, 3).unwrap();
        win.resize(6, 3).unwrap();
        assert_eq!(win.getscrreg(), (1, 3));
        win.resize(3, 3).unwrap();
        assert_eq!(win.getscrreg(), (1, 2));
    }

    #[test]
    fn test_resize_keeps_a_whole_window_scroll_region_whole() {
        let mut win = Window::new(3, 3, 0, 0).unwrap();
        assert_eq!(win.getscrreg(), (0, 2));
        win.resize(5, 3).unwrap();
        assert_eq!(win.getscrreg(), (0, 4));
        win.resize(2, 3).unwrap();
        assert_eq!(win.getscrreg(), (0, 1));

        // The lines added by growing scroll with the rest.
        win.resize(3, 3).unwrap();
        for (y, text) in ["aaa", "bbb", "ccc"].into_iter().enumerate() {
            win.mvaddstr(y as i32, 0, text).unwrap();
        }
        win.scroll_up(1).unwrap();
        assert_eq!(rows(&mut win), ["bbb", "ccc", "   "]);
    }

    /// A five-line window holding `aaa` to `eee`, with the cursor on `row`.
    fn five_lines(row: i32) -> Window {
        let mut win = Window::new(5, 3, 0, 0).unwrap();
        for (y, text) in ["aaa", "bbb", "ccc", "ddd", "eee"].into_iter().enumerate() {
            win.mvaddstr(y as i32, 0, text).unwrap();
        }
        win.mv(row, 0).unwrap();
        win
    }

    #[test]
    fn test_insertln_inserts_above_the_cursor_line() {
        let mut win = five_lines(2);
        win.insertln().unwrap();
        assert_eq!(rows(&mut win), ["aaa", "bbb", "   ", "ccc", "ddd"]);
    }

    #[test]
    fn test_deleteln_deletes_the_cursor_line() {
        let mut win = five_lines(2);
        win.deleteln().unwrap();
        assert_eq!(rows(&mut win), ["aaa", "bbb", "ddd", "eee", "   "]);
    }

    #[test]
    fn test_insdelln_counts_are_cut_off_at_the_bottom_of_the_window() {
        let mut win = five_lines(3);
        win.insdelln(2).unwrap();
        assert_eq!(rows(&mut win), ["aaa", "bbb", "ccc", "   ", "   "]);

        for n in [100, i32::MAX, -100, i32::MIN] {
            let mut win = five_lines(1);
            win.insdelln(n).unwrap();
            assert_eq!(
                rows(&mut win),
                ["aaa", "   ", "   ", "   ", "   "],
                "insdelln({n})"
            );
        }

        let mut win = five_lines(4);
        win.insdelln(0).unwrap();
        win.insertln().unwrap();
        assert_eq!(rows(&mut win), ["aaa", "bbb", "ccc", "ddd", "   "]);
    }

    #[test]
    fn test_insdelln_ignores_the_scroll_region() {
        let mut win = five_lines(1);
        win.setscrreg(2, 3).unwrap();
        win.mv(1, 0).unwrap();
        win.insdelln(-1).unwrap();
        assert_eq!(rows(&mut win), ["aaa", "ccc", "ddd", "eee", "   "]);
    }

    #[test]
    fn test_newline_on_the_last_line_of_the_scroll_region_scrolls_it() {
        let mut win = five_lines(0);
        win.scrollok(true);
        win.setscrreg(1, 2).unwrap();
        win.mv(2, 2).unwrap();
        win.addch('\n' as ChType).unwrap();
        // The rest of the line is cleared, the region scrolls, and the
        // cursor stays on the region's last line.
        assert_eq!(rows(&mut win), ["aaa", "cc ", "   ", "ddd", "eee"]);

        let mut win = five_lines(0);
        win.scrollok(true);
        win.setscrreg(1, 2).unwrap();
        win.mv(2, 2).unwrap();
        win.addch('\n' as ChType).unwrap();
        assert_eq!((win.getcury(), win.getcurx()), (2, 0));
    }

    #[test]
    fn test_wrapping_on_the_last_line_of_the_scroll_region_scrolls_it() {
        let mut win = five_lines(0);
        win.scrollok(true);
        win.setscrreg(1, 2).unwrap();
        win.mv(2, 2).unwrap();
        win.addstr("XY").unwrap();
        assert_eq!((win.getcury(), win.getcurx()), (2, 1));
        assert_eq!(rows(&mut win), ["aaa", "ccX", "Y  ", "ddd", "eee"]);
    }

    #[test]
    fn test_newline_outside_the_scroll_region_moves_down_without_scrolling() {
        let mut win = five_lines(0);
        win.scrollok(true);
        win.setscrreg(1, 2).unwrap();

        // Above the region.
        win.mv(0, 3 - 1).unwrap();
        win.addch('\n' as ChType).unwrap();
        assert_eq!((win.getcury(), win.getcurx()), (1, 0));

        // Below it: down to the last line of the window, and no further.
        win.mv(3, 2).unwrap();
        win.addch('\n' as ChType).unwrap();
        assert_eq!((win.getcury(), win.getcurx()), (4, 0));
        win.mv(4, 2).unwrap();
        win.addch('\n' as ChType).unwrap();
        assert_eq!((win.getcury(), win.getcurx()), (4, 2));
        assert_eq!(rows(&mut win), ["aa ", "bbb", "ccc", "dd ", "ee "]);
    }

    #[test]
    fn test_newline_at_the_scroll_region_bottom_without_scrollok_stays_put() {
        let mut win = five_lines(0);
        win.setscrreg(1, 2).unwrap();
        win.mv(2, 1).unwrap();
        win.addch('\n' as ChType).unwrap();
        assert_eq!((win.getcury(), win.getcurx()), (2, 1));
        win.mv(2, 2).unwrap();
        win.addstr("XY").unwrap();
        assert_eq!((win.getcury(), win.getcurx()), (2, 2));
        assert_eq!(rows(&mut win), ["aaa", "bbb", "c Y", "ddd", "eee"]);
    }

    #[test]
    fn test_newline_scrolls_a_window_with_the_default_scroll_region() {
        let mut win = Window::new(2, 3, 0, 0).unwrap();
        win.scrollok(true);
        win.addstr("ab\ncd\nef").unwrap();
        assert_eq!((win.getcury(), win.getcurx()), (1, 2));
        assert_eq!(rows(&mut win), ["cd ", "ef "]);
    }

    #[test]
    fn test_scrolling_stays_inside_the_scroll_region() {
        let mut win = five_lines(0);
        win.setscrreg(1, 3).unwrap();
        win.scroll_up(1).unwrap();
        assert_eq!(rows(&mut win), ["aaa", "ccc", "ddd", "   ", "eee"]);
        win.scroll_down(2).unwrap();
        assert_eq!(rows(&mut win), ["aaa", "   ", "   ", "ccc", "eee"]);
        win.scroll_down(i32::MAX).unwrap();
        assert_eq!(rows(&mut win), ["aaa", "   ", "   ", "   ", "eee"]);
    }

    #[test]
    fn test_scroll_up_shifts_lines() {
        let mut win = Window::new(3, 3, 0, 0).unwrap();
        for (y, text) in ["aaa", "bbb", "ccc"].into_iter().enumerate() {
            win.mvaddstr(y as i32, 0, text).unwrap();
        }
        win.scroll_up(1).unwrap();
        assert_eq!(rows(&mut win), ["bbb", "ccc", "   "]);
    }

    #[test]
    fn test_scroll_up_whole_region_clears_it() {
        let mut win = Window::new(3, 3, 0, 0).unwrap();
        for (y, text) in ["aaa", "bbb", "ccc"].into_iter().enumerate() {
            win.mvaddstr(y as i32, 0, text).unwrap();
        }
        // More than the region holds: clamped to the whole region.
        win.scroll_up(5).unwrap();
        assert_eq!(rows(&mut win), ["   ", "   ", "   "]);
    }

    #[test]
    fn test_scroll_up_one_line_window() {
        let mut win = Window::new(1, 3, 0, 0).unwrap();
        win.mvaddstr(0, 0, "aaa").unwrap();
        win.scroll_up(1).unwrap();
        assert_eq!(rows(&mut win), ["   "]);
    }
}
