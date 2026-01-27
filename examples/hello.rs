//! Simple "Hello World" example demonstrating basic ncurses-pure usage.

use ncurses::*;

fn main() -> Result<()> {
    // Initialize the screen
    let mut screen = Screen::init()?;

    // Optional: initialize colors if the terminal supports them
    if screen.has_colors() {
        screen.start_color()?;
        screen.init_pair(1, COLOR_GREEN, COLOR_BLACK)?;
        screen.init_pair(2, COLOR_YELLOW, COLOR_BLACK)?;
    }

    // Enable keypad mode to capture special keys
    screen.keypad(true);

    // Clear the screen and display a message
    screen.clear()?;

    // Display centered title
    let title = "Welcome to ncurses-pure!";
    let cols = screen.cols();
    let lines = screen.lines();
    let x = (cols - title.len() as i32) / 2;
    let y = lines / 2 - 2;

    screen.mvaddstr(y, x, title)?;

    // Add some styled text
    if screen.has_colors() {
        screen.attron(attr::color_pair(1))?;
    }
    screen.attron(attr::A_BOLD)?;
    screen.mvaddstr(y + 2, x - 5, "A pure Rust ncurses implementation")?;
    screen.attroff(attr::A_BOLD)?;
    if screen.has_colors() {
        screen.attroff(attr::color_pair(1))?;
    }

    // Display instructions
    if screen.has_colors() {
        screen.attron(attr::color_pair(2))?;
    }
    screen.mvaddstr(y + 5, x - 3, "Press any key to exit...")?;
    if screen.has_colors() {
        screen.attroff(attr::color_pair(2))?;
    }

    // Refresh to show changes
    screen.refresh()?;

    // Wait for a key press
    screen.getch()?;

    // Screen is automatically cleaned up when dropped
    Ok(())
}
