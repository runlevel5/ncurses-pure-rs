//! Menu library for ncurses-pure.
//!
//! This module provides menu functionality for creating selectable menus
//! with items. This feature must be enabled with the `menu` feature flag.

use crate::error::{Error, Result};
use crate::types::{AttrT, ChType};
use crate::window::Window;

use std::cell::RefCell;
use std::rc::Rc;

// ============================================================================
// Menu option flags
// ============================================================================

bitflags::bitflags! {
    /// Menu option flags.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    pub struct MenuOpts: u32 {
        /// Show descriptions.
        const O_SHOWDESC = 0x01;
        /// Row-major order.
        const O_ROWMAJOR = 0x02;
        /// Ignore case in pattern match.
        const O_IGNORECASE = 0x04;
        /// Show match cursor.
        const O_SHOWMATCH = 0x08;
        /// Only one item can be selected.
        const O_ONEVALUE = 0x10;
        /// Non-cyclic scrolling.
        const O_NONCYCLIC = 0x20;
        /// Mouse menu support.
        const O_MOUSE_MENU = 0x40;
    }
}

bitflags::bitflags! {
    /// Item option flags.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    pub struct ItemOpts: u32 {
        /// Item can be selected.
        const O_SELECTABLE = 0x01;
    }
}

// ============================================================================
// Menu request codes
// ============================================================================

/// Menu request: move to next item.
pub const REQ_NEXT_ITEM: i32 = 0x200;
/// Menu request: move to previous item.
pub const REQ_PREV_ITEM: i32 = 0x201;
/// Menu request: move down.
pub const REQ_DOWN_ITEM: i32 = 0x202;
/// Menu request: move up.
pub const REQ_UP_ITEM: i32 = 0x203;
/// Menu request: move left.
pub const REQ_LEFT_ITEM: i32 = 0x204;
/// Menu request: move right.
pub const REQ_RIGHT_ITEM: i32 = 0x205;
/// Menu request: scroll down a line.
pub const REQ_SCR_DLINE: i32 = 0x206;
/// Menu request: scroll up a line.
pub const REQ_SCR_ULINE: i32 = 0x207;
/// Menu request: scroll down a page.
pub const REQ_SCR_DPAGE: i32 = 0x208;
/// Menu request: scroll up a page.
pub const REQ_SCR_UPAGE: i32 = 0x209;
/// Menu request: go to first item.
pub const REQ_FIRST_ITEM: i32 = 0x20a;
/// Menu request: go to last item.
pub const REQ_LAST_ITEM: i32 = 0x20b;
/// Menu request: toggle item selection.
pub const REQ_TOGGLE_ITEM: i32 = 0x20c;
/// Menu request: clear pattern.
pub const REQ_CLEAR_PATTERN: i32 = 0x20d;
/// Menu request: back pattern.
pub const REQ_BACK_PATTERN: i32 = 0x20e;
/// Menu request: next match.
pub const REQ_NEXT_MATCH: i32 = 0x20f;
/// Menu request: previous match.
pub const REQ_PREV_MATCH: i32 = 0x210;

// ============================================================================
// Menu Item
// ============================================================================

/// A menu item.
pub struct MenuItem {
    /// Item name.
    name: String,
    /// Item description.
    description: String,
    /// Item options.
    opts: ItemOpts,
    /// Whether this item is selected.
    selected: bool,
    /// User data.
    user_data: Option<Box<dyn std::any::Any>>,
    /// Item index in the menu.
    index: usize,
}

impl MenuItem {
    /// Create a new menu item.
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            opts: ItemOpts::O_SELECTABLE,
            selected: false,
            user_data: None,
            index: 0,
        }
    }

    /// Get the item name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the item description.
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Check if the item is selectable.
    pub fn is_selectable(&self) -> bool {
        self.opts.contains(ItemOpts::O_SELECTABLE)
    }

    /// Check if the item is selected.
    pub fn is_selected(&self) -> bool {
        self.selected
    }

    /// Set item options.
    pub fn set_opts(&mut self, opts: ItemOpts) {
        self.opts = opts;
    }

    /// Get item options.
    pub fn opts(&self) -> ItemOpts {
        self.opts
    }

    /// Set user data.
    pub fn set_userptr<T: 'static>(&mut self, data: T) {
        self.user_data = Some(Box::new(data));
    }

    /// Get user data.
    pub fn userptr<T: 'static>(&self) -> Option<&T> {
        self.user_data.as_ref()?.downcast_ref::<T>()
    }

    /// Get the item index.
    pub fn index(&self) -> usize {
        self.index
    }
}

impl std::fmt::Debug for MenuItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MenuItem")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("opts", &self.opts)
            .field("selected", &self.selected)
            .finish()
    }
}

// ============================================================================
// Menu
// ============================================================================

/// Type alias for menu hook callbacks.
///
/// These are called at specific points in menu/item lifecycle:
/// - Menu init: when menu is posted or after top row changes
/// - Menu term: when menu is unposted or before top row changes
/// - Item init: when menu is posted or after current item changes
/// - Item term: when menu is unposted or before current item changes
pub type MenuHook = Box<dyn Fn(&Menu)>;

/// A menu containing items.
pub struct Menu {
    /// The menu items.
    items: Vec<Rc<RefCell<MenuItem>>>,
    /// Current item index.
    current: usize,
    /// Top item index (for scrolling).
    top: usize,
    /// Menu options.
    opts: MenuOpts,
    /// Number of rows to display.
    rows: i32,
    /// Number of columns to display.
    cols: i32,
    /// Pattern buffer for search.
    pattern: String,
    /// Mark character for selected items.
    mark: String,
    /// Foreground attribute for selected item.
    fore: AttrT,
    /// Background attribute for unselected items.
    back: AttrT,
    /// Grey attribute for non-selectable items.
    grey: AttrT,
    /// The menu window.
    window: Option<Rc<RefCell<Window>>>,
    /// The menu sub-window.
    sub_window: Option<Rc<RefCell<Window>>>,
    /// User data.
    user_data: Option<Box<dyn std::any::Any>>,
    /// Whether the menu is posted.
    posted: bool,
    /// Item name width (calculated).
    name_width: usize,
    /// Item description width (calculated).
    desc_width: usize,
    /// Spacing between name and description.
    spacing: usize,
    /// Menu initialization hook (called when posted or after top row changes).
    menu_init: Option<MenuHook>,
    /// Menu termination hook (called when unposted or before top row changes).
    menu_term: Option<MenuHook>,
    /// Item initialization hook (called when posted or after current item changes).
    item_init: Option<MenuHook>,
    /// Item termination hook (called when unposted or before current item changes).
    item_term: Option<MenuHook>,
}

impl Menu {
    /// Create a new menu.
    pub fn new(items: Vec<MenuItem>) -> Self {
        let mut name_width = 0usize;
        let mut desc_width = 0usize;

        let items: Vec<_> = items
            .into_iter()
            .enumerate()
            .map(|(i, mut item)| {
                item.index = i;
                name_width = name_width.max(item.name.len());
                desc_width = desc_width.max(item.description.len());
                Rc::new(RefCell::new(item))
            })
            .collect();

        Self {
            items,
            current: 0,
            top: 0,
            opts: MenuOpts::O_ONEVALUE | MenuOpts::O_SHOWDESC,
            rows: 16,
            cols: 1,
            pattern: String::new(),
            mark: String::from("-"),
            fore: 0,
            back: 0,
            grey: 0,
            window: None,
            sub_window: None,
            user_data: None,
            posted: false,
            name_width,
            desc_width,
            spacing: 2,
            menu_init: None,
            menu_term: None,
            item_init: None,
            item_term: None,
        }
    }

    /// Get the number of items.
    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    /// Get all menu items.
    ///
    /// Returns a slice of all items in the menu.
    pub fn items(&self) -> &[Rc<RefCell<MenuItem>>] {
        &self.items
    }

    /// Set the menu items.
    ///
    /// This replaces all items in the menu. The menu must not be posted.
    /// After setting items, the current selection is reset to the first item.
    ///
    /// # Arguments
    ///
    /// * `items` - The new items for the menu
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the menu is currently posted.
    pub fn set_items(&mut self, items: Vec<MenuItem>) -> Result<()> {
        if self.posted {
            return Err(Error::InvalidArgument(
                "cannot change items while menu is posted".into(),
            ));
        }

        // Recalculate widths
        let mut name_width = 0usize;
        let mut desc_width = 0usize;

        let items: Vec<_> = items
            .into_iter()
            .enumerate()
            .map(|(i, mut item)| {
                item.index = i;
                name_width = name_width.max(item.name.len());
                desc_width = desc_width.max(item.description.len());
                Rc::new(RefCell::new(item))
            })
            .collect();

        self.items = items;
        self.name_width = name_width;
        self.desc_width = desc_width;
        self.current = 0;
        self.top = 0;
        self.pattern.clear();

        Ok(())
    }

    /// Get an item by index.
    pub fn item(&self, index: usize) -> Option<Rc<RefCell<MenuItem>>> {
        self.items.get(index).cloned()
    }

    /// Get the current item.
    pub fn current_item(&self) -> Option<Rc<RefCell<MenuItem>>> {
        self.items.get(self.current).cloned()
    }

    /// Set the current item by index.
    ///
    /// This calls the item termination hook before changing items,
    /// and the item initialization hook after changing items.
    pub fn set_current_item(&mut self, index: usize) -> Result<()> {
        if index < self.items.len() {
            if index != self.current && self.posted {
                // Call item term hook before changing
                self.call_item_term();
            }
            self.current = index;
            self.adjust_scroll();
            if self.posted {
                // Call item init hook after changing
                self.call_item_init();
            }
            Ok(())
        } else {
            Err(Error::InvalidArgument("invalid item index".into()))
        }
    }

    /// Get the current item index.
    pub fn current_item_index(&self) -> usize {
        self.current
    }

    /// Set menu options.
    pub fn set_opts(&mut self, opts: MenuOpts) {
        self.opts = opts;
    }

    /// Get menu options.
    pub fn opts(&self) -> MenuOpts {
        self.opts
    }

    /// Turn on menu options.
    pub fn opts_on(&mut self, opts: MenuOpts) {
        self.opts |= opts;
    }

    /// Turn off menu options.
    pub fn opts_off(&mut self, opts: MenuOpts) {
        self.opts &= !opts;
    }

    /// Set the menu format (rows and columns).
    pub fn set_format(&mut self, rows: i32, cols: i32) {
        self.rows = rows.max(1);
        self.cols = cols.max(1);
    }

    /// Get the menu format.
    pub fn format(&self) -> (i32, i32) {
        (self.rows, self.cols)
    }

    /// Set the mark string.
    pub fn set_mark(&mut self, mark: &str) {
        self.mark = mark.to_string();
    }

    /// Get the mark string.
    pub fn mark(&self) -> &str {
        &self.mark
    }

    /// Set the foreground attribute.
    pub fn set_fore(&mut self, attr: AttrT) {
        self.fore = attr;
    }

    /// Get the foreground attribute.
    pub fn fore(&self) -> AttrT {
        self.fore
    }

    /// Set the background attribute.
    pub fn set_back(&mut self, attr: AttrT) {
        self.back = attr;
    }

    /// Get the background attribute.
    pub fn back(&self) -> AttrT {
        self.back
    }

    /// Set the grey attribute.
    pub fn set_grey(&mut self, attr: AttrT) {
        self.grey = attr;
    }

    /// Get the grey attribute.
    pub fn grey(&self) -> AttrT {
        self.grey
    }

    /// Set the menu window.
    pub fn set_window(&mut self, window: Window) {
        self.window = Some(Rc::new(RefCell::new(window)));
    }

    /// Set the menu sub-window.
    pub fn set_sub(&mut self, window: Window) {
        self.sub_window = Some(Rc::new(RefCell::new(window)));
    }

    /// Get the current pattern.
    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    /// Set the pattern.
    pub fn set_pattern(&mut self, pattern: &str) -> Result<()> {
        self.pattern = pattern.to_string();
        // Find first matching item
        if !self.pattern.is_empty() {
            if let Some(idx) = self.find_match_forward(0) {
                self.current = idx;
                self.adjust_scroll();
            }
        }
        Ok(())
    }

    /// Post the menu (display it).
    pub fn post(&mut self) -> Result<()> {
        if self.posted {
            return Err(Error::InvalidArgument("menu already posted".into()));
        }
        self.posted = true;
        // Call menu init hook
        self.call_menu_init();
        // Call item init hook for the current item
        self.call_item_init();
        self.render()?;
        Ok(())
    }

    /// Unpost the menu (hide it).
    pub fn unpost(&mut self) -> Result<()> {
        if !self.posted {
            return Err(Error::InvalidArgument("menu not posted".into()));
        }
        // Call item term hook for the current item
        self.call_item_term();
        // Call menu term hook
        self.call_menu_term();
        self.posted = false;
        // Clear the menu window
        if let Some(ref sub) = self.sub_window {
            sub.borrow_mut().erase()?;
        } else if let Some(ref win) = self.window {
            win.borrow_mut().erase()?;
        }
        Ok(())
    }

    /// Check if the menu is posted.
    pub fn is_posted(&self) -> bool {
        self.posted
    }

    /// Adjust scroll position to keep current item visible.
    fn adjust_scroll(&mut self) {
        let visible_rows = self.rows as usize;
        if self.current < self.top {
            self.top = self.current;
        } else if self.current >= self.top + visible_rows {
            self.top = self.current - visible_rows + 1;
        }
    }

    /// Find the next item matching the pattern, starting from `start`.
    fn find_match_forward(&self, start: usize) -> Option<usize> {
        if self.pattern.is_empty() {
            return None;
        }
        let ignore_case = self.opts.contains(MenuOpts::O_IGNORECASE);

        // Helper to check if item name starts with pattern
        let matches = |item: &MenuItem| -> bool {
            if ignore_case {
                item.name
                    .to_lowercase()
                    .starts_with(&self.pattern.to_lowercase())
            } else {
                item.name.starts_with(&self.pattern)
            }
        };

        // Search from start to end
        if let Some(i) = (start..self.items.len()).find(|&i| matches(&self.items[i].borrow())) {
            return Some(i);
        }

        // Wrap around from beginning to start
        (0..start).find(|&i| matches(&self.items[i].borrow()))
    }

    /// Find the previous item matching the pattern, starting from `start`.
    fn find_match_backward(&self, start: usize) -> Option<usize> {
        if self.pattern.is_empty() {
            return None;
        }
        let ignore_case = self.opts.contains(MenuOpts::O_IGNORECASE);

        // Helper to check if item name starts with pattern
        let matches = |item: &MenuItem| -> bool {
            if ignore_case {
                item.name
                    .to_lowercase()
                    .starts_with(&self.pattern.to_lowercase())
            } else {
                item.name.starts_with(&self.pattern)
            }
        };

        // Search from start to beginning
        if let Some(i) = (0..=start)
            .rev()
            .find(|&i| matches(&self.items[i].borrow()))
        {
            return Some(i);
        }

        // Wrap around from end to start
        ((start + 1)..self.items.len())
            .rev()
            .find(|&i| matches(&self.items[i].borrow()))
    }

    /// Process a menu request.
    pub fn driver(&mut self, req: i32) -> Result<()> {
        match req {
            REQ_NEXT_ITEM | REQ_DOWN_ITEM => {
                if self.current + 1 < self.items.len() {
                    self.current += 1;
                } else if !self.opts.contains(MenuOpts::O_NONCYCLIC) {
                    self.current = 0;
                }
                self.adjust_scroll();
            }
            REQ_PREV_ITEM | REQ_UP_ITEM => {
                if self.current > 0 {
                    self.current -= 1;
                } else if !self.opts.contains(MenuOpts::O_NONCYCLIC) {
                    self.current = self.items.len().saturating_sub(1);
                }
                self.adjust_scroll();
            }
            REQ_LEFT_ITEM => {
                // In multi-column layout, move left
                if self.cols > 1 {
                    let items_per_col = self.items.len().div_ceil(self.cols as usize);
                    if self.current >= items_per_col {
                        self.current -= items_per_col;
                        self.adjust_scroll();
                    }
                }
            }
            REQ_RIGHT_ITEM => {
                // In multi-column layout, move right
                if self.cols > 1 {
                    let items_per_col = self.items.len().div_ceil(self.cols as usize);
                    if self.current + items_per_col < self.items.len() {
                        self.current += items_per_col;
                        self.adjust_scroll();
                    }
                }
            }
            REQ_SCR_DLINE => {
                // Scroll down one line
                if self.top + (self.rows as usize) < self.items.len() {
                    self.top += 1;
                    if self.current < self.top {
                        self.current = self.top;
                    }
                }
            }
            REQ_SCR_ULINE => {
                // Scroll up one line
                if self.top > 0 {
                    self.top -= 1;
                    if self.current >= self.top + (self.rows as usize) {
                        self.current = self.top + (self.rows as usize) - 1;
                    }
                }
            }
            REQ_SCR_DPAGE => {
                // Scroll down one page
                let page_size = self.rows as usize;
                if self.top + page_size < self.items.len() {
                    self.top =
                        (self.top + page_size).min(self.items.len().saturating_sub(page_size));
                    if self.current < self.top {
                        self.current = self.top;
                    }
                }
            }
            REQ_SCR_UPAGE => {
                // Scroll up one page
                let page_size = self.rows as usize;
                if self.top > 0 {
                    self.top = self.top.saturating_sub(page_size);
                    if self.current >= self.top + page_size {
                        self.current = self.top + page_size - 1;
                    }
                }
            }
            REQ_FIRST_ITEM => {
                self.current = 0;
                self.adjust_scroll();
            }
            REQ_LAST_ITEM => {
                self.current = self.items.len().saturating_sub(1);
                self.adjust_scroll();
            }
            REQ_TOGGLE_ITEM => {
                if let Some(item) = self.items.get(self.current) {
                    let mut item = item.borrow_mut();
                    if item.is_selectable() {
                        item.selected = !item.selected;
                    }
                }
            }
            REQ_CLEAR_PATTERN => {
                self.pattern.clear();
            }
            REQ_BACK_PATTERN => {
                self.pattern.pop();
            }
            REQ_NEXT_MATCH => {
                // Find next matching item after current
                let start = if self.current + 1 < self.items.len() {
                    self.current + 1
                } else {
                    0
                };
                if let Some(idx) = self.find_match_forward(start) {
                    self.current = idx;
                    self.adjust_scroll();
                }
            }
            REQ_PREV_MATCH => {
                // Find previous matching item before current
                let start = if self.current > 0 {
                    self.current - 1
                } else {
                    self.items.len().saturating_sub(1)
                };
                if let Some(idx) = self.find_match_backward(start) {
                    self.current = idx;
                    self.adjust_scroll();
                }
            }
            _ => {
                // Handle printable characters for pattern matching
                if (0x20..0x7f).contains(&req) {
                    self.pattern.push(req as u8 as char);
                    // Find first matching item from current position
                    if let Some(idx) = self.find_match_forward(self.current) {
                        self.current = idx;
                        self.adjust_scroll();
                    }
                }
            }
        }

        // Re-render if posted
        if self.posted {
            self.render()?;
        }

        Ok(())
    }

    /// Render the menu to its window.
    pub fn render(&self) -> Result<()> {
        let win = if let Some(ref sub) = self.sub_window {
            sub.clone()
        } else if let Some(ref w) = self.window {
            w.clone()
        } else {
            return Ok(()); // No window to render to
        };

        let mut win = win.borrow_mut();
        win.erase()?;

        let maxy = win.getmaxy() as usize;
        let maxx = win.getmaxx() as usize;
        let show_desc = self.opts.contains(MenuOpts::O_SHOWDESC);

        // Calculate column width
        let col_width = if self.cols > 1 {
            maxx / self.cols as usize
        } else {
            maxx
        };

        // Pre-allocate reusable buffer for padding to avoid repeated allocations
        let mark_len = self.mark.len();
        let empty_mark: String = " ".repeat(mark_len);
        let spacing_str: String = " ".repeat(self.spacing);

        for row in 0..self.rows as usize {
            if row >= maxy {
                break;
            }

            let item_idx = self.top + row;
            if item_idx >= self.items.len() {
                break;
            }

            let item = self.items[item_idx].borrow();
            let is_current = item_idx == self.current;
            let is_selectable = item.is_selectable();

            // Determine attributes
            let attr = if !is_selectable {
                self.grey
            } else if is_current {
                self.fore
            } else {
                self.back
            };

            // Move to row position
            win.mv(row as i32, 0)?;
            win.attrset(attr)?;

            // Draw mark if selected (using pre-allocated empty mark)
            let mark_display = if item.selected {
                &self.mark
            } else {
                &empty_mark
            };
            win.addstr(mark_display)?;

            // Draw item name with padding (avoid allocation when possible)
            let name = &item.name;
            if name.len() >= self.name_width {
                win.addnstr(name, self.name_width as i32)?;
            } else {
                win.addstr(name)?;
                // Add padding using spaces
                for _ in 0..(self.name_width - name.len()) {
                    win.addch(b' ' as ChType)?;
                }
            }

            // Draw description if enabled
            if show_desc && self.desc_width > 0 {
                win.addstr(&spacing_str)?;
                let desc = &item.description;
                let available = col_width.saturating_sub(mark_len + self.name_width + self.spacing);
                if desc.len() > available {
                    win.addnstr(desc, available as i32)?;
                } else {
                    win.addstr(desc)?;
                }
            }

            // Clear to end of line using character output instead of string allocation
            let cur_x = win.getcurx() as usize;
            for _ in cur_x..col_width {
                win.addch(b' ' as ChType)?;
            }

            win.attrset(0)?;
        }

        Ok(())
    }

    /// Set user data.
    pub fn set_userptr<T: 'static>(&mut self, data: T) {
        self.user_data = Some(Box::new(data));
    }

    /// Get user data.
    pub fn userptr<T: 'static>(&self) -> Option<&T> {
        self.user_data.as_ref()?.downcast_ref::<T>()
    }

    /// Get all selected items.
    pub fn selected_items(&self) -> Vec<Rc<RefCell<MenuItem>>> {
        self.items
            .iter()
            .filter(|item| item.borrow().is_selected())
            .cloned()
            .collect()
    }

    /// Get the top item index (for scrolling).
    pub fn top_row(&self) -> usize {
        self.top
    }

    /// Set the top row (for scrolling).
    pub fn set_top_row(&mut self, row: usize) -> Result<()> {
        if row < self.items.len() {
            self.top = row;
            Ok(())
        } else {
            Err(Error::InvalidArgument("invalid row".into()))
        }
    }

    /// Get the item width for display.
    pub fn item_width(&self) -> usize {
        self.mark.len()
            + self.name_width
            + if self.opts.contains(MenuOpts::O_SHOWDESC) {
                self.spacing + self.desc_width
            } else {
                0
            }
    }

    /// Get the number of items per page.
    pub fn items_per_page(&self) -> usize {
        self.rows as usize
    }

    /// Set spacing between name and description.
    pub fn set_spacing(&mut self, spacing: usize) {
        self.spacing = spacing;
    }

    /// Get the window associated with the menu.
    pub fn window(&self) -> Option<std::cell::Ref<'_, Window>> {
        self.window.as_ref().map(|w| w.borrow())
    }

    /// Get the sub-window associated with the menu.
    pub fn sub_window(&self) -> Option<std::cell::Ref<'_, Window>> {
        self.sub_window.as_ref().map(|w| w.borrow())
    }

    // ========================================================================
    // Hook functions
    // ========================================================================

    /// Set the menu initialization hook.
    ///
    /// This function is called when the menu is posted or just after
    /// the top row changes.
    pub fn set_menu_init<F>(&mut self, hook: F)
    where
        F: Fn(&Menu) + 'static,
    {
        self.menu_init = Some(Box::new(hook));
    }

    /// Get a reference to the menu initialization hook.
    ///
    /// Returns `true` if a menu init hook is set.
    pub fn has_menu_init(&self) -> bool {
        self.menu_init.is_some()
    }

    /// Clear the menu initialization hook.
    pub fn clear_menu_init(&mut self) {
        self.menu_init = None;
    }

    /// Set the menu termination hook.
    ///
    /// This function is called when the menu is unposted or just before
    /// the top row changes.
    pub fn set_menu_term<F>(&mut self, hook: F)
    where
        F: Fn(&Menu) + 'static,
    {
        self.menu_term = Some(Box::new(hook));
    }

    /// Get a reference to the menu termination hook.
    ///
    /// Returns `true` if a menu term hook is set.
    pub fn has_menu_term(&self) -> bool {
        self.menu_term.is_some()
    }

    /// Clear the menu termination hook.
    pub fn clear_menu_term(&mut self) {
        self.menu_term = None;
    }

    /// Set the item initialization hook.
    ///
    /// This function is called when the menu is posted or just after
    /// the current item changes.
    pub fn set_item_init<F>(&mut self, hook: F)
    where
        F: Fn(&Menu) + 'static,
    {
        self.item_init = Some(Box::new(hook));
    }

    /// Get a reference to the item initialization hook.
    ///
    /// Returns `true` if an item init hook is set.
    pub fn has_item_init(&self) -> bool {
        self.item_init.is_some()
    }

    /// Clear the item initialization hook.
    pub fn clear_item_init(&mut self) {
        self.item_init = None;
    }

    /// Set the item termination hook.
    ///
    /// This function is called when the menu is unposted or just before
    /// the current item changes.
    pub fn set_item_term<F>(&mut self, hook: F)
    where
        F: Fn(&Menu) + 'static,
    {
        self.item_term = Some(Box::new(hook));
    }

    /// Get a reference to the item termination hook.
    ///
    /// Returns `true` if an item term hook is set.
    pub fn has_item_term(&self) -> bool {
        self.item_term.is_some()
    }

    /// Clear the item termination hook.
    pub fn clear_item_term(&mut self) {
        self.item_term = None;
    }

    /// Call the menu init hook if set.
    fn call_menu_init(&self) {
        if let Some(ref hook) = self.menu_init {
            hook(self);
        }
    }

    /// Call the menu term hook if set.
    fn call_menu_term(&self) {
        if let Some(ref hook) = self.menu_term {
            hook(self);
        }
    }

    /// Call the item init hook if set.
    fn call_item_init(&self) {
        if let Some(ref hook) = self.item_init {
            hook(self);
        }
    }

    /// Call the item term hook if set.
    fn call_item_term(&self) {
        if let Some(ref hook) = self.item_term {
            hook(self);
        }
    }
}

impl std::fmt::Debug for Menu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Menu")
            .field("item_count", &self.items.len())
            .field("current", &self.current)
            .field("top", &self.top)
            .field("opts", &self.opts)
            .field("posted", &self.posted)
            .field("pattern", &self.pattern)
            .finish()
    }
}

// ============================================================================
// Free functions for ncurses compatibility (Menu)
// ============================================================================

/// Create a new menu from an array of items.
///
/// This is the ncurses `new_menu()` function.
pub fn new_menu(items: Vec<MenuItem>) -> Menu {
    Menu::new(items)
}

/// Free a menu.
///
/// This is the ncurses `free_menu()` function.
/// In Rust, the menu is automatically freed when dropped.
pub fn free_menu(_menu: Menu) {
    // Menu is dropped automatically
}

/// Post a menu to its associated window.
///
/// This is the ncurses `post_menu()` function.
pub fn post_menu(menu: &mut Menu) -> Result<()> {
    menu.post()
}

/// Unpost a menu from its associated window.
///
/// This is the ncurses `unpost_menu()` function.
pub fn unpost_menu(menu: &mut Menu) -> Result<()> {
    menu.unpost()
}

/// Process menu input.
///
/// This is the ncurses `menu_driver()` function.
pub fn menu_driver(menu: &mut Menu, request: i32) -> Result<()> {
    menu.driver(request)
}

/// Set the menu window.
///
/// This is the ncurses `set_menu_win()` function.
pub fn set_menu_win(menu: &mut Menu, window: Window) {
    menu.set_window(window);
}

/// Get the menu window.
///
/// This is the ncurses `menu_win()` function.
pub fn menu_win(menu: &Menu) -> Option<std::cell::Ref<'_, Window>> {
    menu.window()
}

/// Set the menu sub-window.
///
/// This is the ncurses `set_menu_sub()` function.
pub fn set_menu_sub(menu: &mut Menu, window: Window) {
    menu.set_sub(window);
}

/// Get the menu sub-window.
///
/// This is the ncurses `menu_sub()` function.
pub fn menu_sub(menu: &Menu) -> Option<std::cell::Ref<'_, Window>> {
    menu.sub_window()
}

/// Set the menu format (rows and columns).
///
/// This is the ncurses `set_menu_format()` function.
pub fn set_menu_format(menu: &mut Menu, rows: i32, cols: i32) {
    menu.set_format(rows, cols);
}

/// Get the menu format.
///
/// This is the ncurses `menu_format()` function.
pub fn menu_format(menu: &Menu) -> (i32, i32) {
    menu.format()
}

/// Set the menu mark string.
///
/// This is the ncurses `set_menu_mark()` function.
pub fn set_menu_mark(menu: &mut Menu, mark: &str) {
    menu.set_mark(mark);
}

/// Get the menu mark string.
///
/// This is the ncurses `menu_mark()` function.
pub fn menu_mark(menu: &Menu) -> &str {
    menu.mark()
}

/// Set menu options.
///
/// This is the ncurses `set_menu_opts()` function.
pub fn set_menu_opts(menu: &mut Menu, opts: MenuOpts) {
    menu.set_opts(opts);
}

/// Get menu options.
///
/// This is the ncurses `menu_opts()` function.
pub fn menu_opts(menu: &Menu) -> MenuOpts {
    menu.opts()
}

/// Turn on menu options.
///
/// This is the ncurses `menu_opts_on()` function.
pub fn menu_opts_on(menu: &mut Menu, opts: MenuOpts) {
    menu.opts_on(opts);
}

/// Turn off menu options.
///
/// This is the ncurses `menu_opts_off()` function.
pub fn menu_opts_off(menu: &mut Menu, opts: MenuOpts) {
    menu.opts_off(opts);
}

/// Set the foreground attribute.
///
/// This is the ncurses `set_menu_fore()` function.
pub fn set_menu_fore(menu: &mut Menu, attr: AttrT) {
    menu.set_fore(attr);
}

/// Get the foreground attribute.
///
/// This is the ncurses `menu_fore()` function.
pub fn menu_fore(menu: &Menu) -> AttrT {
    menu.fore()
}

/// Set the background attribute.
///
/// This is the ncurses `set_menu_back()` function.
pub fn set_menu_back(menu: &mut Menu, attr: AttrT) {
    menu.set_back(attr);
}

/// Get the background attribute.
///
/// This is the ncurses `menu_back()` function.
pub fn menu_back(menu: &Menu) -> AttrT {
    menu.back()
}

/// Set the grey (non-selectable) attribute.
///
/// This is the ncurses `set_menu_grey()` function.
pub fn set_menu_grey(menu: &mut Menu, attr: AttrT) {
    menu.set_grey(attr);
}

/// Get the grey attribute.
///
/// This is the ncurses `menu_grey()` function.
pub fn menu_grey(menu: &Menu) -> AttrT {
    menu.grey()
}

/// Set the menu pattern.
///
/// This is the ncurses `set_menu_pattern()` function.
pub fn set_menu_pattern(menu: &mut Menu, pattern: &str) -> Result<()> {
    menu.set_pattern(pattern)
}

/// Get the menu pattern.
///
/// This is the ncurses `menu_pattern()` function.
pub fn menu_pattern(menu: &Menu) -> &str {
    menu.pattern()
}

/// Get the current item.
///
/// This is the ncurses `current_item()` function.
pub fn current_item(menu: &Menu) -> Option<Rc<RefCell<MenuItem>>> {
    menu.current_item()
}

/// Set the current item.
///
/// This is the ncurses `set_current_item()` function.
pub fn set_current_item(menu: &mut Menu, index: usize) -> Result<()> {
    menu.set_current_item(index)
}

/// Get the top row.
///
/// This is the ncurses `top_row()` function.
pub fn top_row(menu: &Menu) -> usize {
    menu.top_row()
}

/// Set the top row.
///
/// This is the ncurses `set_top_row()` function.
pub fn set_top_row(menu: &mut Menu, row: usize) {
    let _ = menu.set_top_row(row);
}

/// Get the item count.
///
/// This is the ncurses `item_count()` function.
pub fn item_count(menu: &Menu) -> usize {
    menu.item_count()
}

/// Get the menu items.
///
/// This is the ncurses `menu_items()` function.
/// Returns a slice of all items in the menu.
pub fn menu_items(menu: &Menu) -> &[Rc<RefCell<MenuItem>>] {
    menu.items()
}

/// Set the menu items.
///
/// This is the ncurses `set_menu_items()` function.
/// Replaces all items in the menu. The menu must not be posted.
///
/// # Arguments
///
/// * `menu` - The menu to modify
/// * `items` - The new items for the menu
///
/// # Returns
///
/// `Ok(())` on success, or an error if the menu is currently posted.
pub fn set_menu_items(menu: &mut Menu, items: Vec<MenuItem>) -> Result<()> {
    menu.set_items(items)
}

/// Set menu user pointer.
///
/// This is the ncurses `set_menu_userptr()` function.
pub fn set_menu_userptr<T: 'static>(menu: &mut Menu, data: T) {
    menu.set_userptr(data);
}

/// Get menu user pointer.
///
/// This is the ncurses `menu_userptr()` function.
pub fn menu_userptr<T: 'static>(menu: &Menu) -> Option<&T> {
    menu.userptr::<T>()
}

/// Set the menu spacing.
///
/// This is the ncurses `set_menu_spacing()` function.
pub fn set_menu_spacing(menu: &mut Menu, desc: i32, _rows: i32, _cols: i32) {
    menu.set_spacing(desc as usize);
}

/// Get the menu spacing.
///
/// This is the ncurses `menu_spacing()` function.
/// Returns (description spacing, row spacing, column spacing).
pub fn menu_spacing(menu: &Menu) -> (i32, i32, i32) {
    let spacing = menu.item_width() as i32;
    (spacing, 1, 1)
}

/// Calculate the scale (size) required for the menu.
///
/// This is the ncurses `scale_menu()` function.
/// Returns (rows, cols) needed to display the menu.
pub fn scale_menu(menu: &Menu) -> (i32, i32) {
    let (rows, cols) = menu.format();
    let item_width = menu.item_width() as i32;
    (rows, cols * item_width)
}

/// Get the number of items per page.
///
/// This is derived from menu format.
pub fn menu_items_per_page(menu: &Menu) -> usize {
    menu.items_per_page()
}

// ============================================================================
// Free functions for ncurses compatibility (MenuItem)
// ============================================================================

/// Create a new menu item.
///
/// This is the ncurses `new_item()` function.
pub fn new_item(name: &str, description: &str) -> MenuItem {
    MenuItem::new(name, description)
}

/// Free a menu item.
///
/// This is the ncurses `free_item()` function.
/// In Rust, items are automatically freed when dropped.
pub fn free_item(_item: MenuItem) {
    // Item is dropped automatically
}

/// Get the item name.
///
/// This is the ncurses `item_name()` function.
pub fn item_name(item: &MenuItem) -> &str {
    item.name()
}

/// Get the item description.
///
/// This is the ncurses `item_description()` function.
pub fn item_description(item: &MenuItem) -> &str {
    item.description()
}

/// Get the item index.
///
/// This is the ncurses `item_index()` function.
pub fn item_index(item: &MenuItem) -> usize {
    item.index()
}

/// Set item options.
///
/// This is the ncurses `set_item_opts()` function.
pub fn set_item_opts(item: &mut MenuItem, opts: ItemOpts) {
    item.set_opts(opts);
}

/// Get item options.
///
/// This is the ncurses `item_opts()` function.
pub fn item_opts(item: &MenuItem) -> ItemOpts {
    item.opts()
}

/// Turn on item options.
///
/// This is the ncurses `item_opts_on()` function.
pub fn item_opts_on(item: &mut MenuItem, opts: ItemOpts) {
    let current = item.opts();
    item.set_opts(current | opts);
}

/// Turn off item options.
///
/// This is the ncurses `item_opts_off()` function.
pub fn item_opts_off(item: &mut MenuItem, opts: ItemOpts) {
    let current = item.opts();
    item.set_opts(current & !opts);
}

/// Check if item is selectable.
///
/// This is derived from item options.
pub fn item_visible(item: &MenuItem) -> bool {
    item.is_selectable()
}

/// Get the item value (selection state).
///
/// This is the ncurses `item_value()` function.
pub fn item_value(item: &MenuItem) -> bool {
    item.is_selected()
}

/// Set item user pointer.
///
/// This is the ncurses `set_item_userptr()` function.
pub fn set_item_userptr<T: 'static>(item: &mut MenuItem, data: T) {
    item.set_userptr(data);
}

/// Get item user pointer.
///
/// This is the ncurses `item_userptr()` function.
pub fn item_userptr<T: 'static>(item: &MenuItem) -> Option<&T> {
    item.userptr::<T>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_item() {
        let item = MenuItem::new("Option 1", "First option");
        assert_eq!(item.name(), "Option 1");
        assert_eq!(item.description(), "First option");
        assert!(item.is_selectable());
        assert!(!item.is_selected());
    }

    #[test]
    fn test_menu() {
        let items = vec![
            MenuItem::new("Option 1", "First"),
            MenuItem::new("Option 2", "Second"),
            MenuItem::new("Option 3", "Third"),
        ];
        let mut menu = Menu::new(items);

        assert_eq!(menu.item_count(), 3);
        assert_eq!(menu.current_item_index(), 0);

        menu.driver(REQ_NEXT_ITEM).unwrap();
        assert_eq!(menu.current_item_index(), 1);

        menu.driver(REQ_LAST_ITEM).unwrap();
        assert_eq!(menu.current_item_index(), 2);

        menu.driver(REQ_FIRST_ITEM).unwrap();
        assert_eq!(menu.current_item_index(), 0);
    }

    #[test]
    fn test_menu_format() {
        let mut menu = Menu::new(vec![]);
        menu.set_format(10, 2);
        assert_eq!(menu.format(), (10, 2));
    }

    #[test]
    fn test_menu_pattern_matching() {
        let items = vec![
            MenuItem::new("Apple", "Fruit"),
            MenuItem::new("Banana", "Fruit"),
            MenuItem::new("Apricot", "Fruit"),
            MenuItem::new("Cherry", "Fruit"),
        ];
        let mut menu = Menu::new(items);

        // Type 'A' to search
        menu.driver('A' as i32).unwrap();
        assert_eq!(menu.current_item_index(), 0); // Apple

        // Type 'p' to continue search
        menu.driver('p' as i32).unwrap();
        assert_eq!(menu.current_item_index(), 0); // Still Apple (matches "Ap")

        // Next match for "Ap"
        menu.driver(REQ_NEXT_MATCH).unwrap();
        assert_eq!(menu.current_item_index(), 2); // Apricot

        // Clear pattern and search for 'B'
        menu.driver(REQ_CLEAR_PATTERN).unwrap();
        menu.driver('B' as i32).unwrap();
        assert_eq!(menu.current_item_index(), 1); // Banana
    }

    #[test]
    fn test_menu_scrolling() {
        let items: Vec<_> = (0..20)
            .map(|i| MenuItem::new(&format!("Item {}", i), ""))
            .collect();
        let mut menu = Menu::new(items);
        menu.set_format(5, 1);

        // Initial state
        assert_eq!(menu.top_row(), 0);
        assert_eq!(menu.current_item_index(), 0);

        // Move to item 10
        for _ in 0..10 {
            menu.driver(REQ_NEXT_ITEM).unwrap();
        }
        assert_eq!(menu.current_item_index(), 10);
        // Top should have scrolled
        assert!(menu.top_row() > 0);

        // Scroll down a page
        menu.driver(REQ_SCR_DPAGE).unwrap();
        assert!(menu.top_row() > 5);
    }
}
