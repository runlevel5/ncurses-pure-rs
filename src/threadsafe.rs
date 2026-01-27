//! # Thread Safety Wrappers
//!
//! This module provides thread-safe access to ncurses Screen and Window objects.
//! These functions are only available when the `sp-funcs` feature is enabled.
//!
//! ## Overview
//!
//! In traditional ncurses, functions like `use_screen()` and `use_window()` provide
//! thread-safe access by acquiring a lock before executing a callback. In Rust,
//! we achieve the same goal using `Mutex` or `RwLock` wrappers.
//!
//! ## Example
//!
//! ```rust,ignore
//! use ncurses::threadsafe::*;
//! use std::sync::Arc;
//!
//! // Create a thread-safe screen wrapper
//! let screen = ThreadSafeScreen::new(Screen::init()?);
//!
//! // Access from multiple threads
//! let screen_clone = screen.clone();
//! std::thread::spawn(move || {
//!     screen_clone.use_screen(|s| {
//!         s.refresh()?;
//!         Ok(())
//!     });
//! });
//! ```

use crate::screen::Screen;
use crate::window::Window;
use crate::Result;
use std::sync::{Arc, Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

// ============================================================================
// Thread-safe Screen wrapper
// ============================================================================

/// A thread-safe wrapper around a Screen.
///
/// This wrapper uses a `Mutex` to ensure that only one thread can access
/// the screen at a time. This is equivalent to ncurses' `use_screen()` function.
///
/// # Example
///
/// ```rust,ignore
/// use ncurses::threadsafe::ThreadSafeScreen;
///
/// let screen = ThreadSafeScreen::new(Screen::init()?);
///
/// // Use the screen safely from any thread
/// screen.use_screen(|s| {
///     s.printw("Hello from thread!")?;
///     s.refresh()
/// })?;
/// ```
#[derive(Clone)]
pub struct ThreadSafeScreen {
    inner: Arc<Mutex<Screen>>,
}

impl ThreadSafeScreen {
    /// Create a new thread-safe screen wrapper.
    pub fn new(screen: Screen) -> Self {
        Self {
            inner: Arc::new(Mutex::new(screen)),
        }
    }

    /// Execute a function with exclusive access to the screen.
    ///
    /// This is equivalent to ncurses' `use_screen()` function.
    /// The callback receives a mutable reference to the screen and can
    /// perform any screen operations safely.
    ///
    /// # Arguments
    ///
    /// * `f` - A closure that takes `&mut Screen` and returns a `Result<T>`
    ///
    /// # Returns
    ///
    /// The result of the closure, or an error if the lock is poisoned.
    pub fn use_screen<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&mut Screen) -> Result<T>,
    {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| crate::error::Error::LockPoisoned)?;
        f(&mut guard)
    }

    /// Try to execute a function with exclusive access to the screen.
    ///
    /// This is a non-blocking version that returns `None` if the lock
    /// cannot be acquired immediately.
    ///
    /// # Arguments
    ///
    /// * `f` - A closure that takes `&mut Screen` and returns a `Result<T>`
    ///
    /// # Returns
    ///
    /// `Some(result)` if the lock was acquired, `None` otherwise.
    pub fn try_use_screen<F, T>(&self, f: F) -> Option<Result<T>>
    where
        F: FnOnce(&mut Screen) -> Result<T>,
    {
        self.inner.try_lock().ok().map(|mut guard| f(&mut guard))
    }

    /// Get direct access to the mutex guard.
    ///
    /// This is useful when you need to hold the lock for multiple operations.
    /// Be careful to release the guard promptly to avoid blocking other threads.
    pub fn lock(
        &self,
    ) -> std::result::Result<MutexGuard<'_, Screen>, std::sync::PoisonError<MutexGuard<'_, Screen>>>
    {
        self.inner.lock()
    }
}

// ============================================================================
// Thread-safe Window wrapper (using RwLock for read/write separation)
// ============================================================================

/// A thread-safe wrapper around a Window.
///
/// This wrapper uses an `RwLock` to allow multiple readers or a single writer.
/// This is equivalent to ncurses' `use_window()` function.
///
/// # Example
///
/// ```rust,ignore
/// use ncurses::threadsafe::ThreadSafeWindow;
///
/// let window = ThreadSafeWindow::new(Window::new(10, 40, 0, 0)?);
///
/// // Multiple threads can read simultaneously
/// window.use_window_read(|w| {
///     let (y, x) = (w.getcury(), w.getcurx());
///     println!("Cursor at ({}, {})", y, x);
/// });
///
/// // Only one thread can write at a time
/// window.use_window(|w| {
///     w.addstr("Hello!")?;
///     Ok(())
/// })?;
/// ```
#[derive(Clone)]
pub struct ThreadSafeWindow {
    inner: Arc<RwLock<Window>>,
}

impl ThreadSafeWindow {
    /// Create a new thread-safe window wrapper.
    pub fn new(window: Window) -> Self {
        Self {
            inner: Arc::new(RwLock::new(window)),
        }
    }

    /// Execute a function with exclusive (write) access to the window.
    ///
    /// This is equivalent to ncurses' `use_window()` function.
    /// The callback receives a mutable reference to the window and can
    /// perform any window operations safely.
    ///
    /// # Arguments
    ///
    /// * `f` - A closure that takes `&mut Window` and returns a `Result<T>`
    ///
    /// # Returns
    ///
    /// The result of the closure, or an error if the lock is poisoned.
    pub fn use_window<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&mut Window) -> Result<T>,
    {
        let mut guard = self
            .inner
            .write()
            .map_err(|_| crate::error::Error::LockPoisoned)?;
        f(&mut guard)
    }

    /// Execute a function with shared (read) access to the window.
    ///
    /// Multiple threads can hold read access simultaneously.
    /// The callback receives an immutable reference to the window.
    ///
    /// # Arguments
    ///
    /// * `f` - A closure that takes `&Window` and returns `T`
    ///
    /// # Returns
    ///
    /// The result of the closure, or an error if the lock is poisoned.
    pub fn use_window_read<F, T>(&self, f: F) -> std::result::Result<T, crate::error::Error>
    where
        F: FnOnce(&Window) -> T,
    {
        let guard = self
            .inner
            .read()
            .map_err(|_| crate::error::Error::LockPoisoned)?;
        Ok(f(&guard))
    }

    /// Try to execute a function with exclusive access to the window.
    ///
    /// This is a non-blocking version that returns `None` if the lock
    /// cannot be acquired immediately.
    pub fn try_use_window<F, T>(&self, f: F) -> Option<Result<T>>
    where
        F: FnOnce(&mut Window) -> Result<T>,
    {
        self.inner.try_write().ok().map(|mut guard| f(&mut guard))
    }

    /// Try to execute a function with shared access to the window.
    ///
    /// This is a non-blocking version that returns `None` if the lock
    /// cannot be acquired immediately.
    pub fn try_use_window_read<F, T>(&self, f: F) -> Option<T>
    where
        F: FnOnce(&Window) -> T,
    {
        self.inner.try_read().ok().map(|guard| f(&guard))
    }

    /// Get direct write access to the RwLock guard.
    pub fn write(
        &self,
    ) -> std::result::Result<
        RwLockWriteGuard<'_, Window>,
        std::sync::PoisonError<RwLockWriteGuard<'_, Window>>,
    > {
        self.inner.write()
    }

    /// Get direct read access to the RwLock guard.
    pub fn read(
        &self,
    ) -> std::result::Result<
        RwLockReadGuard<'_, Window>,
        std::sync::PoisonError<RwLockReadGuard<'_, Window>>,
    > {
        self.inner.read()
    }
}

// ============================================================================
// Convenience type aliases
// ============================================================================

/// Type alias for a shared thread-safe screen.
pub type SharedScreen = Arc<Mutex<Screen>>;

/// Type alias for a shared thread-safe window.
pub type SharedWindow = Arc<RwLock<Window>>;

// ============================================================================
// Free functions matching ncurses API
// ============================================================================

/// Execute a function with exclusive access to a screen.
///
/// This matches the ncurses `use_screen(SCREEN*, NCURSES_SCREEN_CB, void*)` API.
/// In the C API, the callback receives the screen and a void* data pointer.
/// In Rust, we use closures which can capture any data needed.
///
/// # C API Equivalent
///
/// ```c
/// int use_screen(SCREEN *scr, NCURSES_SCREEN_CB func, void *data);
/// // where NCURSES_SCREEN_CB is: int (*)(SCREEN*, void*)
/// ```
///
/// # Arguments
///
/// * `screen` - The thread-safe screen wrapper
/// * `f` - The function to execute with exclusive screen access
///
/// # Returns
///
/// The result of the function, or an error if the lock is poisoned.
///
/// # Example
///
/// ```rust,ignore
/// use ncurses::threadsafe::{ThreadSafeScreen, use_screen};
///
/// let screen = ThreadSafeScreen::new(Screen::init()?);
///
/// // Use from any thread
/// use_screen(&screen, |scr| {
///     scr.printw("Hello!")?;
///     scr.refresh()
/// })?;
/// ```
pub fn use_screen<F, T>(screen: &ThreadSafeScreen, f: F) -> Result<T>
where
    F: FnOnce(&mut Screen) -> Result<T>,
{
    screen.use_screen(f)
}

/// Execute a function with exclusive access to a mutex-protected screen.
///
/// This is a lower-level version that works directly with `Arc<Mutex<Screen>>`.
///
/// # Arguments
///
/// * `screen` - A mutex-protected screen
/// * `f` - The function to execute
///
/// # Returns
///
/// The result of the function, or an error if the lock is poisoned.
pub fn use_screen_mutex<F, T>(screen: &Mutex<Screen>, f: F) -> Result<T>
where
    F: FnOnce(&mut Screen) -> Result<T>,
{
    let mut guard = screen
        .lock()
        .map_err(|_| crate::error::Error::LockPoisoned)?;
    f(&mut guard)
}

/// Execute a function with exclusive access to a window.
///
/// This matches the ncurses `use_window(WINDOW*, NCURSES_WINDOW_CB, void*)` API.
/// In the C API, the callback receives the window and a void* data pointer.
/// In Rust, we use closures which can capture any data needed.
///
/// # C API Equivalent
///
/// ```c
/// int use_window(WINDOW *win, NCURSES_WINDOW_CB func, void *data);
/// // where NCURSES_WINDOW_CB is: int (*)(WINDOW*, void*)
/// ```
///
/// # Arguments
///
/// * `window` - The thread-safe window wrapper
/// * `f` - The function to execute with exclusive window access
///
/// # Returns
///
/// The result of the function, or an error if the lock is poisoned.
///
/// # Example
///
/// ```rust,ignore
/// use ncurses::threadsafe::{ThreadSafeWindow, use_window};
///
/// let window = ThreadSafeWindow::new(Window::new(10, 40, 0, 0)?);
///
/// // Use from any thread
/// use_window(&window, |win| {
///     win.addstr("Hello!")?;
///     win.refresh()
/// })?;
/// ```
pub fn use_window<F, T>(window: &ThreadSafeWindow, f: F) -> Result<T>
where
    F: FnOnce(&mut Window) -> Result<T>,
{
    window.use_window(f)
}

/// Execute a function with exclusive access to an RwLock-protected window.
///
/// This is a lower-level version that works directly with `Arc<RwLock<Window>>`.
///
/// # Arguments
///
/// * `window` - An RwLock-protected window
/// * `f` - The function to execute
///
/// # Returns
///
/// The result of the function, or an error if the lock is poisoned.
pub fn use_window_rwlock<F, T>(window: &RwLock<Window>, f: F) -> Result<T>
where
    F: FnOnce(&mut Window) -> Result<T>,
{
    let mut guard = window
        .write()
        .map_err(|_| crate::error::Error::LockPoisoned)?;
    f(&mut guard)
}

/// Execute a function with shared (read) access to an RwLock-protected window.
///
/// Multiple threads can hold read access simultaneously.
///
/// # Arguments
///
/// * `window` - An RwLock-protected window
/// * `f` - The function to execute with shared window access
///
/// # Returns
///
/// The result of the function, or an error if the lock is poisoned.
pub fn use_window_read<F, T>(
    window: &ThreadSafeWindow,
    f: F,
) -> std::result::Result<T, crate::error::Error>
where
    F: FnOnce(&Window) -> T,
{
    window.use_window_read(f)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thread_safe_window_creation() {
        let window = Window::new(10, 40, 0, 0);
        assert!(window.is_ok());
        let ts_window = ThreadSafeWindow::new(window.unwrap());

        // Test that we can clone the wrapper
        let _clone = ts_window.clone();
    }

    #[test]
    fn test_thread_safe_window_read() {
        let window = Window::new(10, 40, 0, 0).unwrap();
        let ts_window = ThreadSafeWindow::new(window);

        let result = ts_window.use_window_read(|w| (w.getmaxy(), w.getmaxx()));

        assert!(result.is_ok());
        let (height, width) = result.unwrap();
        assert_eq!(height, 10); // getmaxy() returns height (nlines)
        assert_eq!(width, 40); // getmaxx() returns width (ncols)
    }

    #[test]
    fn test_thread_safe_window_write() {
        let window = Window::new(10, 40, 0, 0).unwrap();
        let ts_window = ThreadSafeWindow::new(window);

        let result = ts_window.use_window(|w| {
            w.addstr("Hello")?;
            Ok(())
        });

        assert!(result.is_ok());
    }
}
