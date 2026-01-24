//! Key input demo showing how to handle keyboard input.

use ncurses_rs::*;

fn main() -> Result<()> {
    let mut screen = Screen::init()?;

    // Enable keypad mode for special keys
    screen.keypad(true);
    // Don't echo typed characters
    screen.noecho()?;
    // React to keys immediately
    screen.cbreak()?;

    // Initialize colors if available
    if screen.has_colors() {
        screen.start_color()?;
        screen.init_pair(1, COLOR_CYAN, COLOR_BLACK)?;
        screen.init_pair(2, COLOR_YELLOW, COLOR_BLACK)?;
    }

    screen.clear()?;
    screen.mvaddstr(0, 0, "Key Input Demo - Press keys to see their codes")?;
    screen.mvaddstr(1, 0, "Press 'q' to quit, arrow keys, function keys, etc.")?;
    screen.mvaddstr(3, 0, "Key pressed: ")?;
    screen.refresh()?;

    loop {
        let ch = screen.getch()?;

        // Clear the display area
        screen.mv(3, 13)?;
        screen.clrtoeol()?;

        // Display the key info
        if screen.has_colors() {
            screen.attron(attr::color_pair(1))?;
        }

        let key_name = match ch {
            // Special keys
            k if k == key::KEY_UP => "KEY_UP (Up Arrow)".to_string(),
            k if k == key::KEY_DOWN => "KEY_DOWN (Down Arrow)".to_string(),
            k if k == key::KEY_LEFT => "KEY_LEFT (Left Arrow)".to_string(),
            k if k == key::KEY_RIGHT => "KEY_RIGHT (Right Arrow)".to_string(),
            k if k == key::KEY_HOME => "KEY_HOME".to_string(),
            k if k == key::KEY_END => "KEY_END".to_string(),
            k if k == key::KEY_PPAGE => "KEY_PPAGE (Page Up)".to_string(),
            k if k == key::KEY_NPAGE => "KEY_NPAGE (Page Down)".to_string(),
            k if k == key::KEY_BACKSPACE => "KEY_BACKSPACE".to_string(),
            k if k == key::KEY_DC => "KEY_DC (Delete)".to_string(),
            k if k == key::KEY_IC => "KEY_IC (Insert)".to_string(),
            k if k == key::KEY_ENTER => "KEY_ENTER".to_string(),
            // Function keys
            k if k >= key::KEY_F0 + 1 && k <= key::KEY_F0 + 12 => {
                format!("KEY_F{}", k - key::KEY_F0)
            }
            // Escape
            0x1b => "ESC (Escape)".to_string(),
            // Printable ASCII
            32..=126 => format!("'{}' (ASCII {})", ch as u8 as char, ch),
            // Control characters
            1..=26 => format!("Ctrl+{} (ASCII {})", (ch as u8 + b'A' - 1) as char, ch),
            // Other
            _ => format!("Code: {} (0x{:X})", ch, ch),
        };

        screen.mvaddstr(3, 13, &key_name)?;

        if screen.has_colors() {
            screen.attroff(attr::color_pair(1))?;
        }

        // Show raw code
        if screen.has_colors() {
            screen.attron(attr::color_pair(2))?;
        }
        screen.mvaddstr(5, 0, &format!("Raw code: {} (0x{:04X})     ", ch, ch))?;
        if screen.has_colors() {
            screen.attroff(attr::color_pair(2))?;
        }

        screen.refresh()?;

        // Quit on 'q'
        if ch == b'q' as i32 || ch == b'Q' as i32 {
            break;
        }
    }

    Ok(())
}
