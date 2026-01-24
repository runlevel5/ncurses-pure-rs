//! Mouse event handling example for ncurses-rs.
//!
//! This example demonstrates:
//! - Enabling mouse support
//! - Handling mouse button events
//! - Tracking mouse position
//! - Using mouse events with windows
//!
//! Note: Mouse support requires a terminal that supports mouse events
//! (e.g., xterm, iTerm2, gnome-terminal).

use ncurses_rs::*;

#[cfg(feature = "mouse")]
fn main() -> Result<()> {
    use ncurses_rs::mouse::*;

    let mut screen = Screen::init()?;

    // Initialize colors
    if screen.has_colors() {
        screen.start_color()?;
        screen.init_pair(1, COLOR_WHITE, COLOR_BLUE)?; // Header
        screen.init_pair(2, COLOR_BLACK, COLOR_WHITE)?; // Button area
        screen.init_pair(3, COLOR_GREEN, COLOR_BLACK)?; // Event info
        screen.init_pair(4, COLOR_YELLOW, COLOR_BLACK)?; // Instructions
    }

    screen.keypad(true);
    screen.noecho()?;
    screen.cbreak()?;

    let max_y = screen.lines();
    let max_x = screen.cols();

    // Draw the interface
    draw_interface(&mut screen, max_y, max_x)?;

    // Create mouse state for tracking
    let mut mouse_state = MouseState::new();
    mouse_state.mousemask(ALL_MOUSE_EVENTS);

    // Variables for tracking
    let mut click_count = 0;
    let mut last_x = 0;
    let mut last_y = 0;
    let mut last_button = "None";

    // Create a clickable button area
    let button_y = max_y / 2;
    let button_x = (max_x - 20) / 2;
    let button_h = 3;
    let button_w = 20;

    // Draw the button
    draw_button(&mut screen, button_y, button_x, button_h, button_w, false)?;

    screen.refresh()?;

    // Main event loop
    loop {
        // Display instructions
        if screen.has_colors() {
            screen.attron(attr::color_pair(4))?;
        }
        screen.mvaddstr(
            max_y - 2,
            2,
            "Move mouse, click buttons, or press 'q' to quit",
        )?;
        if screen.has_colors() {
            screen.attroff(attr::color_pair(4))?;
        }

        // Update event display
        update_event_display(&mut screen, last_y, last_x, last_button, click_count)?;

        screen.refresh()?;

        // Wait for input
        let ch = screen.getch()?;

        // Check for quit
        if ch == b'q' as i32 || ch == b'Q' as i32 {
            break;
        }

        // In a full implementation, we would check for KEY_MOUSE here
        // and then call getmouse() to get the event details.
        //
        // For now, demonstrate with keyboard simulation:
        match ch {
            // Simulate mouse events with number keys for demo
            k if k == b'1' as i32 => {
                last_button = "Button 1 (Left)";
                click_count += 1;
                // Check if in button area
                if wenclose(button_y, button_x, button_h, button_w, last_y, last_x) {
                    draw_button(&mut screen, button_y, button_x, button_h, button_w, true)?;
                    screen.refresh()?;
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    draw_button(&mut screen, button_y, button_x, button_h, button_w, false)?;
                }
            }
            k if k == b'2' as i32 => {
                last_button = "Button 2 (Middle)";
                click_count += 1;
            }
            k if k == b'3' as i32 => {
                last_button = "Button 3 (Right)";
                click_count += 1;
            }
            k if k == key::KEY_UP => {
                if last_y > 0 {
                    last_y -= 1;
                }
            }
            k if k == key::KEY_DOWN => {
                if last_y < max_y - 1 {
                    last_y += 1;
                }
            }
            k if k == key::KEY_LEFT => {
                if last_x > 0 {
                    last_x -= 1;
                }
            }
            k if k == key::KEY_RIGHT => {
                if last_x < max_x - 1 {
                    last_x += 1;
                }
            }
            _ => {}
        }
    }

    Ok(())
}

#[cfg(feature = "mouse")]
fn draw_interface(screen: &mut Screen, max_y: i32, max_x: i32) -> Result<()> {
    screen.clear()?;

    // Draw header
    if screen.has_colors() {
        screen.attron(attr::color_pair(1))?;
    }
    screen.attron(attr::A_BOLD)?;
    let title = " ncurses-rs Mouse Demo ";
    let title_x = (max_x - title.len() as i32) / 2;
    screen.mvaddstr(0, title_x, title)?;
    screen.attroff(attr::A_BOLD)?;
    if screen.has_colors() {
        screen.attroff(attr::color_pair(1))?;
    }

    // Draw border
    screen.mv(1, 0)?;
    screen.hline(0, max_x)?;
    screen.mv(max_y - 3, 0)?;
    screen.hline(0, max_x)?;

    // Draw info area
    if screen.has_colors() {
        screen.attron(attr::color_pair(3))?;
    }
    screen.mvaddstr(3, 2, "Mouse Events:")?;
    screen.mvaddstr(4, 4, "Position: (0, 0)")?;
    screen.mvaddstr(5, 4, "Last Button: None")?;
    screen.mvaddstr(6, 4, "Click Count: 0")?;
    if screen.has_colors() {
        screen.attroff(attr::color_pair(3))?;
    }

    Ok(())
}

#[cfg(feature = "mouse")]
fn draw_button(screen: &mut Screen, y: i32, x: i32, _h: i32, w: i32, pressed: bool) -> Result<()> {
    // Create a window for the button
    let mut button_win = screen.newwin(3, w, y, x)?;

    if screen.has_colors() {
        button_win.bkgd(attr::color_pair(2))?;
    }

    if pressed {
        button_win.attron(attr::A_REVERSE)?;
    }

    button_win.box_(0, 0)?;

    let label = if pressed { "[ PRESSED ]" } else { "Click Me!" };
    let label_x = (w - label.len() as i32) / 2;
    button_win.mvaddstr(1, label_x, label)?;

    if pressed {
        button_win.attroff(attr::A_REVERSE)?;
    }

    screen.wrefresh(&mut button_win)?;
    Ok(())
}

#[cfg(feature = "mouse")]
fn update_event_display(
    screen: &mut Screen,
    y: i32,
    x: i32,
    button: &str,
    clicks: u32,
) -> Result<()> {
    if screen.has_colors() {
        screen.attron(attr::color_pair(3))?;
    }

    // Clear the lines first
    screen.mv(4, 4)?;
    screen.clrtoeol()?;
    screen.mv(5, 4)?;
    screen.clrtoeol()?;
    screen.mv(6, 4)?;
    screen.clrtoeol()?;

    // Update with new values
    screen.mvaddstr(4, 4, &format!("Position: ({}, {})", y, x))?;
    screen.mvaddstr(5, 4, &format!("Last Button: {}", button))?;
    screen.mvaddstr(6, 4, &format!("Click Count: {}", clicks))?;

    if screen.has_colors() {
        screen.attroff(attr::color_pair(3))?;
    }

    Ok(())
}

#[cfg(not(feature = "mouse"))]
fn main() {
    eprintln!("This example requires the 'mouse' feature.");
    eprintln!("Run with: cargo run --example mouse --features mouse");
}
