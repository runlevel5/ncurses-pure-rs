//! Simple key test to see what bytes are received for each keypress

use ncurses::*;

fn main() -> error::Result<()> {
    let mut screen = Screen::init()?;

    // Enable keypad mode
    screen.keypad(true);

    // Use short escape delay
    screen.set_escdelay(25);

    let stdscr = screen.stdscr_mut();
    stdscr.nodelay(false); // Block for input

    screen.addstr("Press keys to see their codes. Press 'q' to quit.\n")?;
    screen.addstr("Try pressing: Ctrl+i, Tab, 'u', etc.\n\n")?;
    screen.refresh()?;

    loop {
        let ch = screen.getch()?;

        // Show the key code
        screen.addstr(&format!("Key code: {} (0x{:02X})", ch, ch))?;

        // Show what the key is
        if ch < 32 {
            // Control character
            let ctrl_char = (ch as u8 + b'@') as char;
            match ch {
                0x09 => screen.addstr(" = TAB (or Ctrl+I)")?,
                0x0A => screen.addstr(" = Line Feed")?,
                0x0D => screen.addstr(" = Carriage Return")?,
                0x1B => screen.addstr(" = ESC")?,
                _ => screen.addstr(&format!(" = Ctrl+{}", ctrl_char))?,
            }
        } else if ch < 127 {
            screen.addstr(&format!(" = '{}'", ch as u8 as char))?;
        } else {
            // Special key
            match ch {
                0x103 => screen.addstr(" = KEY_UP")?,
                0x102 => screen.addstr(" = KEY_DOWN")?,
                0x104 => screen.addstr(" = KEY_LEFT")?,
                0x105 => screen.addstr(" = KEY_RIGHT")?,
                0x106 => screen.addstr(" = KEY_HOME")?,
                0x168 => screen.addstr(" = KEY_END")?,
                0x199 => screen.addstr(" = KEY_MOUSE")?,
                0x161 => screen.addstr(" = KEY_BTAB (Shift+Tab)")?,
                _ => screen.addstr(&format!(" = special key (0x{:03X})", ch))?,
            }
        }
        screen.addstr("\n")?;
        screen.refresh()?;

        // Quit on 'q'
        if ch == b'q' as i32 {
            break;
        }
    }

    screen.endwin()?;
    Ok(())
}
