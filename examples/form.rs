//! Form example demonstrating data entry forms.
//!
//! This example shows how to use the form library to create
//! interactive data entry forms with validation.
//!
//! Features demonstrated:
//! - Creating form fields
//! - Building forms
//! - Field navigation (Tab, Shift+Tab)
//! - Text input and editing
//! - Field validation

use ncurses_rs::*;

#[cfg(feature = "form")]
fn main() -> Result<()> {
    use ncurses_rs::form::*;

    let mut screen = Screen::init()?;

    // Initialize colors
    if screen.has_colors() {
        screen.start_color()?;
        screen.init_pair(1, COLOR_WHITE, COLOR_BLUE)?; // Header
        screen.init_pair(2, COLOR_BLACK, COLOR_CYAN)?; // Active field
        screen.init_pair(3, COLOR_WHITE, COLOR_BLACK)?; // Inactive field
        screen.init_pair(4, COLOR_YELLOW, COLOR_BLACK)?; // Labels
        screen.init_pair(5, COLOR_RED, COLOR_BLACK)?; // Error
        screen.init_pair(6, COLOR_GREEN, COLOR_BLACK)?; // Success
    }

    screen.keypad(true);
    screen.noecho()?;
    screen.cbreak()?;

    let max_y = screen.lines();
    let max_x = screen.cols();

    // Create form fields
    let field_width = 30;
    let start_row = 5;
    let label_col = 5;
    let field_col = 20;

    let fields = vec![
        Field::new(1, field_width, start_row, field_col, 0, 0), // Name
        Field::new(1, field_width, start_row + 2, field_col, 0, 0), // Email
        Field::new(1, field_width, start_row + 4, field_col, 0, 0), // Phone
        Field::new(1, 10, start_row + 6, field_col, 0, 0),      // Age
        Field::new(3, field_width, start_row + 8, field_col, 0, 0), // Comments
    ];

    // Create form
    let mut form = Form::new(fields);

    // Set up field validation
    if let Some(age_field) = form.field(3) {
        age_field.borrow_mut().set_type(TypeInteger {
            padding: 0,
            min: 1,
            max: 150,
        });
    }

    // Define labels
    let labels = ["Name:", "Email:", "Phone:", "Age:", "Comments:"];

    // Post form
    form.post()?;

    // Draw interface
    draw_interface(&mut screen, max_y, max_x)?;
    draw_labels(&mut screen, &labels, start_row, label_col)?;
    draw_instructions(&mut screen, max_y)?;

    screen.refresh()?;

    // Main loop
    let mut message: Option<(String, bool)> = None; // (message, is_error)

    loop {
        // Draw the form fields
        draw_form_fields(&mut screen, &form, field_col, field_width)?;

        // Draw any message
        if let Some((ref msg, is_error)) = message {
            draw_message(&mut screen, max_y - 4, msg, is_error)?;
        }

        screen.refresh()?;

        let ch = screen.getch()?;

        // Clear message on next keypress
        message = None;

        match ch {
            // Tab to next field
            k if k == b'\t' as i32 => {
                form.driver(REQ_NEXT_FIELD)?;
            }

            // Shift+Tab to previous field (KEY_BTAB)
            k if k == key::KEY_BTAB || k == 353 => {
                form.driver(REQ_PREV_FIELD)?;
            }

            // Arrow keys for cursor movement
            k if k == key::KEY_LEFT => {
                form.driver(REQ_LEFT_CHAR)?;
            }
            k if k == key::KEY_RIGHT => {
                form.driver(REQ_RIGHT_CHAR)?;
            }
            k if k == key::KEY_UP => {
                form.driver(REQ_PREV_FIELD)?;
            }
            k if k == key::KEY_DOWN => {
                form.driver(REQ_NEXT_FIELD)?;
            }

            // Home/End
            k if k == key::KEY_HOME => {
                form.driver(REQ_BEG_FIELD)?;
            }
            k if k == key::KEY_END => {
                form.driver(REQ_END_FIELD)?;
            }

            // Backspace
            k if k == 127 || k == 8 || k == key::KEY_BACKSPACE => {
                form.driver(REQ_DEL_PREV)?;
            }

            // Delete
            k if k == key::KEY_DC => {
                form.driver(REQ_DEL_CHAR)?;
            }

            // Enter to submit
            k if k == 10 || k == 13 => {
                // Validate all fields
                if form.validate_all() {
                    let data = form.data();
                    message = Some((
                        format!(
                            "Submitted: Name={}, Email={}, Phone={}, Age={}",
                            data[0].trim(),
                            data[1].trim(),
                            data[2].trim(),
                            data[3].trim()
                        ),
                        false,
                    ));
                } else {
                    message = Some(("Validation failed! Check your input.".to_string(), true));
                }
            }

            // Ctrl+C or Escape to clear current field
            k if k == 27 => {
                form.driver(REQ_CLR_FIELD)?;
            }

            // F1 to show help
            k if k == key::key_f(1) => {
                message = Some((
                    "Help: Tab=next, Shift+Tab=prev, Enter=submit, Esc=clear".to_string(),
                    false,
                ));
            }

            // q/Q with Ctrl to quit
            k if k == 17 => {
                // Ctrl+Q
                break;
            }

            // F10 to quit
            k if k == key::key_f(10) => {
                break;
            }

            // Printable characters
            k if k >= 0x20 && k < 0x7f => {
                form.driver(k)?;
            }

            _ => {}
        }
    }

    // Unpost form
    form.unpost()?;

    Ok(())
}

#[cfg(feature = "form")]
fn draw_interface(screen: &mut Screen, _max_y: i32, max_x: i32) -> Result<()> {
    screen.clear()?;

    // Draw header
    if screen.has_colors() {
        screen.attron(attr::color_pair(1))?;
    }
    screen.attron(attr::A_BOLD)?;

    let title = " ncurses-rs Form Demo - Registration Form ";
    let x = (max_x - title.len() as i32) / 2;
    screen.mvaddstr(1, x, title)?;

    screen.attroff(attr::A_BOLD)?;
    if screen.has_colors() {
        screen.attroff(attr::color_pair(1))?;
    }

    // Draw separator
    screen.mv(3, 0)?;
    screen.hline(0, max_x)?;

    Ok(())
}

#[cfg(feature = "form")]
fn draw_labels(screen: &mut Screen, labels: &[&str], start_row: i32, label_col: i32) -> Result<()> {
    if screen.has_colors() {
        screen.attron(attr::color_pair(4))?;
    }

    for (i, label) in labels.iter().enumerate() {
        let row = start_row + (i as i32 * 2);
        screen.mvaddstr(row, label_col, label)?;
    }

    if screen.has_colors() {
        screen.attroff(attr::color_pair(4))?;
    }

    Ok(())
}

#[cfg(feature = "form")]
fn draw_instructions(screen: &mut Screen, max_y: i32) -> Result<()> {
    if screen.has_colors() {
        screen.attron(attr::color_pair(4))?;
    }

    screen.mvaddstr(max_y - 3, 2, "Navigation: TAB/Shift+TAB, Arrow keys")?;
    screen.mvaddstr(
        max_y - 2,
        2,
        "Actions: ENTER=Submit, ESC=Clear field, F1=Help, F10=Quit",
    )?;

    if screen.has_colors() {
        screen.attroff(attr::color_pair(4))?;
    }

    Ok(())
}

#[cfg(feature = "form")]
fn draw_form_fields(
    screen: &mut Screen,
    form: &form::Form,
    field_col: i32,
    field_width: i32,
) -> Result<()> {
    let current_idx = form.current_field_index();

    for i in 0..form.field_count() {
        if let Some(field_rc) = form.field(i) {
            let field = field_rc.borrow();
            let (height, width, row, _) = field.dimensions();
            let is_current = i == current_idx;

            // Choose colors based on active state
            if screen.has_colors() {
                if is_current {
                    screen.attron(attr::color_pair(2))?;
                } else {
                    screen.attron(attr::color_pair(3))?;
                }
            }

            // Draw field background
            for h in 0..height {
                screen.mv(row + h, field_col)?;
                let actual_width = if i == 3 {
                    10
                } else if i == 4 {
                    field_width
                } else {
                    field_width
                };
                for _ in 0..actual_width {
                    screen.addch(b' ' as ChType)?;
                }
            }

            // Draw field content
            let buffer = field.buffer();
            let display_width = if i == 3 { 10 } else { width };
            let visible = if buffer.len() > display_width as usize {
                &buffer[buffer.len() - display_width as usize..]
            } else {
                &buffer
            };

            screen.mvaddstr(row, field_col, visible)?;

            // Draw cursor for active field
            if is_current {
                let (_cursor_row, cursor_col) = field.cursor_pos();
                let cursor_pos = cursor_col.min(display_width as usize - 1);
                screen.mv(row, field_col + cursor_pos as i32)?;
            }

            // Draw field border indicator
            if is_current {
                screen.mvaddstr(row, field_col - 2, "> ")?;
            } else {
                screen.mvaddstr(row, field_col - 2, "  ")?;
            }

            if screen.has_colors() {
                if is_current {
                    screen.attroff(attr::color_pair(2))?;
                } else {
                    screen.attroff(attr::color_pair(3))?;
                }
            }
        }
    }

    Ok(())
}

#[cfg(feature = "form")]
fn draw_message(screen: &mut Screen, row: i32, message: &str, is_error: bool) -> Result<()> {
    screen.mv(row, 0)?;
    screen.clrtoeol()?;

    if screen.has_colors() {
        if is_error {
            screen.attron(attr::color_pair(5))?;
        } else {
            screen.attron(attr::color_pair(6))?;
        }
    }

    screen.attron(attr::A_BOLD)?;
    screen.mvaddstr(row, 2, message)?;
    screen.attroff(attr::A_BOLD)?;

    if screen.has_colors() {
        if is_error {
            screen.attroff(attr::color_pair(5))?;
        } else {
            screen.attroff(attr::color_pair(6))?;
        }
    }

    Ok(())
}

#[cfg(not(feature = "form"))]
fn main() {
    eprintln!("This example requires the 'form' feature.");
    eprintln!("Run with: cargo run --example form --features form");
}
