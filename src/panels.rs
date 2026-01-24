//! Panels library for ncurses-rs.
//!
//! This module provides panel functionality for managing overlapping windows
//! as a deck of cards. Panels allow easy management of window visibility
//! and stacking order. This feature must be enabled with the `panels` feature flag.

use crate::error::{Error, Result};
use crate::window::Window;

use std::cell::RefCell;
use std::rc::Rc;

/// A panel wraps a window and provides stacking/ordering functionality.
pub struct Panel {
    /// The associated window.
    window: Rc<RefCell<Window>>,
    /// Whether this panel is visible.
    visible: bool,
    /// User data pointer (optional).
    user_data: Option<Box<dyn std::any::Any>>,
}

impl Panel {
    /// Create a new panel from a window.
    pub fn new(window: Window) -> Self {
        Self {
            window: Rc::new(RefCell::new(window)),
            visible: true,
            user_data: None,
        }
    }

    /// Get a reference to the panel's window.
    pub fn window(&self) -> std::cell::Ref<'_, Window> {
        self.window.borrow()
    }

    /// Get a mutable reference to the panel's window.
    pub fn window_mut(&self) -> std::cell::RefMut<'_, Window> {
        self.window.borrow_mut()
    }

    /// Check if the panel is visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Show the panel.
    pub fn show(&mut self) {
        self.visible = true;
    }

    /// Hide the panel.
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Set user data.
    pub fn set_userptr<T: 'static>(&mut self, data: T) {
        self.user_data = Some(Box::new(data));
    }

    /// Get user data.
    pub fn userptr<T: 'static>(&self) -> Option<&T> {
        self.user_data.as_ref()?.downcast_ref::<T>()
    }

    /// Move the panel to a new position.
    ///
    /// This updates the window's position on the screen. The window content
    /// is preserved but will be displayed at the new location.
    pub fn move_panel(&mut self, starty: i32, startx: i32) -> Result<()> {
        let mut win = self.window.borrow_mut();
        win.mvwin(starty, startx)
    }

    /// Replace the window associated with this panel.
    pub fn replace(&mut self, window: Window) -> Result<Rc<RefCell<Window>>> {
        let old = self.window.clone();
        self.window = Rc::new(RefCell::new(window));
        Ok(old)
    }

    /// Get the panel's window position.
    pub fn position(&self) -> (i32, i32) {
        let win = self.window.borrow();
        (win.getbegy(), win.getbegx())
    }

    /// Get the panel's window size.
    pub fn size(&self) -> (i32, i32) {
        let win = self.window.borrow();
        (win.getmaxy(), win.getmaxx())
    }
}

impl std::fmt::Debug for Panel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (y, x) = self.position();
        let (h, w) = self.size();
        f.debug_struct("Panel")
            .field("visible", &self.visible)
            .field("position", &(y, x))
            .field("size", &(h, w))
            .finish()
    }
}

/// The panel deck - manages a stack of panels.
pub struct PanelDeck {
    /// Panels in bottom-to-top order.
    panels: Vec<Rc<RefCell<Panel>>>,
}

impl PanelDeck {
    /// Create a new empty panel deck.
    pub fn new() -> Self {
        Self { panels: Vec::new() }
    }

    /// Create a new panel and add it to the top of the deck.
    pub fn new_panel(&mut self, window: Window) -> Rc<RefCell<Panel>> {
        let panel = Rc::new(RefCell::new(Panel::new(window)));
        self.panels.push(panel.clone());
        panel
    }

    /// Remove a panel from the deck.
    pub fn del_panel(&mut self, panel: &Rc<RefCell<Panel>>) -> Result<()> {
        if let Some(pos) = self.panels.iter().position(|p| Rc::ptr_eq(p, panel)) {
            self.panels.remove(pos);
            Ok(())
        } else {
            Err(Error::InvalidArgument("panel not found in deck".into()))
        }
    }

    /// Move a panel to the top of the deck.
    pub fn top_panel(&mut self, panel: &Rc<RefCell<Panel>>) -> Result<()> {
        if let Some(pos) = self.panels.iter().position(|p| Rc::ptr_eq(p, panel)) {
            let p = self.panels.remove(pos);
            self.panels.push(p);
            Ok(())
        } else {
            Err(Error::InvalidArgument("panel not found in deck".into()))
        }
    }

    /// Move a panel to the bottom of the deck.
    pub fn bottom_panel(&mut self, panel: &Rc<RefCell<Panel>>) -> Result<()> {
        if let Some(pos) = self.panels.iter().position(|p| Rc::ptr_eq(p, panel)) {
            let p = self.panels.remove(pos);
            self.panels.insert(0, p);
            Ok(())
        } else {
            Err(Error::InvalidArgument("panel not found in deck".into()))
        }
    }

    /// Get the top panel.
    pub fn ceiling_panel(&self) -> Option<Rc<RefCell<Panel>>> {
        self.panels.last().cloned()
    }

    /// Get the bottom panel.
    pub fn ground_panel(&self) -> Option<Rc<RefCell<Panel>>> {
        self.panels.first().cloned()
    }

    /// Get the panel above the given panel.
    pub fn panel_above(&self, panel: &Rc<RefCell<Panel>>) -> Option<Rc<RefCell<Panel>>> {
        let pos = self.panels.iter().position(|p| Rc::ptr_eq(p, panel))?;
        self.panels.get(pos + 1).cloned()
    }

    /// Get the panel below the given panel.
    pub fn panel_below(&self, panel: &Rc<RefCell<Panel>>) -> Option<Rc<RefCell<Panel>>> {
        let pos = self.panels.iter().position(|p| Rc::ptr_eq(p, panel))?;
        if pos > 0 {
            self.panels.get(pos - 1).cloned()
        } else {
            None
        }
    }

    /// Hide a panel (make it invisible).
    pub fn hide_panel(&self, panel: &Rc<RefCell<Panel>>) -> Result<()> {
        panel.borrow_mut().hide();
        Ok(())
    }

    /// Show a panel (make it visible).
    pub fn show_panel(&self, panel: &Rc<RefCell<Panel>>) -> Result<()> {
        panel.borrow_mut().show();
        Ok(())
    }

    /// Check if a panel is hidden.
    pub fn panel_hidden(&self, panel: &Rc<RefCell<Panel>>) -> bool {
        !panel.borrow().is_visible()
    }

    /// Update all panels (copy to virtual screen).
    ///
    /// This should be called before `doupdate()` to ensure all visible
    /// panels are properly composited. It processes panels from bottom
    /// to top, so higher panels properly overlay lower ones.
    pub fn update_panels(&self) {
        // Touch all visible panels in bottom-to-top order
        // This ensures proper z-order when refreshing
        for panel in &self.panels {
            let p = panel.borrow();
            if p.is_visible() {
                p.window_mut().touchwin();
            }
        }
    }

    /// Composite all visible panels to a destination window.
    ///
    /// This properly handles overlapping panels by copying them in
    /// bottom-to-top order, so higher panels overlay lower ones.
    ///
    /// # Arguments
    /// * `dest` - The destination window (usually newscr or a screen buffer)
    pub fn composite_to(&self, dest: &mut Window) -> Result<()> {
        // Process panels from bottom to top
        for panel in &self.panels {
            let p = panel.borrow();
            if !p.is_visible() {
                continue;
            }

            let win = p.window();
            let win_begy = win.getbegy();
            let win_begx = win.getbegx();
            let win_maxy = win.getmaxy();
            let win_maxx = win.getmaxx();

            let dest_maxy = dest.getmaxy();
            let dest_maxx = dest.getmaxx();

            // Copy each cell from the panel's window to the destination
            for y in 0..win_maxy {
                let dest_y = win_begy + y;
                if dest_y < 0 || dest_y >= dest_maxy {
                    continue;
                }

                if let Some(src_line) = win.line(y as usize) {
                    for x in 0..win_maxx {
                        let dest_x = win_begx + x;
                        if dest_x < 0 || dest_x >= dest_maxx {
                            continue;
                        }

                        let ch = src_line.get(x as usize);
                        if let Some(dest_line) = dest.line_mut(dest_y as usize) {
                            dest_line.set(dest_x as usize, ch);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Get all visible panels that contain the given screen coordinates.
    ///
    /// Returns panels from top to bottom (so the topmost panel is first).
    pub fn panels_at(&self, y: i32, x: i32) -> Vec<Rc<RefCell<Panel>>> {
        let mut result = Vec::new();
        // Iterate in reverse (top to bottom)
        for panel in self.panels.iter().rev() {
            let p = panel.borrow();
            if !p.is_visible() {
                continue;
            }

            let (begy, begx) = p.position();
            let (maxy, maxx) = p.size();

            if y >= begy && y < begy + maxy && x >= begx && x < begx + maxx {
                result.push(panel.clone());
            }
        }
        result
    }

    /// Get the number of panels in the deck.
    pub fn len(&self) -> usize {
        self.panels.len()
    }

    /// Check if the deck is empty.
    pub fn is_empty(&self) -> bool {
        self.panels.is_empty()
    }

    /// Iterate over panels from bottom to top.
    pub fn iter(&self) -> impl Iterator<Item = &Rc<RefCell<Panel>>> {
        self.panels.iter()
    }

    /// Iterate over panels from top to bottom.
    pub fn iter_rev(&self) -> impl Iterator<Item = &Rc<RefCell<Panel>>> {
        self.panels.iter().rev()
    }
}

impl Default for PanelDeck {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Free functions for ncurses compatibility
// ============================================================================

/// Get the user pointer from a panel.
///
/// This is the ncurses `panel_userptr()` function.
/// Returns `None` if no user data is set or if the type doesn't match.
pub fn panel_userptr<T: 'static>(panel: &Panel) -> Option<&T> {
    panel.userptr::<T>()
}

/// Set the user pointer for a panel.
///
/// This is the ncurses `set_panel_userptr()` function.
pub fn set_panel_userptr<T: 'static>(panel: &mut Panel, data: T) {
    panel.set_userptr(data);
}

/// Get the window associated with a panel.
///
/// This is the ncurses `panel_window()` function.
/// Returns a reference to the panel's window.
pub fn panel_window(panel: &Panel) -> std::cell::Ref<'_, Window> {
    panel.window()
}

/// Replace the window associated with a panel.
///
/// This is the ncurses `replace_panel()` function.
/// Returns the old window wrapped in `Rc<RefCell>`.
pub fn replace_panel(panel: &mut Panel, window: Window) -> Result<Rc<RefCell<Window>>> {
    panel.replace(window)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panel_creation() {
        let win = Window::new(10, 20, 0, 0).unwrap();
        let panel = Panel::new(win);
        assert!(panel.is_visible());
    }

    #[test]
    fn test_panel_visibility() {
        let win = Window::new(10, 20, 0, 0).unwrap();
        let mut panel = Panel::new(win);

        panel.hide();
        assert!(!panel.is_visible());

        panel.show();
        assert!(panel.is_visible());
    }

    #[test]
    fn test_panel_deck() {
        let mut deck = PanelDeck::new();

        let win1 = Window::new(10, 20, 0, 0).unwrap();
        let win2 = Window::new(10, 20, 5, 5).unwrap();

        let p1 = deck.new_panel(win1);
        let p2 = deck.new_panel(win2);

        assert_eq!(deck.len(), 2);

        // p2 should be on top
        assert!(Rc::ptr_eq(&deck.ceiling_panel().unwrap(), &p2));
        assert!(Rc::ptr_eq(&deck.ground_panel().unwrap(), &p1));

        // Move p1 to top
        deck.top_panel(&p1).unwrap();
        assert!(Rc::ptr_eq(&deck.ceiling_panel().unwrap(), &p1));
    }

    #[test]
    fn test_panel_user_data() {
        let win = Window::new(10, 20, 0, 0).unwrap();
        let mut panel = Panel::new(win);

        panel.set_userptr(42i32);
        assert_eq!(panel.userptr::<i32>(), Some(&42));
        assert_eq!(panel.userptr::<String>(), None);
    }

    #[test]
    fn test_panel_move() {
        let win = Window::new(10, 20, 0, 0).unwrap();
        let mut panel = Panel::new(win);

        assert_eq!(panel.position(), (0, 0));
        panel.move_panel(5, 10).unwrap();
        assert_eq!(panel.position(), (5, 10));
    }

    #[test]
    fn test_panels_at() {
        let mut deck = PanelDeck::new();

        let win1 = Window::new(10, 20, 0, 0).unwrap();
        let win2 = Window::new(10, 20, 5, 5).unwrap();

        let _p1 = deck.new_panel(win1);
        let _p2 = deck.new_panel(win2);

        // Point (7, 7) should be in both panels
        let at_7_7 = deck.panels_at(7, 7);
        assert_eq!(at_7_7.len(), 2);

        // Point (1, 1) should only be in p1
        let at_1_1 = deck.panels_at(1, 1);
        assert_eq!(at_1_1.len(), 1);

        // Point (20, 20) should be in neither
        let at_20_20 = deck.panels_at(20, 20);
        assert_eq!(at_20_20.len(), 0);
    }
}
