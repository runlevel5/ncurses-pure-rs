//! Dialog box example demonstrating popup windows and user interaction.
//!
//! This example shows how to create centered dialog boxes with
//! buttons, shadows, and user input handling.
//!
//! Features demonstrated:
//! - Creating centered popup windows
//! - Drawing shadows
//! - Button navigation
//! - Modal dialogs
//! - Keyboard handling

use ncurses_rs::*;

fn main() -> Result<()> {
    let mut screen = Screen::init()?;

    // Initialize colors
    if screen.has_colors() {
        screen.start_color()?;
        screen.init_pair(1, COLOR_WHITE, COLOR_BLUE)?; // Dialog background
        screen.init_pair(2, COLOR_BLACK, COLOR_WHITE)?; // Button normal
        screen.init_pair(3, COLOR_WHITE, COLOR_RED)?; // Button selected
        screen.init_pair(4, COLOR_BLACK, COLOR_BLACK)?; // Shadow
        screen.init_pair(5, COLOR_YELLOW, COLOR_BLUE)?; // Dialog title
        screen.init_pair(6, COLOR_WHITE, COLOR_GREEN)?; // Success message
    }

    screen.keypad(true);
    screen.noecho()?;
    screen.cbreak()?;
    screen.curs_set(CursorVisibility::Invisible)?;

    // Draw main screen background
    draw_background(&mut screen)?;
    screen.refresh()?;

    // Show a series of dialogs
    let name = show_input_dialog(&mut screen, "Welcome", "Please enter your name:", 20)?;

    if name.is_empty() {
        show_message_dialog(
            &mut screen,
            "Notice",
            "No name entered. Using 'Guest'.",
            &["OK"],
        )?;
    }

    let display_name = if name.is_empty() {
        "Guest".to_string()
    } else {
        name
    };

    // Confirmation dialog
    let choice = show_message_dialog(
        &mut screen,
        "Confirm",
        &format!("Hello, {}! Would you like to continue?", display_name),
        &["Yes", "No", "Cancel"],
    )?;

    let message = match choice {
        0 => format!("Welcome aboard, {}!", display_name),
        1 => "Maybe next time!".to_string(),
        _ => "Action cancelled.".to_string(),
    };

    // Show result
    show_message_dialog(&mut screen, "Result", &message, &["OK"])?;

    // Final goodbye
    if screen.has_colors() {
        screen.attron(attr::color_pair(6))?;
    }
    let goodbye = " Thanks for trying ncurses-rs! Press any key to exit. ";
    let x = (screen.cols() - goodbye.len() as i32) / 2;
    screen.mvaddstr(screen.lines() - 2, x, goodbye)?;
    if screen.has_colors() {
        screen.attroff(attr::color_pair(6))?;
    }
    screen.refresh()?;
    screen.getch()?;

    Ok(())
}

fn draw_background(screen: &mut Screen) -> Result<()> {
    let max_y = screen.lines();
    let max_x = screen.cols();

    // Fill with a pattern
    for y in 0..max_y {
        for x in 0..max_x {
            let ch = if (x + y) % 2 == 0 { '.' } else { ' ' };
            screen.mvaddch(y, x, ch as ChType)?;
        }
    }

    // Title bar
    if screen.has_colors() {
        screen.attron(attr::color_pair(1))?;
    }
    screen.mv(0, 0)?;
    for _ in 0..max_x {
        screen.addch(b' ' as ChType)?;
    }
    let title = " Dialog Demo - ncurses-rs ";
    screen.mvaddstr(0, (max_x - title.len() as i32) / 2, title)?;
    if screen.has_colors() {
        screen.attroff(attr::color_pair(1))?;
    }

    Ok(())
}

/// Show a message dialog with multiple buttons.
/// Returns the index of the selected button.
fn show_message_dialog(
    screen: &mut Screen,
    title: &str,
    message: &str,
    buttons: &[&str],
) -> Result<usize> {
    let screen_h = screen.lines();
    let screen_w = screen.cols();

    // Calculate dialog size
    let button_width: i32 =
        buttons.iter().map(|b| b.len() as i32 + 4).sum::<i32>() + (buttons.len() as i32 - 1) * 2;
    let dialog_w = (message.len() as i32 + 4)
        .max(title.len() as i32 + 4)
        .max(button_width + 4);
    let dialog_h = 7;

    // Center the dialog
    let dialog_y = (screen_h - dialog_h) / 2;
    let dialog_x = (screen_w - dialog_w) / 2;

    // Draw shadow
    draw_shadow(screen, dialog_y, dialog_x, dialog_h, dialog_w)?;

    // Create dialog window
    let mut dialog = screen.newwin(dialog_h, dialog_w, dialog_y, dialog_x)?;

    let mut selected = 0;

    loop {
        // Draw dialog
        if screen.has_colors() {
            dialog.bkgd(b' ' as ChType | attr::color_pair(1))?;
        }
        dialog.erase()?;
        dialog.box_(0, 0)?;

        // Title
        if screen.has_colors() {
            dialog.attron(attr::color_pair(5) | attr::A_BOLD)?;
        }
        let title_x = (dialog_w - title.len() as i32) / 2;
        dialog.mvaddstr(0, title_x, &format!(" {} ", title))?;
        if screen.has_colors() {
            dialog.attroff(attr::color_pair(5) | attr::A_BOLD)?;
        }

        // Message
        let msg_x = (dialog_w - message.len() as i32) / 2;
        dialog.mvaddstr(2, msg_x, message)?;

        // Buttons
        let mut btn_x = (dialog_w - button_width) / 2;
        for (i, btn) in buttons.iter().enumerate() {
            let btn_text = format!("[ {} ]", btn);

            if i == selected {
                if screen.has_colors() {
                    dialog.attron(attr::color_pair(3))?;
                } else {
                    dialog.attron(attr::A_REVERSE)?;
                }
            } else if screen.has_colors() {
                dialog.attron(attr::color_pair(2))?;
            }

            dialog.mvaddstr(4, btn_x, &btn_text)?;

            if i == selected {
                if screen.has_colors() {
                    dialog.attroff(attr::color_pair(3))?;
                } else {
                    dialog.attroff(attr::A_REVERSE)?;
                }
            } else if screen.has_colors() {
                dialog.attroff(attr::color_pair(2))?;
            }

            btn_x += btn_text.len() as i32 + 2;
        }

        screen.wrefresh(&mut dialog)?;

        // Handle input
        let ch = screen.getch()?;
        match ch {
            k if k == key::KEY_LEFT || k == b'h' as i32 => {
                if selected > 0 {
                    selected -= 1;
                }
            }
            k if k == key::KEY_RIGHT || k == b'l' as i32 => {
                if selected < buttons.len() - 1 {
                    selected += 1;
                }
            }
            k if k == 10 || k == 13 || k == key::KEY_ENTER => {
                break;
            }
            k if k == 27 => {
                // ESC - select last button (usually Cancel)
                selected = buttons.len() - 1;
                break;
            }
            k if k == b'\t' as i32 => {
                selected = (selected + 1) % buttons.len();
            }
            _ => {}
        }
    }

    // Restore background
    draw_background(screen)?;
    screen.refresh()?;

    Ok(selected)
}

/// Show an input dialog and return the entered text.
fn show_input_dialog(
    screen: &mut Screen,
    title: &str,
    prompt: &str,
    input_width: i32,
) -> Result<String> {
    let screen_h = screen.lines();
    let screen_w = screen.cols();

    // Calculate dialog size
    let dialog_w = (prompt.len() as i32 + 4)
        .max(title.len() as i32 + 4)
        .max(input_width + 6);
    let dialog_h = 8;

    // Center the dialog
    let dialog_y = (screen_h - dialog_h) / 2;
    let dialog_x = (screen_w - dialog_w) / 2;

    // Draw shadow
    draw_shadow(screen, dialog_y, dialog_x, dialog_h, dialog_w)?;

    // Create dialog window
    let mut dialog = screen.newwin(dialog_h, dialog_w, dialog_y, dialog_x)?;

    let mut input = String::new();

    loop {
        // Draw dialog
        if screen.has_colors() {
            dialog.bkgd(b' ' as ChType | attr::color_pair(1))?;
        }
        dialog.erase()?;
        dialog.box_(0, 0)?;

        // Title
        if screen.has_colors() {
            dialog.attron(attr::color_pair(5) | attr::A_BOLD)?;
        }
        let title_x = (dialog_w - title.len() as i32) / 2;
        dialog.mvaddstr(0, title_x, &format!(" {} ", title))?;
        if screen.has_colors() {
            dialog.attroff(attr::color_pair(5) | attr::A_BOLD)?;
        }

        // Prompt
        let prompt_x = (dialog_w - prompt.len() as i32) / 2;
        dialog.mvaddstr(2, prompt_x, prompt)?;

        // Input field
        let input_x = (dialog_w - input_width) / 2;
        if screen.has_colors() {
            dialog.attron(attr::color_pair(2))?;
        }
        dialog.mv(4, input_x)?;
        for _ in 0..input_width {
            dialog.addch(b' ' as ChType)?;
        }

        // Show input text (scrolling if needed)
        let display_text = if input.len() > (input_width - 1) as usize {
            &input[input.len() - (input_width - 1) as usize..]
        } else {
            &input
        };
        dialog.mvaddstr(4, input_x, display_text)?;
        if screen.has_colors() {
            dialog.attroff(attr::color_pair(2))?;
        }

        // Instructions
        dialog.mvaddstr(6, 2, "Enter: OK | Esc: Cancel")?;

        // Position cursor
        let cursor_x = input_x + display_text.len() as i32;
        dialog.mv(4, cursor_x.min(input_x + input_width - 1))?;
        screen.curs_set(CursorVisibility::Normal)?;

        screen.wrefresh(&mut dialog)?;

        // Handle input
        let ch = screen.getch()?;
        match ch {
            10 | 13 => {
                // Enter
                break;
            }
            27 => {
                // ESC - cancel
                input.clear();
                break;
            }
            127 | 8 => {
                // Backspace
                input.pop();
            }
            k if k == key::KEY_BACKSPACE => {
                input.pop();
            }
            k if (0x20..0x7f).contains(&k) => {
                // Printable character
                if input.len() < 100 {
                    // Reasonable limit
                    input.push(k as u8 as char);
                }
            }
            _ => {}
        }
    }

    screen.curs_set(CursorVisibility::Invisible)?;

    // Restore background
    draw_background(screen)?;
    screen.refresh()?;

    Ok(input)
}

fn draw_shadow(screen: &mut Screen, y: i32, x: i32, h: i32, w: i32) -> Result<()> {
    if !screen.has_colors() {
        return Ok(());
    }

    screen.attron(attr::color_pair(4))?;

    // Right edge shadow
    for row in 1..h {
        screen.mvaddch(y + row, x + w, b' ' as ChType)?;
        screen.mvaddch(y + row, x + w + 1, b' ' as ChType)?;
    }

    // Bottom edge shadow
    for col in 2..w + 2 {
        screen.mvaddch(y + h, x + col, b' ' as ChType)?;
    }

    screen.attroff(attr::color_pair(4))?;

    Ok(())
}
