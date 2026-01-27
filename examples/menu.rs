//! Menu example demonstrating menu creation and navigation.
//!
//! This example shows how to use the menu library to create
//! interactive selection menus.
//!
//! Features demonstrated:
//! - Creating menu items
//! - Building menus
//! - Navigating with arrow keys
//! - Selecting items

use ncurses::*;

#[cfg(feature = "menu")]
fn main() -> Result<()> {
    use ncurses::menu::*;

    let mut screen = Screen::init()?;

    // Initialize colors
    if screen.has_colors() {
        screen.start_color()?;
        screen.init_pair(1, COLOR_WHITE, COLOR_BLUE)?; // Header
        screen.init_pair(2, COLOR_BLACK, COLOR_CYAN)?; // Selected item
        screen.init_pair(3, COLOR_WHITE, COLOR_BLACK)?; // Normal item
        screen.init_pair(4, COLOR_YELLOW, COLOR_BLACK)?; // Status
    }

    screen.keypad(true);
    screen.noecho()?;
    screen.cbreak()?;

    let max_y = screen.lines();
    let max_x = screen.cols();

    // Create menu items
    let items = vec![
        MenuItem::new("New File", "Create a new file"),
        MenuItem::new("Open File", "Open an existing file"),
        MenuItem::new("Save", "Save the current file"),
        MenuItem::new("Save As", "Save with a new name"),
        MenuItem::new("Export", "Export to different format"),
        MenuItem::new("Print", "Print the document"),
        MenuItem::new("Settings", "Edit preferences"),
        MenuItem::new("Help", "Show help information"),
        MenuItem::new("About", "About this application"),
        MenuItem::new("Exit", "Quit the application"),
    ];

    // Create menu
    let mut menu = Menu::new(items);
    menu.set_format(10, 1); // 10 rows, 1 column

    // Set attributes
    if screen.has_colors() {
        menu.set_fore(attr::color_pair(2) | attr::A_BOLD);
        menu.set_back(attr::color_pair(3));
    }

    // Create window for the menu
    let menu_height = 14;
    let menu_width = 40;
    let menu_y = (max_y - menu_height) / 2;
    let menu_x = (max_x - menu_width) / 2;

    let mut menu_win = screen.newwin(menu_height, menu_width, menu_y, menu_x)?;

    // Draw interface
    draw_title(&mut screen)?;
    draw_instructions(&mut screen, max_y)?;

    // Post menu
    menu.post()?;

    screen.refresh()?;

    // Main loop
    loop {
        // Draw the menu
        draw_menu(&mut screen, &mut menu_win, &menu)?;
        screen.wrefresh(&mut menu_win)?;

        // Draw status showing current selection
        draw_status(&mut screen, max_y, &menu)?;

        let ch = screen.getch()?;

        match ch {
            // Arrow keys for navigation
            k if k == key::KEY_UP => {
                menu.driver(REQ_PREV_ITEM)?;
            }
            k if k == key::KEY_DOWN => {
                menu.driver(REQ_NEXT_ITEM)?;
            }

            // Home/End for first/last
            k if k == key::KEY_HOME => {
                menu.driver(REQ_FIRST_ITEM)?;
            }
            k if k == key::KEY_END => {
                menu.driver(REQ_LAST_ITEM)?;
            }

            // Page up/down (jump by 5)
            k if k == key::KEY_PPAGE => {
                for _ in 0..5 {
                    menu.driver(REQ_PREV_ITEM)?;
                }
            }
            k if k == key::KEY_NPAGE => {
                for _ in 0..5 {
                    menu.driver(REQ_NEXT_ITEM)?;
                }
            }

            // Enter to select
            k if k == 10 || k == 13 || k == key::KEY_ENTER => {
                if let Some(item) = menu.current_item() {
                    let item = item.borrow();
                    if item.name() == "Exit" {
                        break;
                    }
                    // Show selection
                    show_selection(&mut screen, max_y, &item)?;
                }
            }

            // q to quit
            k if k == b'q' as i32 || k == b'Q' as i32 => {
                break;
            }

            // Pass printable characters to pattern matching
            k if k >= 0x20 && k < 0x7f => {
                menu.driver(k)?;
            }

            _ => {}
        }
    }

    // Unpost menu
    menu.unpost()?;

    Ok(())
}

#[cfg(feature = "menu")]
fn draw_title(screen: &mut Screen) -> Result<()> {
    screen.clear()?;

    if screen.has_colors() {
        screen.attron(attr::color_pair(1))?;
    }
    screen.attron(attr::A_BOLD)?;

    let title = " ncurses-pure Menu Demo ";
    let x = (screen.cols() - title.len() as i32) / 2;
    screen.mvaddstr(1, x, title)?;

    screen.attroff(attr::A_BOLD)?;
    if screen.has_colors() {
        screen.attroff(attr::color_pair(1))?;
    }

    Ok(())
}

#[cfg(feature = "menu")]
fn draw_instructions(screen: &mut Screen, max_y: i32) -> Result<()> {
    if screen.has_colors() {
        screen.attron(attr::color_pair(4))?;
    }

    screen.mvaddstr(
        max_y - 3,
        2,
        "Navigation: UP/DOWN arrows, HOME/END, PGUP/PGDN",
    )?;
    screen.mvaddstr(max_y - 2, 2, "Select: ENTER | Type to search | Quit: 'q'")?;

    if screen.has_colors() {
        screen.attroff(attr::color_pair(4))?;
    }

    Ok(())
}

#[cfg(feature = "menu")]
fn draw_menu(screen: &mut Screen, menu_win: &mut Window, menu: &menu::Menu) -> Result<()> {
    menu_win.erase()?;

    // Draw border
    menu_win.box_(0, 0)?;

    // Draw title
    menu_win.attron(attr::A_BOLD)?;
    menu_win.mvaddstr(0, 2, " File Menu ")?;
    menu_win.attroff(attr::A_BOLD)?;

    // Draw items
    let current_idx = menu.current_item_index();
    for i in 0..menu.item_count() {
        let y = (i + 2) as i32;
        let is_current = i == current_idx;

        if let Some(item_rc) = menu.item(i) {
            let item = item_rc.borrow();

            if is_current {
                menu_win.attron(menu.fore())?;
                // Fill the line with background
                menu_win.mv(y, 1)?;
                for _ in 0..(menu_win.getmaxx() - 2) {
                    menu_win.addch(b' ' as ChType)?;
                }
            } else {
                menu_win.attron(menu.back())?;
            }

            // Draw item name
            menu_win.mvaddstr(y, 2, &format!(" {} ", item.name()))?;

            // Draw description on the right
            let desc = item.description();
            let desc_x = menu_win.getmaxx() - desc.len() as i32 - 3;
            if desc_x > item.name().len() as i32 + 5 {
                menu_win.mvaddstr(y, desc_x, desc)?;
            }

            if is_current {
                menu_win.attroff(menu.fore())?;
            } else {
                menu_win.attroff(menu.back())?;
            }
        }
    }

    // Show pattern if any
    let pattern = menu.pattern();
    if !pattern.is_empty() {
        menu_win.mvaddstr(menu_win.getmaxy() - 1, 2, &format!("Search: {}", pattern))?;
    }

    screen.wrefresh(menu_win)?;

    Ok(())
}

#[cfg(feature = "menu")]
fn draw_status(screen: &mut Screen, max_y: i32, menu: &menu::Menu) -> Result<()> {
    screen.mv(max_y - 5, 0)?;
    screen.clrtoeol()?;

    if let Some(item_rc) = menu.current_item() {
        let item = item_rc.borrow();
        screen.mvaddstr(
            max_y - 5,
            2,
            &format!("Current: {} - {}", item.name(), item.description()),
        )?;
    }

    screen.refresh()?;
    Ok(())
}

#[cfg(feature = "menu")]
fn show_selection(screen: &mut Screen, max_y: i32, item: &menu::MenuItem) -> Result<()> {
    if screen.has_colors() {
        screen.attron(attr::color_pair(4))?;
    }
    screen.attron(attr::A_BOLD)?;

    screen.mv(max_y - 4, 0)?;
    screen.clrtoeol()?;
    screen.mvaddstr(max_y - 4, 2, &format!(">>> Selected: {} <<<", item.name()))?;

    screen.attroff(attr::A_BOLD)?;
    if screen.has_colors() {
        screen.attroff(attr::color_pair(4))?;
    }

    screen.refresh()?;

    // Brief pause to show selection
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Clear the message
    screen.mv(max_y - 4, 0)?;
    screen.clrtoeol()?;
    screen.refresh()?;

    Ok(())
}

#[cfg(not(feature = "menu"))]
fn main() {
    eprintln!("This example requires the 'menu' feature.");
    eprintln!("Run with: cargo run --example menu --features menu");
}
