//! # ncurses-pure
//!
//! A pure Rust implementation of the ncurses library, compliant with the
//! X/Open XSI Curses standard.
//!
//! ## Features
//!
//! - **wide**: Wide character support (Unicode via `cchar_t`)
//! - **mouse**: Mouse event handling
//! - **ext-colors**: Extended colors (>256 color pairs)
//! - **slk**: Soft function key labels
//! - **panels**: Panels library for window stacking
//! - **menu**: Menu library for selection interfaces
//! - **form**: Form library for data entry
//!
//! ## Example
//!
//! ```rust,no_run
//! use ncurses::*;
//!
//! fn main() -> Result<()> {
//!     let mut screen = Screen::init()?;
//!     
//!     screen.start_color()?;
//!     screen.init_pair(1, COLOR_RED, COLOR_BLACK)?;
//!     
//!     // Access stdscr for window operations
//!     {
//!         let stdscr = screen.stdscr_mut();
//!         stdscr.keypad(true);
//!         stdscr.addstr("Hello, ncurses-pure!")?;
//!         stdscr.attron(attr::A_BOLD)?;
//!         stdscr.addstr(" (Press any key)")?;
//!     }
//!     
//!     screen.refresh()?;
//!     screen.getch()?;
//!     Ok(())
//! }
//! ```

#![allow(clippy::needless_doctest_main)]
#![warn(missing_docs)]

pub mod acs;
pub mod attr;
pub mod color;
pub mod error;
pub mod input;
pub mod key;
pub mod line;
pub mod screen;
pub mod terminal;
pub mod types;
pub mod window;

#[cfg(feature = "mouse")]
pub mod mouse;

#[cfg(feature = "wide")]
pub mod wide;

#[cfg(feature = "slk")]
pub mod slk;

#[cfg(feature = "panels")]
pub mod panels;

#[cfg(feature = "menu")]
pub mod menu;

#[cfg(feature = "form")]
pub mod form;

#[cfg(feature = "trace")]
pub mod trace;

#[cfg(feature = "sp-funcs")]
pub mod threadsafe;

// Re-export commonly used items at crate root
pub use acs::*;
pub use attr::*;
pub use color::*;
pub use error::{Error, Result};
pub use input::*;
pub use key::*;
pub use screen::globals::{COLS, LINES};
pub use screen::Screen;
pub use types::*;
pub use window::Window;

#[cfg(feature = "mouse")]
pub use mouse::*;

#[cfg(feature = "wide")]
pub use wide::*;

/// The ncurses version string
pub const VERSION: &str = "0.1.0";

/// ncurses major version
pub const VERSION_MAJOR: u32 = 0;

/// ncurses minor version
pub const VERSION_MINOR: u32 = 1;

/// ncurses patch version
pub const VERSION_PATCH: u32 = 0;
