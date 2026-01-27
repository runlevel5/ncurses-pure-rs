//! Panels example demonstrating overlapping window management.
//!
//! This example shows how to use the panels library to manage
//! multiple overlapping windows as a deck of cards.
//!
//! Features demonstrated:
//! - Creating panels from windows
//! - Managing panel stacking order (top, bottom)
//! - Showing and hiding panels
//! - Updating panels for display

use ncurses::*;

#[cfg(feature = "panels")]
fn main() -> Result<()> {
    use ncurses::panels::*;

    let mut screen = Screen::init()?;

    // Initialize colors
    if screen.has_colors() {
        screen.start_color()?;
        screen.init_pair(1, COLOR_WHITE, COLOR_RED)?;
        screen.init_pair(2, COLOR_WHITE, COLOR_GREEN)?;
        screen.init_pair(3, COLOR_WHITE, COLOR_BLUE)?;
        screen.init_pair(4, COLOR_BLACK, COLOR_YELLOW)?;
    }

    screen.keypad(true);
    screen.noecho()?;
    screen.cbreak()?;

    // Get screen dimensions
    let max_y = screen.lines();
    let max_x = screen.cols();

    // Create three overlapping windows
    let panel_height = 10;
    let panel_width = 30;

    let win1 = screen.newwin(panel_height, panel_width, 2, 5)?;
    let win2 = screen.newwin(panel_height, panel_width, 5, 15)?;
    let win3 = screen.newwin(panel_height, panel_width, 8, 25)?;

    // Create panel deck and add windows as panels
    let mut deck = PanelDeck::new();
    let panel1 = deck.new_panel(win1);
    let panel2 = deck.new_panel(win2);
    let panel3 = deck.new_panel(win3);

    // Set user data for identification
    panel1.borrow_mut().set_userptr("Panel 1 (Red)");
    panel2.borrow_mut().set_userptr("Panel 2 (Green)");
    panel3.borrow_mut().set_userptr("Panel 3 (Blue)");

    // Draw initial content on each panel
    draw_panel(&panel1, 1)?;
    draw_panel(&panel2, 2)?;
    draw_panel(&panel3, 3)?;

    // Track which panel is currently "active"
    let mut active = 3;

    // Draw status bar
    draw_status(&mut screen, max_y, max_x, active)?;

    // Initial refresh
    update_all(&mut screen, &deck)?;

    // Main loop
    loop {
        let ch = screen.getch()?;

        match ch {
            // Tab to cycle through panels
            k if k == b'\t' as i32 => {
                active = (active % 3) + 1;
                match active {
                    1 => deck.top_panel(&panel1)?,
                    2 => deck.top_panel(&panel2)?,
                    3 => deck.top_panel(&panel3)?,
                    _ => {}
                }
                draw_status(&mut screen, max_y, max_x, active)?;
            }

            // 1, 2, 3 to bring specific panel to top
            k if k == b'1' as i32 => {
                active = 1;
                deck.top_panel(&panel1)?;
                draw_status(&mut screen, max_y, max_x, active)?;
            }
            k if k == b'2' as i32 => {
                active = 2;
                deck.top_panel(&panel2)?;
                draw_status(&mut screen, max_y, max_x, active)?;
            }
            k if k == b'3' as i32 => {
                active = 3;
                deck.top_panel(&panel3)?;
                draw_status(&mut screen, max_y, max_x, active)?;
            }

            // b to send active panel to bottom
            k if k == b'b' as i32 || k == b'B' as i32 => match active {
                1 => deck.bottom_panel(&panel1)?,
                2 => deck.bottom_panel(&panel2)?,
                3 => deck.bottom_panel(&panel3)?,
                _ => {}
            },

            // h to hide active panel
            k if k == b'h' as i32 || k == b'H' as i32 => match active {
                1 => deck.hide_panel(&panel1)?,
                2 => deck.hide_panel(&panel2)?,
                3 => deck.hide_panel(&panel3)?,
                _ => {}
            },

            // s to show all panels
            k if k == b's' as i32 || k == b'S' as i32 => {
                deck.show_panel(&panel1)?;
                deck.show_panel(&panel2)?;
                deck.show_panel(&panel3)?;
            }

            // q to quit
            k if k == b'q' as i32 || k == b'Q' as i32 => {
                break;
            }

            _ => {}
        }

        // Update display
        update_all(&mut screen, &deck)?;
    }

    Ok(())
}

#[cfg(feature = "panels")]
fn draw_panel(panel: &std::rc::Rc<std::cell::RefCell<panels::Panel>>, num: i32) -> Result<()> {
    let p = panel.borrow();
    let mut win = p.window_mut();

    // Set color
    if num >= 1 && num <= 4 {
        win.bkgd(attr::color_pair(num as i16))?;
    }

    win.erase()?;
    win.box_(0, 0)?;

    // Title
    win.attron(attr::A_BOLD)?;
    let title = format!(" Panel {} ", num);
    let x = (win.getmaxx() - title.len() as i32) / 2;
    win.mvaddstr(0, x, &title)?;
    win.attroff(attr::A_BOLD)?;

    // Content
    win.mvaddstr(2, 2, &format!("This is panel {}", num))?;
    win.mvaddstr(4, 2, "Press 1,2,3 to select")?;
    win.mvaddstr(5, 2, "Press TAB to cycle")?;
    win.mvaddstr(6, 2, "Press 'b' for bottom")?;
    win.mvaddstr(7, 2, "Press 'h' to hide")?;
    win.mvaddstr(8, 2, "Press 's' to show all")?;

    Ok(())
}

#[cfg(feature = "panels")]
fn draw_status(screen: &mut Screen, max_y: i32, max_x: i32, active: i32) -> Result<()> {
    // Clear status line
    screen.mv(max_y - 1, 0)?;
    screen.clrtoeol()?;

    // Draw status
    if screen.has_colors() {
        screen.attron(attr::color_pair(4))?;
    }
    screen.attron(attr::A_BOLD)?;
    screen.mvaddstr(
        max_y - 1,
        0,
        &format!(
            " Active: Panel {} | TAB=cycle, 1-3=select, b=bottom, h=hide, s=show, q=quit ",
            active
        ),
    )?;
    // Pad to full width
    let current_x = screen.getcurx();
    if current_x < max_x {
        for _ in current_x..max_x {
            screen.addch(b' ' as ChType)?;
        }
    }
    screen.attroff(attr::A_BOLD)?;
    if screen.has_colors() {
        screen.attroff(attr::color_pair(4))?;
    }

    Ok(())
}

#[cfg(feature = "panels")]
fn update_all(screen: &mut Screen, deck: &panels::PanelDeck) -> Result<()> {
    // Clear background
    screen.clear()?;

    // Draw title
    screen.attron(attr::A_BOLD)?;
    screen.mvaddstr(0, 2, "ncurses-pure Panels Demo")?;
    screen.attroff(attr::A_BOLD)?;

    screen.refresh()?;

    // Update panels - this marks them for refresh
    deck.update_panels();

    // Refresh each visible panel in order
    for panel_rc in deck.iter() {
        let panel = panel_rc.borrow();
        if panel.is_visible() {
            // Get a mutable reference to window and refresh it
            let mut win = panel.window_mut();
            screen.wrefresh(&mut *win)?;
        }
    }

    Ok(())
}

#[cfg(not(feature = "panels"))]
fn main() {
    eprintln!("This example requires the 'panels' feature.");
    eprintln!("Run with: cargo run --example panels --features panels");
}
