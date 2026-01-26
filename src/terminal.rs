//! Terminal handling for ncurses-rs.
//!
//! This module provides low-level terminal I/O functionality using
//! POSIX termios for terminal configuration and control.
//!
//! # No-TTY Mode
//!
//! When the input/output file descriptors are not connected to a real TTY
//! (e.g., when running via non-interactive SSH or with redirected I/O),
//! the terminal operates in "no-TTY mode". In this mode:
//!
//! - Output operations (writing escape sequences) still work normally
//! - Terminal attribute changes (raw mode, cbreak, echo) are no-ops
//! - Input operations may return EOF or block indefinitely
//! - The [`Terminal::is_no_tty()`] method returns `true`
//!
//! This behavior matches how C ncurses handles non-TTY file descriptors,
//! allowing applications to work in pipelines or non-interactive contexts
//! where output is still useful even if interactive input is not available.
//!
//! # Safety
//!
//! This module contains unsafe code for interfacing with POSIX terminal APIs.
//! All unsafe blocks are carefully reviewed and documented with safety invariants.
//! The unsafe code is limited to:
//! - `libc::tcgetattr`/`libc::tcsetattr` for terminal attribute manipulation
//! - `libc::ioctl` for terminal size queries
//! - `libc::read`/`libc::write` for raw terminal I/O
//! - `libc::select` for input availability checking

use crate::error::{Error, Result};
use std::io;
use std::mem::MaybeUninit;
use std::os::unix::io::RawFd;

/// Terminal state flags for tracking initialization.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TermState {
    /// Terminal not initialized.
    #[default]
    Unknown,
    /// Terminal initialized but in shell mode.
    Initial,
    /// Terminal in program mode (curses active).
    Running,
    /// Terminal suspended (endwin called).
    Suspend,
}

/// Original terminal settings for restoration.
#[derive(Clone)]
pub struct TermSettings {
    /// Original termios settings.
    termios: libc::termios,
    /// Whether settings have been saved.
    saved: bool,
}

impl TermSettings {
    /// Create empty settings.
    pub fn new() -> Self {
        // SAFETY: `libc::termios` is a C struct that can be safely zero-initialized.
        // All fields are primitive types (integers and arrays of integers) that have
        // valid zero representations. The struct will be properly initialized by
        // `tcgetattr` before use.
        let termios = unsafe {
            let t = MaybeUninit::<libc::termios>::zeroed();
            t.assume_init()
        };
        Self {
            termios,
            saved: false,
        }
    }

    /// Save current terminal settings.
    ///
    /// Returns `Ok(true)` if settings were successfully saved, `Ok(false)` if the
    /// file descriptor is not a TTY (ENOTTY), or an error for other failures.
    pub fn save(&mut self, fd: RawFd) -> Result<bool> {
        // SAFETY: `tcgetattr` is a POSIX function that reads terminal attributes.
        // - `fd` is a valid file descriptor (checked by the caller)
        // - `&mut self.termios` is a valid pointer to a `libc::termios` struct
        // - The function will fully initialize the termios struct on success
        let result = unsafe { libc::tcgetattr(fd, &mut self.termios) };
        if result == 0 {
            self.saved = true;
            Ok(true)
        } else {
            let errno = io::Error::last_os_error().raw_os_error().unwrap_or(-1);
            if errno == libc::ENOTTY {
                // Not a TTY - that's okay, we just can't save/restore settings
                self.saved = false;
                Ok(false)
            } else {
                Err(Error::SystemError(errno))
            }
        }
    }

    /// Restore saved terminal settings.
    ///
    /// If `no_tty` is true or no settings were saved, this is a no-op.
    /// Also gracefully handles ENOTTY errors (returns Ok).
    pub fn restore(&self, fd: RawFd, no_tty: bool) -> Result<()> {
        if !self.saved || no_tty {
            return Ok(());
        }
        // SAFETY: `tcsetattr` is a POSIX function that sets terminal attributes.
        // - `fd` is a valid file descriptor (checked by the caller)
        // - `&self.termios` points to a valid, initialized `libc::termios` struct
        //   (guaranteed by `self.saved == true`, which is only set after successful `tcgetattr`)
        // - `TCSANOW` is a valid action flag
        let result = unsafe { libc::tcsetattr(fd, libc::TCSANOW, &self.termios) };
        if result == 0 {
            Ok(())
        } else {
            let errno = io::Error::last_os_error().raw_os_error().unwrap_or(-1);
            if errno == libc::ENOTTY {
                // TTY went away - that's okay
                Ok(())
            } else {
                Err(Error::SystemError(errno))
            }
        }
    }

    /// Check if settings have been saved.
    pub fn is_saved(&self) -> bool {
        self.saved
    }
}

impl Default for TermSettings {
    fn default() -> Self {
        Self::new()
    }
}

/// Low-level terminal interface.
///
/// This struct provides the core terminal I/O functionality. It can operate
/// in two modes:
///
/// - **Normal mode**: When connected to a real TTY, full terminal control
///   is available including raw mode, echo control, and reliable input.
///
/// - **No-TTY mode**: When not connected to a TTY (e.g., via non-interactive
///   SSH or pipes), terminal attribute changes are no-ops but output still
///   works. Check [`is_no_tty()`](Self::is_no_tty) to detect this mode.
pub struct Terminal {
    /// Input file descriptor.
    input_fd: RawFd,
    /// Output file descriptor.
    output_fd: RawFd,
    /// Whether the terminal is operating without a real TTY.
    ///
    /// When true, `tcsetattr`/`tcgetattr` operations are skipped (they would
    /// fail with ENOTTY anyway) and input operations may be limited.
    /// Output via escape sequences still works normally.
    no_tty: bool,
    /// Current terminal state.
    state: TermState,
    /// Original (shell) terminal settings.
    shell_settings: TermSettings,
    /// Program terminal settings.
    prog_settings: TermSettings,
    /// Current termios settings.
    current: libc::termios,
    /// Terminal name/type.
    term_type: String,
    /// Number of lines.
    lines: i32,
    /// Number of columns.
    columns: i32,
    /// Number of colors.
    colors: i32,
    /// Number of color pairs.
    color_pairs: i32,
    /// Whether terminal can change colors.
    can_change_color: bool,
    /// Output buffer for batching writes.
    output_buffer: Vec<u8>,
    /// File descriptor for typeahead checking (-1 to disable).
    typeahead_fd: i32,
    /// Whether terminal has insert/delete character capability.
    has_ic: bool,
    /// Whether terminal has insert/delete line capability.
    has_il: bool,
}

impl Terminal {
    /// Create a new terminal with the given file descriptors.
    pub fn new(input_fd: RawFd, output_fd: RawFd) -> Result<Self> {
        Self::new_internal(input_fd, output_fd)
    }

    /// Internal constructor.
    fn new_internal(input_fd: RawFd, output_fd: RawFd) -> Result<Self> {
        // SAFETY: `libc::termios` is a C struct that can be safely zero-initialized.
        // All fields are primitive types that have valid zero representations.
        // The struct will be properly initialized by `tcgetattr` below.
        let current = unsafe {
            let t = MaybeUninit::<libc::termios>::zeroed();
            t.assume_init()
        };

        let mut term = Self {
            input_fd,
            output_fd,
            no_tty: false,
            state: TermState::Unknown,
            shell_settings: TermSettings::new(),
            prog_settings: TermSettings::new(),
            current,
            term_type: String::new(),
            lines: 24,
            columns: 80,
            colors: 8,
            color_pairs: 64,
            can_change_color: false,
            output_buffer: Vec::with_capacity(4096),
            typeahead_fd: libc::STDIN_FILENO,
            has_ic: true, // Will be updated in detect_terminal
            has_il: true, // Will be updated in detect_terminal
        };

        // SAFETY: `tcgetattr` is a POSIX function that reads terminal attributes.
        // - `input_fd` is provided by the caller and expected to be a valid terminal fd
        // - `&mut term.current` is a valid pointer to a `libc::termios` struct
        // - On success, the entire termios struct is initialized
        // - On ENOTTY, we enter no-TTY mode instead of failing
        let result = unsafe { libc::tcgetattr(input_fd, &mut term.current) };
        if result != 0 {
            let errno = io::Error::last_os_error().raw_os_error().unwrap_or(-1);
            if errno == libc::ENOTTY {
                // Not a TTY - enter no-TTY mode
                term.no_tty = true;
                term.typeahead_fd = -1; // Disable typeahead checking
            } else {
                return Err(Error::SystemError(errno));
            }
        }

        // Save shell settings (will return Ok(false) in no-TTY mode)
        term.shell_settings.save(input_fd)?;

        // Detect terminal type
        term.detect_terminal()?;

        // Get terminal size
        term.update_size()?;

        term.state = TermState::Initial;
        Ok(term)
    }

    /// Create a terminal using stdin/stdout.
    ///
    /// If stdin/stdout is not a TTY (e.g., when input is redirected), the terminal
    /// will operate in no-TTY mode where output still works but terminal attribute
    /// changes are no-ops.
    pub fn from_stdio() -> Result<Self> {
        Self::new(libc::STDIN_FILENO, libc::STDOUT_FILENO)
    }

    /// Detect terminal type and capabilities.
    fn detect_terminal(&mut self) -> Result<()> {
        // Get TERM environment variable
        self.term_type = std::env::var("TERM").unwrap_or_else(|_| "dumb".to_string());

        // Set capabilities based on terminal type
        // Start with defaults
        self.colors = 8;
        self.color_pairs = 64;
        self.can_change_color = false;

        match self.term_type.as_str() {
            // xterm and variants - most common
            "xterm" | "xterm-color" => {
                self.colors = 8;
                self.color_pairs = 64;
                self.can_change_color = true;
            }
            "xterm-256color" | "xterm-direct" => {
                self.colors = 256;
                self.color_pairs = 32767;
                self.can_change_color = true;
            }
            "xterm-16color" => {
                self.colors = 16;
                self.color_pairs = 256;
                self.can_change_color = true;
            }
            "xterm-88color" => {
                self.colors = 88;
                self.color_pairs = 256;
                self.can_change_color = true;
            }

            // GNU Screen
            "screen" | "screen.xterm" => {
                self.colors = 8;
                self.color_pairs = 64;
                self.can_change_color = false;
            }
            "screen-256color" | "screen.xterm-256color" => {
                self.colors = 256;
                self.color_pairs = 32767;
                self.can_change_color = false;
            }

            // tmux
            "tmux" => {
                self.colors = 8;
                self.color_pairs = 64;
                self.can_change_color = false;
            }
            "tmux-256color" | "tmux-direct" => {
                self.colors = 256;
                self.color_pairs = 32767;
                self.can_change_color = false;
            }

            // Linux console
            "linux" | "linux-16color" => {
                self.colors = 8;
                self.color_pairs = 64;
                self.can_change_color = true;
            }

            // VT100/VT220 family - no color support
            "vt100" | "vt100-am" | "vt100-nav" => {
                self.colors = 0;
                self.color_pairs = 0;
                self.can_change_color = false;
            }
            "vt220" | "vt220-8bit" | "vt320" | "vt420" => {
                self.colors = 0;
                self.color_pairs = 0;
                self.can_change_color = false;
            }

            // ANSI terminals
            "ansi" | "ansi-m" | "ansi.sys" => {
                self.colors = 8;
                self.color_pairs = 64;
                self.can_change_color = false;
            }

            // rxvt family
            "rxvt" | "rxvt-color" => {
                self.colors = 8;
                self.color_pairs = 64;
                self.can_change_color = true;
            }
            "rxvt-256color" | "rxvt-unicode-256color" => {
                self.colors = 256;
                self.color_pairs = 32767;
                self.can_change_color = true;
            }
            "rxvt-unicode" => {
                self.colors = 88;
                self.color_pairs = 256;
                self.can_change_color = true;
            }

            // Konsole (KDE terminal)
            "konsole" | "konsole-256color" => {
                self.colors = 256;
                self.color_pairs = 32767;
                self.can_change_color = true;
            }

            // GNOME Terminal / VTE-based terminals
            "gnome" | "gnome-256color" | "vte" | "vte-256color" => {
                self.colors = 256;
                self.color_pairs = 32767;
                self.can_change_color = true;
            }

            // Alacritty
            "alacritty" | "alacritty-direct" => {
                self.colors = 256;
                self.color_pairs = 32767;
                self.can_change_color = true;
            }

            // kitty
            "xterm-kitty" | "kitty" => {
                self.colors = 256;
                self.color_pairs = 32767;
                self.can_change_color = true;
            }

            // iTerm2
            "iterm2" | "iTerm2.app" | "iTerm.app" => {
                self.colors = 256;
                self.color_pairs = 32767;
                self.can_change_color = true;
            }

            // Apple Terminal
            "nsterm" | "Apple_Terminal" => {
                self.colors = 256;
                self.color_pairs = 32767;
                self.can_change_color = false;
            }

            // Windows Terminal / mintty
            "mintty" | "mintty-direct" => {
                self.colors = 256;
                self.color_pairs = 32767;
                self.can_change_color = true;
            }

            // PuTTY
            "putty" | "putty-256color" => {
                self.colors = 256;
                self.color_pairs = 32767;
                self.can_change_color = false;
            }

            // Emacs terminals
            "eterm" | "eterm-color" => {
                self.colors = 8;
                self.color_pairs = 64;
                self.can_change_color = false;
            }

            // foot terminal
            "foot" | "foot-direct" => {
                self.colors = 256;
                self.color_pairs = 32767;
                self.can_change_color = true;
            }

            // wezterm
            "wezterm" => {
                self.colors = 256;
                self.color_pairs = 32767;
                self.can_change_color = true;
            }

            // st (simple terminal)
            "st" | "st-256color" => {
                self.colors = 256;
                self.color_pairs = 32767;
                self.can_change_color = true;
            }

            // contour terminal
            "contour" | "contour-direct" => {
                self.colors = 256;
                self.color_pairs = 32767;
                self.can_change_color = true;
            }

            // dumb terminal - no capabilities
            "dumb" | "unknown" | "" => {
                self.colors = 0;
                self.color_pairs = 0;
                self.can_change_color = false;
            }

            // Default fallback - check for common suffixes
            _ => {
                // Check for 256color suffix
                if self.term_type.ends_with("-256color")
                    || self.term_type.ends_with(".256color")
                    || self.term_type.contains("256")
                {
                    self.colors = 256;
                    self.color_pairs = 32767;
                    self.can_change_color = false;
                }
                // Check for direct color suffix
                else if self.term_type.ends_with("-direct") {
                    self.colors = 256;
                    self.color_pairs = 32767;
                    self.can_change_color = true;
                }
                // Check for 16color suffix
                else if self.term_type.ends_with("-16color") {
                    self.colors = 16;
                    self.color_pairs = 256;
                    self.can_change_color = false;
                }
                // Assume basic 8-color support for unknown terminals
                else {
                    self.colors = 8;
                    self.color_pairs = 64;
                    self.can_change_color = false;
                }
            }
        }

        // Check COLORTERM for true color support override
        if let Ok(colorterm) = std::env::var("COLORTERM") {
            match colorterm.as_str() {
                "truecolor" | "24bit" => {
                    self.colors = 16777216; // 24-bit color
                    self.can_change_color = true;
                }
                "256" => {
                    if self.colors < 256 {
                        self.colors = 256;
                        self.color_pairs = 32767;
                    }
                }
                _ => {}
            }
        }

        // Check for TERM_PROGRAM to identify specific terminal emulators
        if let Ok(term_program) = std::env::var("TERM_PROGRAM") {
            match term_program.as_str() {
                "iTerm.app" | "iTerm2.app" => {
                    // iTerm2 supports true color
                    if self.colors < 256 {
                        self.colors = 256;
                        self.color_pairs = 32767;
                    }
                    self.can_change_color = true;
                }
                "Apple_Terminal" => {
                    // macOS Terminal supports 256 colors but not true color modification
                    if self.colors < 256 {
                        self.colors = 256;
                        self.color_pairs = 32767;
                    }
                }
                "vscode" | "VSCode" => {
                    // VS Code integrated terminal supports true color
                    if self.colors < 256 {
                        self.colors = 256;
                        self.color_pairs = 32767;
                    }
                    self.can_change_color = true;
                }
                "Hyper" => {
                    // Hyper terminal supports true color
                    if self.colors < 256 {
                        self.colors = 256;
                        self.color_pairs = 32767;
                    }
                    self.can_change_color = true;
                }
                "WezTerm" => {
                    // WezTerm supports true color
                    if self.colors < 256 {
                        self.colors = 256;
                        self.color_pairs = 32767;
                    }
                    self.can_change_color = true;
                }
                _ => {}
            }
        }

        // Check VTE_VERSION for VTE-based terminals (GNOME Terminal, etc.)
        if std::env::var("VTE_VERSION").is_ok() {
            // VTE-based terminals typically support 256 colors
            if self.colors < 256 {
                self.colors = 256;
                self.color_pairs = 32767;
            }
            self.can_change_color = true;
        }

        // Check for KONSOLE_VERSION
        if std::env::var("KONSOLE_VERSION").is_ok() {
            if self.colors < 256 {
                self.colors = 256;
                self.color_pairs = 32767;
            }
            self.can_change_color = true;
        }

        // Check for KITTY_WINDOW_ID
        if std::env::var("KITTY_WINDOW_ID").is_ok() {
            self.colors = 16777216; // kitty supports true color
            self.color_pairs = 32767;
            self.can_change_color = true;
        }

        // Check for WT_SESSION (Windows Terminal)
        if std::env::var("WT_SESSION").is_ok() {
            self.colors = 16777216; // Windows Terminal supports true color
            self.color_pairs = 32767;
            self.can_change_color = true;
        }

        // Check for ALACRITTY_WINDOW_ID
        if std::env::var("ALACRITTY_WINDOW_ID").is_ok() {
            self.colors = 16777216; // Alacritty supports true color
            self.color_pairs = 32767;
            self.can_change_color = true;
        }

        // Detect insert/delete character and line capabilities
        // Most modern terminals support these, but some basic/dumb terminals don't
        match self.term_type.as_str() {
            // Dumb/basic terminals with no capabilities
            "dumb" | "unknown" | "" => {
                self.has_ic = false;
                self.has_il = false;
            }
            // VT100 has no insert/delete character capability
            "vt100" | "vt100-am" | "vt100-nav" => {
                self.has_ic = false;
                self.has_il = true; // VT100 can scroll
            }
            // Hardcopy terminals
            "hardcopy" | "lpr" | "printer" => {
                self.has_ic = false;
                self.has_il = false;
            }
            // All modern terminals support both
            _ => {
                self.has_ic = true;
                self.has_il = true;
            }
        }

        Ok(())
    }

    /// Update terminal size from the system.
    pub fn update_size(&mut self) -> Result<()> {
        // SAFETY: `libc::winsize` is a C struct that can be safely zero-initialized.
        // All fields are primitive integer types with valid zero representations.
        // The struct will be initialized by the `ioctl` call below.
        let mut ws = unsafe {
            let w = MaybeUninit::<libc::winsize>::zeroed();
            w.assume_init()
        };

        // SAFETY: `ioctl` with `TIOCGWINSZ` reads the terminal window size.
        // - `self.output_fd` is a valid file descriptor (validated in `new()`)
        // - `&mut ws` is a valid pointer to a `libc::winsize` struct
        // - `TIOCGWINSZ` is a valid ioctl request for getting window size
        // - On success, ws.ws_row and ws.ws_col contain the terminal dimensions
        let result = unsafe { libc::ioctl(self.output_fd, libc::TIOCGWINSZ, &mut ws) };

        if result == 0 && ws.ws_row > 0 && ws.ws_col > 0 {
            self.lines = ws.ws_row as i32;
            self.columns = ws.ws_col as i32;
        } else {
            // Try environment variables
            if let Ok(lines) = std::env::var("LINES") {
                if let Ok(n) = lines.parse() {
                    self.lines = n;
                }
            }
            if let Ok(cols) = std::env::var("COLUMNS") {
                if let Ok(n) = cols.parse() {
                    self.columns = n;
                }
            }
        }

        Ok(())
    }

    /// Enter program mode (raw/cbreak).
    pub fn enter_program_mode(&mut self) -> Result<()> {
        // Save current as program settings
        self.prog_settings.termios = self.current;
        self.prog_settings.saved = !self.no_tty;

        if !self.no_tty {
            // Set up raw-ish mode
            let mut new_settings = self.current;

            // Disable canonical mode and echo
            new_settings.c_lflag &= !(libc::ICANON | libc::ECHO | libc::ISIG);

            // Disable input processing
            new_settings.c_iflag &= !(libc::ICRNL | libc::INLCR | libc::IXON);

            // Disable output processing
            new_settings.c_oflag &= !libc::OPOST;

            // Set minimum characters and timeout for read
            new_settings.c_cc[libc::VMIN] = 1;
            new_settings.c_cc[libc::VTIME] = 0;

            // SAFETY: `tcsetattr` is a POSIX function that sets terminal attributes.
            // - `self.input_fd` is a valid file descriptor (validated in `new()`)
            // - `&new_settings` points to a valid, initialized `libc::termios` struct
            //   (copied from `self.current` which was initialized by `tcgetattr`)
            // - `TCSANOW` applies changes immediately
            let result = unsafe { libc::tcsetattr(self.input_fd, libc::TCSANOW, &new_settings) };
            if result != 0 {
                return Err(Error::SystemError(
                    io::Error::last_os_error().raw_os_error().unwrap_or(-1),
                ));
            }

            self.current = new_settings;
        }

        self.state = TermState::Running;

        // Enter alternate screen buffer
        self.write_escape("\x1b[?1049h")?;

        // Hide cursor initially
        self.write_escape("\x1b[?25l")?;

        Ok(())
    }

    /// Leave program mode (restore terminal).
    pub fn leave_program_mode(&mut self) -> Result<()> {
        // Show cursor
        self.write_escape("\x1b[?25h")?;

        // Leave alternate screen buffer
        self.write_escape("\x1b[?1049l")?;

        // Flush output
        self.flush()?;

        // Restore shell settings (no-op in no-TTY mode)
        self.shell_settings.restore(self.input_fd, self.no_tty)?;
        if !self.no_tty {
            self.current = self.shell_settings.termios;
        }
        self.state = TermState::Suspend;

        Ok(())
    }

    /// Set raw mode.
    pub fn raw(&mut self, enable: bool) -> Result<()> {
        if self.no_tty {
            return Ok(());
        }

        let mut new_settings = self.current;

        if enable {
            new_settings.c_lflag &= !(libc::ICANON | libc::ECHO | libc::ISIG | libc::IEXTEN);
            new_settings.c_iflag &= !(libc::IXON | libc::ICRNL);
        } else {
            // Restore canonical mode and signals
            new_settings.c_lflag |= libc::ICANON | libc::ISIG | libc::IEXTEN;
            new_settings.c_iflag |= libc::IXON | libc::ICRNL;
        }

        // SAFETY: `tcsetattr` is a POSIX function that sets terminal attributes.
        // - `self.input_fd` is a valid file descriptor (validated in `new()`)
        // - `&new_settings` points to a valid `libc::termios` struct (copied from `self.current`)
        // - `TCSANOW` applies changes immediately
        let result = unsafe { libc::tcsetattr(self.input_fd, libc::TCSANOW, &new_settings) };
        if result == 0 {
            self.current = new_settings;
            Ok(())
        } else {
            Err(Error::SystemError(
                io::Error::last_os_error().raw_os_error().unwrap_or(-1),
            ))
        }
    }

    /// Set cbreak mode.
    pub fn cbreak(&mut self, enable: bool) -> Result<()> {
        if self.no_tty {
            return Ok(());
        }

        let mut new_settings = self.current;

        if enable {
            new_settings.c_lflag &= !libc::ICANON;
            new_settings.c_lflag |= libc::ISIG; // Keep signals
            new_settings.c_cc[libc::VMIN] = 1;
            new_settings.c_cc[libc::VTIME] = 0;
        } else {
            new_settings.c_lflag |= libc::ICANON;
        }

        // SAFETY: `tcsetattr` is a POSIX function that sets terminal attributes.
        // - `self.input_fd` is a valid file descriptor (validated in `new()`)
        // - `&new_settings` points to a valid `libc::termios` struct (copied from `self.current`)
        // - `TCSANOW` applies changes immediately
        let result = unsafe { libc::tcsetattr(self.input_fd, libc::TCSANOW, &new_settings) };
        if result == 0 {
            self.current = new_settings;
            Ok(())
        } else {
            Err(Error::SystemError(
                io::Error::last_os_error().raw_os_error().unwrap_or(-1),
            ))
        }
    }

    /// Set echo mode.
    pub fn echo(&mut self, enable: bool) -> Result<()> {
        if self.no_tty {
            return Ok(());
        }

        let mut new_settings = self.current;

        if enable {
            new_settings.c_lflag |= libc::ECHO;
        } else {
            new_settings.c_lflag &= !libc::ECHO;
        }

        // SAFETY: `tcsetattr` is a POSIX function that sets terminal attributes.
        // - `self.input_fd` is a valid file descriptor (validated in `new()`)
        // - `&new_settings` points to a valid `libc::termios` struct (copied from `self.current`)
        // - `TCSANOW` applies changes immediately
        let result = unsafe { libc::tcsetattr(self.input_fd, libc::TCSANOW, &new_settings) };
        if result == 0 {
            self.current = new_settings;
            Ok(())
        } else {
            Err(Error::SystemError(
                io::Error::last_os_error().raw_os_error().unwrap_or(-1),
            ))
        }
    }

    /// Write an escape sequence.
    fn write_escape(&mut self, seq: &str) -> Result<()> {
        self.output_buffer.extend_from_slice(seq.as_bytes());
        Ok(())
    }

    /// Write bytes to the output buffer.
    pub fn write(&mut self, data: &[u8]) -> Result<()> {
        self.output_buffer.extend_from_slice(data);
        Ok(())
    }

    /// Write a string to the output buffer.
    pub fn write_str(&mut self, s: &str) -> Result<()> {
        self.output_buffer.extend_from_slice(s.as_bytes());
        Ok(())
    }

    /// Flush the output buffer to the terminal.
    pub fn flush(&mut self) -> Result<()> {
        if self.output_buffer.is_empty() {
            return Ok(());
        }

        // SAFETY: `libc::write` writes data to a file descriptor.
        // - `self.output_fd` is a valid file descriptor (validated in `new()`)
        // - `self.output_buffer.as_ptr()` returns a valid pointer to the buffer's data
        // - `self.output_buffer.len()` is the exact number of bytes to write
        // - The buffer remains valid and unchanged during the write call
        let result = unsafe {
            libc::write(
                self.output_fd,
                self.output_buffer.as_ptr() as *const libc::c_void,
                self.output_buffer.len(),
            )
        };

        self.output_buffer.clear();

        if result < 0 {
            Err(Error::SystemError(
                io::Error::last_os_error().raw_os_error().unwrap_or(-1),
            ))
        } else {
            Ok(())
        }
    }

    /// Read a single byte from the terminal.
    pub fn read_byte(&self) -> Result<Option<u8>> {
        let mut buf = [0u8; 1];
        // SAFETY: `libc::read` reads data from a file descriptor.
        // - `self.input_fd` is a valid file descriptor (validated in `new()`)
        // - `buf.as_mut_ptr()` returns a valid pointer to a 1-byte buffer
        // - The size argument (1) matches the buffer size exactly
        // - The buffer is stack-allocated and remains valid during the read call
        let result = unsafe { libc::read(self.input_fd, buf.as_mut_ptr() as *mut libc::c_void, 1) };

        if result > 0 {
            Ok(Some(buf[0]))
        } else if result == 0 {
            Ok(None) // EOF
        } else {
            let err = io::Error::last_os_error();
            if err.kind() == io::ErrorKind::WouldBlock {
                Ok(None)
            } else {
                Err(Error::SystemError(err.raw_os_error().unwrap_or(-1)))
            }
        }
    }

    /// Check if input is available.
    pub fn has_input(&self) -> bool {
        // SAFETY: This unsafe block uses `select` to check for available input.
        // - `libc::fd_set` is zero-initialized, which is the correct initial state
        // - `FD_ZERO` clears the set (redundant but safe)
        // - `FD_SET` adds our file descriptor to the set
        // - `self.input_fd` is a valid file descriptor (validated in `new()`)
        // - `select` is called with a zero timeout for non-blocking check
        // - All pointers passed to `select` are valid stack-allocated variables
        unsafe {
            let mut fds = {
                let f = MaybeUninit::<libc::fd_set>::zeroed();
                f.assume_init()
            };
            libc::FD_ZERO(&mut fds);
            libc::FD_SET(self.input_fd, &mut fds);

            let mut timeout = libc::timeval {
                tv_sec: 0,
                tv_usec: 0,
            };

            let result = libc::select(
                self.input_fd + 1,
                &mut fds,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &mut timeout,
            );

            result > 0
        }
    }

    // ========================================================================
    // Terminal output operations
    // ========================================================================

    /// Move cursor to position.
    pub fn move_cursor(&mut self, y: i32, x: i32) -> Result<()> {
        // ANSI cursor positioning (1-based)
        let seq = format!("\x1b[{};{}H", y + 1, x + 1);
        self.write_str(&seq)
    }

    /// Clear the entire screen.
    pub fn clear_screen(&mut self) -> Result<()> {
        self.write_escape("\x1b[2J")
    }

    /// Clear to end of line.
    pub fn clear_to_eol(&mut self) -> Result<()> {
        self.write_escape("\x1b[K")
    }

    /// Clear to end of screen.
    pub fn clear_to_eos(&mut self) -> Result<()> {
        self.write_escape("\x1b[J")
    }

    /// Set cursor visibility.
    pub fn cursor_visible(&mut self, visible: bool) -> Result<()> {
        if visible {
            self.write_escape("\x1b[?25h")
        } else {
            self.write_escape("\x1b[?25l")
        }
    }

    /// Set text attributes.
    pub fn set_attributes(&mut self, attr: crate::types::AttrT) -> Result<()> {
        use crate::attr::*;

        let mut codes = vec![0u8]; // Reset

        if attr & A_BOLD != 0 {
            codes.push(1);
        }
        if attr & A_DIM != 0 {
            codes.push(2);
        }
        if attr & A_ITALIC != 0 {
            codes.push(3);
        }
        if attr & A_UNDERLINE != 0 {
            codes.push(4);
        }
        if attr & A_BLINK != 0 {
            codes.push(5);
        }
        if attr & A_REVERSE != 0 {
            codes.push(7);
        }
        if attr & A_INVIS != 0 {
            codes.push(8);
        }

        let code_str: String = codes
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join(";");
        let seq = format!("\x1b[{}m", code_str);
        self.write_str(&seq)
    }

    /// Set foreground color.
    pub fn set_fg_color(&mut self, color: i16) -> Result<()> {
        if color < 0 {
            // Default color
            self.write_escape("\x1b[39m")
        } else if color < 8 {
            let seq = format!("\x1b[{}m", 30 + color);
            self.write_str(&seq)
        } else if color < 16 {
            let seq = format!("\x1b[{}m", 90 + color - 8);
            self.write_str(&seq)
        } else {
            let seq = format!("\x1b[38;5;{}m", color);
            self.write_str(&seq)
        }
    }

    /// Set background color.
    pub fn set_bg_color(&mut self, color: i16) -> Result<()> {
        if color < 0 {
            // Default color
            self.write_escape("\x1b[49m")
        } else if color < 8 {
            let seq = format!("\x1b[{}m", 40 + color);
            self.write_str(&seq)
        } else if color < 16 {
            let seq = format!("\x1b[{}m", 100 + color - 8);
            self.write_str(&seq)
        } else {
            let seq = format!("\x1b[48;5;{}m", color);
            self.write_str(&seq)
        }
    }

    /// Ring the terminal bell.
    pub fn beep(&mut self) -> Result<()> {
        self.write_escape("\x07")
    }

    /// Flash the screen (visible bell).
    pub fn flash(&mut self) -> Result<()> {
        // Reverse video, wait, then restore
        self.write_escape("\x1b[?5h")?;
        self.flush()?;
        std::thread::sleep(std::time::Duration::from_millis(100));
        self.write_escape("\x1b[?5l")
    }

    // ========================================================================
    // Terminal mode save/restore
    // ========================================================================

    /// Save the current terminal settings as program mode.
    pub fn save_prog_mode(&mut self) -> Result<()> {
        self.prog_settings.save(self.input_fd)?;
        Ok(())
    }

    /// Save the current terminal settings as shell mode.
    pub fn save_shell_mode(&mut self) -> Result<()> {
        self.shell_settings.save(self.input_fd)?;
        Ok(())
    }

    /// Restore the saved program terminal settings.
    pub fn restore_prog_mode(&mut self) -> Result<()> {
        if self.prog_settings.is_saved() {
            // SAFETY: `tcsetattr` is a POSIX function that sets terminal attributes.
            // - `self.input_fd` is a valid file descriptor (validated in `new()`)
            // - `&self.prog_settings.termios` points to a valid, initialized struct
            //   (guaranteed by `is_saved() == true`)
            // - `TCSANOW` applies changes immediately
            let result = unsafe {
                libc::tcsetattr(self.input_fd, libc::TCSANOW, &self.prog_settings.termios)
            };
            if result == 0 {
                self.current = self.prog_settings.termios;
                self.state = TermState::Running;
                Ok(())
            } else {
                Err(Error::SystemError(
                    io::Error::last_os_error().raw_os_error().unwrap_or(-1),
                ))
            }
        } else {
            Ok(())
        }
    }

    /// Restore the saved shell terminal settings.
    pub fn restore_shell_mode(&mut self) -> Result<()> {
        if self.shell_settings.is_saved() {
            // SAFETY: `tcsetattr` is a POSIX function that sets terminal attributes.
            // - `self.input_fd` is a valid file descriptor (validated in `new()`)
            // - `&self.shell_settings.termios` points to a valid, initialized struct
            //   (guaranteed by `is_saved() == true`)
            // - `TCSANOW` applies changes immediately
            let result = unsafe {
                libc::tcsetattr(self.input_fd, libc::TCSANOW, &self.shell_settings.termios)
            };
            if result == 0 {
                self.current = self.shell_settings.termios;
                self.state = TermState::Suspend;
                Ok(())
            } else {
                Err(Error::SystemError(
                    io::Error::last_os_error().raw_os_error().unwrap_or(-1),
                ))
            }
        } else {
            Ok(())
        }
    }

    // ========================================================================
    // Getters
    // ========================================================================

    /// Get the terminal state.
    pub fn state(&self) -> TermState {
        self.state
    }

    /// Get the terminal type.
    pub fn term_type(&self) -> &str {
        &self.term_type
    }

    /// Get the number of lines.
    pub fn lines(&self) -> i32 {
        self.lines
    }

    /// Get the number of columns.
    pub fn columns(&self) -> i32 {
        self.columns
    }

    /// Get the number of colors.
    pub fn colors(&self) -> i32 {
        self.colors
    }

    /// Get the number of color pairs.
    pub fn color_pairs(&self) -> i32 {
        self.color_pairs
    }

    /// Check if colors can be changed.
    pub fn can_change_color(&self) -> bool {
        self.can_change_color
    }

    /// Get the input file descriptor.
    pub fn input_fd(&self) -> RawFd {
        self.input_fd
    }

    /// Get the output file descriptor.
    pub fn output_fd(&self) -> RawFd {
        self.output_fd
    }

    /// Returns true if operating in no-TTY mode.
    ///
    /// In no-TTY mode, terminal attribute changes are no-ops but output
    /// (escape sequences) still works.
    pub fn is_no_tty(&self) -> bool {
        self.no_tty
    }

    // ========================================================================
    // Additional terminal control functions
    // ========================================================================

    /// Control flushing of input queue on interrupt.
    ///
    /// When enabled, if an interrupt key is pressed, any pending input will be flushed.
    pub fn intrflush(&mut self, enable: bool) -> Result<()> {
        let mut new_settings = self.current;

        if enable {
            // On most systems, this is controlled by NOFLSH flag (inverted logic)
            new_settings.c_lflag &= !libc::NOFLSH;
        } else {
            new_settings.c_lflag |= libc::NOFLSH;
        }

        // SAFETY: `tcsetattr` is a POSIX function that sets terminal attributes.
        // - `self.input_fd` is a valid file descriptor (validated in `new()`)
        // - `&new_settings` points to a valid `libc::termios` struct (copied from `self.current`)
        // - `TCSANOW` applies changes immediately
        let result = unsafe { libc::tcsetattr(self.input_fd, libc::TCSANOW, &new_settings) };
        if result == 0 {
            self.current = new_settings;
            Ok(())
        } else {
            Err(Error::SystemError(
                io::Error::last_os_error().raw_os_error().unwrap_or(-1),
            ))
        }
    }

    /// Enable or disable 8-bit input mode.
    ///
    /// When enabled, the terminal passes 8-bit characters through without
    /// stripping the high bit.
    pub fn meta(&mut self, enable: bool) -> Result<()> {
        let mut new_settings = self.current;

        if enable {
            // Clear ISTRIP to allow 8-bit input
            new_settings.c_iflag &= !libc::ISTRIP;
            // Set CS8 for 8-bit characters
            new_settings.c_cflag |= libc::CS8;
        } else {
            // Set ISTRIP to strip to 7 bits
            new_settings.c_iflag |= libc::ISTRIP;
        }

        // SAFETY: `tcsetattr` is a POSIX function that sets terminal attributes.
        // - `self.input_fd` is a valid file descriptor (validated in `new()`)
        // - `&new_settings` points to a valid `libc::termios` struct (copied from `self.current`)
        // - `TCSANOW` applies changes immediately
        let result = unsafe { libc::tcsetattr(self.input_fd, libc::TCSANOW, &new_settings) };
        if result == 0 {
            self.current = new_settings;
            Ok(())
        } else {
            Err(Error::SystemError(
                io::Error::last_os_error().raw_os_error().unwrap_or(-1),
            ))
        }
    }

    /// Set the file descriptor used for typeahead checking.
    ///
    /// Pass -1 to disable typeahead checking entirely.
    pub fn set_typeahead_fd(&mut self, fd: i32) {
        self.typeahead_fd = fd;
    }

    /// Get the file descriptor used for typeahead checking.
    pub fn typeahead_fd(&self) -> i32 {
        self.typeahead_fd
    }

    /// Control flushing of input and output queues on interrupt.
    ///
    /// When enabled (the default), pressing interrupt/quit/suspend keys
    /// will flush the queues. When disabled, queues are not flushed.
    pub fn qiflush(&mut self, enable: bool) -> Result<()> {
        let mut new_settings = self.current;

        if enable {
            // Clear NOFLSH to enable flushing
            new_settings.c_lflag &= !libc::NOFLSH;
        } else {
            // Set NOFLSH to disable flushing
            new_settings.c_lflag |= libc::NOFLSH;
        }

        // SAFETY: `tcsetattr` is a POSIX function that sets terminal attributes.
        // - `self.input_fd` is a valid file descriptor (validated in `new()`)
        // - `&new_settings` points to a valid `libc::termios` struct (copied from `self.current`)
        // - `TCSANOW` applies changes immediately
        let result = unsafe { libc::tcsetattr(self.input_fd, libc::TCSANOW, &new_settings) };
        if result == 0 {
            self.current = new_settings;
            Ok(())
        } else {
            Err(Error::SystemError(
                io::Error::last_os_error().raw_os_error().unwrap_or(-1),
            ))
        }
    }

    /// Check if the terminal has insert/delete character capability.
    ///
    /// Returns true if the terminal supports inserting and deleting characters
    /// (ich/dch capabilities in terminfo).
    pub fn has_ic(&self) -> bool {
        self.has_ic
    }

    /// Check if the terminal has insert/delete line capability.
    ///
    /// Returns true if the terminal supports inserting and deleting lines
    /// (il/dl capabilities in terminfo).
    pub fn has_il(&self) -> bool {
        self.has_il
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        // Try to restore terminal state
        if self.state == TermState::Running {
            let _ = self.leave_program_mode();
        }
    }
}

/// Get the baud rate of the terminal.
///
/// This uses cfgetospeed() to get the actual output speed from the terminal.
/// Returns the baud rate as an integer (e.g., 9600, 38400, 115200).
pub fn baudrate() -> i32 {
    // SAFETY: `libc::termios` is zero-initialized, which is safe for this struct.
    // `tcgetattr` will fully initialize it on success.
    let mut termios = unsafe {
        let t = MaybeUninit::<libc::termios>::zeroed();
        t.assume_init()
    };

    // SAFETY: `tcgetattr` reads terminal attributes from stdin.
    // - `STDIN_FILENO` is always a valid file descriptor
    // - `&mut termios` is a valid pointer to a `libc::termios` struct
    let result = unsafe { libc::tcgetattr(libc::STDIN_FILENO, &mut termios) };

    if result != 0 {
        // If we can't get terminal attributes, return a reasonable default
        return 38400;
    }

    // SAFETY: `cfgetospeed` reads the output speed from a valid termios struct.
    // The termios struct was successfully initialized by `tcgetattr` above.
    let ospeed = unsafe { libc::cfgetospeed(&termios) };

    // Convert speed_t constant to actual baud rate
    // The values and their meanings vary by platform, but these are common
    #[cfg(any(target_os = "linux", target_os = "android"))]
    {
        // On Linux, the speed constants are the actual baud rates
        ospeed as i32
    }

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    {
        // On macOS/BSD, the speed constants are the actual baud rates
        ospeed as i32
    }

    #[cfg(not(any(
        target_os = "linux",
        target_os = "android",
        target_os = "macos",
        target_os = "ios"
    )))]
    {
        // Fallback for other platforms - map common speed constants
        match ospeed {
            libc::B0 => 0,
            libc::B50 => 50,
            libc::B75 => 75,
            libc::B110 => 110,
            libc::B134 => 134,
            libc::B150 => 150,
            libc::B200 => 200,
            libc::B300 => 300,
            libc::B600 => 600,
            libc::B1200 => 1200,
            libc::B1800 => 1800,
            libc::B2400 => 2400,
            libc::B4800 => 4800,
            libc::B9600 => 9600,
            libc::B19200 => 19200,
            libc::B38400 => 38400,
            #[cfg(any(target_os = "freebsd", target_os = "netbsd", target_os = "openbsd"))]
            libc::B57600 => 57600,
            #[cfg(any(target_os = "freebsd", target_os = "netbsd", target_os = "openbsd"))]
            libc::B115200 => 115200,
            #[cfg(any(target_os = "freebsd", target_os = "netbsd", target_os = "openbsd"))]
            libc::B230400 => 230400,
            _ => ospeed as i32,
        }
    }
}

/// Get the erase character.
pub fn erasechar() -> char {
    '\x7f' // DEL
}

/// Get the kill character.
pub fn killchar() -> char {
    '\x15' // Ctrl-U
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_term_settings() {
        let settings = TermSettings::new();
        assert!(!settings.is_saved());
    }

    #[test]
    fn test_term_state() {
        assert_eq!(TermState::default(), TermState::Unknown);
    }
}
