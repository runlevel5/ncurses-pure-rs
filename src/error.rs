//! Error types for ncurses-rs.

use std::fmt;

/// Result type alias for ncurses operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Error types that can occur in ncurses operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// A general error occurred (equivalent to ncurses ERR).
    General,

    /// An invalid argument was passed to a function.
    InvalidArgument(String),

    /// The terminal is not initialized.
    NotInitialized,

    /// The terminal has already been initialized.
    AlreadyInitialized,

    /// A window operation failed.
    WindowError(String),

    /// The cursor position is out of bounds.
    OutOfBounds {
        /// The requested Y coordinate.
        y: i32,
        /// The requested X coordinate.
        x: i32,
        /// The maximum Y coordinate.
        max_y: i32,
        /// The maximum X coordinate.
        max_x: i32,
    },

    /// Color support is not available or not started.
    ColorNotAvailable,

    /// An invalid color pair was specified.
    InvalidColorPair(i16),

    /// An invalid color value was specified.
    InvalidColor(i16),

    /// An I/O error occurred.
    Io(std::io::ErrorKind),

    /// A system call failed with the given errno.
    SystemError(i32),

    /// The terminal type is unknown or unsupported.
    UnknownTerminal(String),

    /// Memory allocation failed.
    OutOfMemory,

    /// The operation is not supported.
    NotSupported(String),

    /// An operation timed out.
    Timeout,

    /// Input was interrupted.
    Interrupted,

    /// End of file was reached on input.
    EndOfFile,

    /// A key code is invalid or undefined.
    InvalidKey(i32),

    /// No input is available (non-blocking mode).
    NoInput,

    /// End of file on input.
    Eof,

    /// Input buffer is full.
    BufferFull,

    /// Mouse support is not available.
    #[cfg(feature = "mouse")]
    MouseNotAvailable,

    /// An invalid mouse event occurred.
    #[cfg(feature = "mouse")]
    InvalidMouseEvent,

    /// A lock (mutex/rwlock) was poisoned.
    #[cfg(feature = "sp-funcs")]
    LockPoisoned,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::General => write!(f, "ncurses error"),
            Error::InvalidArgument(msg) => write!(f, "invalid argument: {}", msg),
            Error::NotInitialized => write!(f, "terminal not initialized"),
            Error::AlreadyInitialized => write!(f, "terminal already initialized"),
            Error::WindowError(msg) => write!(f, "window error: {}", msg),
            Error::OutOfBounds { y, x, max_y, max_x } => {
                write!(
                    f,
                    "position ({}, {}) out of bounds (max: {}, {})",
                    y, x, max_y, max_x
                )
            }
            Error::ColorNotAvailable => write!(f, "color support not available"),
            Error::InvalidColorPair(pair) => write!(f, "invalid color pair: {}", pair),
            Error::InvalidColor(color) => write!(f, "invalid color: {}", color),
            Error::Io(kind) => write!(f, "I/O error: {:?}", kind),
            Error::SystemError(errno) => write!(f, "system error: {}", errno),
            Error::UnknownTerminal(term) => write!(f, "unknown terminal: {}", term),
            Error::OutOfMemory => write!(f, "out of memory"),
            Error::NotSupported(msg) => write!(f, "not supported: {}", msg),
            Error::Timeout => write!(f, "operation timed out"),
            Error::Interrupted => write!(f, "operation interrupted"),
            Error::EndOfFile => write!(f, "end of file"),
            Error::InvalidKey(key) => write!(f, "invalid key code: {}", key),
            Error::NoInput => write!(f, "no input available"),
            Error::Eof => write!(f, "end of file on input"),
            Error::BufferFull => write!(f, "input buffer is full"),
            #[cfg(feature = "mouse")]
            Error::MouseNotAvailable => write!(f, "mouse support not available"),
            #[cfg(feature = "mouse")]
            Error::InvalidMouseEvent => write!(f, "invalid mouse event"),
            #[cfg(feature = "sp-funcs")]
            Error::LockPoisoned => write!(f, "lock poisoned"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err.kind())
    }
}

/// Trait for converting ncurses return values to Result.
pub trait IntoResult {
    /// The success type.
    type Output;

    /// Convert to a Result.
    fn into_result(self) -> Result<Self::Output>;
}

impl IntoResult for i32 {
    type Output = ();

    fn into_result(self) -> Result<Self::Output> {
        if self == crate::types::OK {
            Ok(())
        } else {
            Err(Error::General)
        }
    }
}

impl<T> IntoResult for Option<T> {
    type Output = T;

    fn into_result(self) -> Result<Self::Output> {
        self.ok_or(Error::General)
    }
}
