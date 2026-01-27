//! Colors demo showing the color capabilities of ncurses-pure.

use ncurses::*;

fn main() -> Result<()> {
    let mut screen = Screen::init()?;

    screen.clear()?;
    screen.mvaddstr(0, 0, "ncurses-pure Color Demo")?;

    if !screen.has_colors() {
        screen.mvaddstr(2, 0, "Your terminal does not support colors!")?;
        screen.mvaddstr(3, 0, "Press any key to exit...")?;
        screen.refresh()?;
        screen.getch()?;
        return Ok(());
    }

    screen.start_color()?;

    // Show terminal capabilities
    let num_colors = screen.num_colors();
    let num_pairs = screen.num_color_pairs();
    screen.mvaddstr(
        2,
        0,
        &format!(
            "Terminal supports {} colors and {} color pairs",
            num_colors, num_pairs
        ),
    )?;

    // Initialize color pairs for the basic 8 colors
    let colors = [
        ("Black", COLOR_BLACK),
        ("Red", COLOR_RED),
        ("Green", COLOR_GREEN),
        ("Yellow", COLOR_YELLOW),
        ("Blue", COLOR_BLUE),
        ("Magenta", COLOR_MAGENTA),
        ("Cyan", COLOR_CYAN),
        ("White", COLOR_WHITE),
    ];

    // Create color pairs: each color on black background
    for (i, (_, color)) in colors.iter().enumerate() {
        screen.init_pair((i + 1) as i16, *color, COLOR_BLACK)?;
    }

    // Display each color
    screen.mvaddstr(4, 0, "Basic colors on black background:")?;
    for (i, (name, _)) in colors.iter().enumerate() {
        let y = 5 + i as i32;
        screen.mv(y, 2)?;
        screen.attron(attr::color_pair((i + 1) as i16))?;
        screen.addstr(&format!("  {} - {}", name, "Sample Text"))?;
        screen.attroff(attr::color_pair((i + 1) as i16))?;
    }

    // Create reverse pairs: black text on colored backgrounds
    for (i, (_, color)) in colors.iter().enumerate() {
        screen.init_pair((i + 9) as i16, COLOR_BLACK, *color)?;
    }

    // Display reverse colors
    screen.mvaddstr(4, 40, "Black text on colored backgrounds:")?;
    for (i, (name, _)) in colors.iter().enumerate() {
        let y = 5 + i as i32;
        screen.mv(y, 42)?;
        screen.attron(attr::color_pair((i + 9) as i16))?;
        screen.addstr(&format!("  {} background  ", name))?;
        screen.attroff(attr::color_pair((i + 9) as i16))?;
    }

    // Show attributes combined with colors
    screen.mvaddstr(14, 0, "Attributes combined with colors:")?;

    screen.init_pair(17, COLOR_GREEN, COLOR_BLACK)?;
    screen.mv(15, 2)?;
    screen.attron(attr::color_pair(17) | attr::A_BOLD)?;
    screen.addstr("Bold Green")?;
    screen.attroff(attr::color_pair(17) | attr::A_BOLD)?;

    screen.mv(15, 20)?;
    screen.attron(attr::color_pair(17) | attr::A_UNDERLINE)?;
    screen.addstr("Underline Green")?;
    screen.attroff(attr::color_pair(17) | attr::A_UNDERLINE)?;

    screen.init_pair(18, COLOR_CYAN, COLOR_BLACK)?;
    screen.mv(16, 2)?;
    screen.attron(attr::color_pair(18) | attr::A_REVERSE)?;
    screen.addstr("Reverse Cyan")?;
    screen.attroff(attr::color_pair(18) | attr::A_REVERSE)?;

    screen.mv(16, 20)?;
    screen.attron(attr::color_pair(18) | attr::A_BOLD | attr::A_UNDERLINE)?;
    screen.addstr("Bold+Underline Cyan")?;
    screen.attroff(attr::color_pair(18) | attr::A_BOLD | attr::A_UNDERLINE)?;

    // Instructions
    screen.mvaddstr(18, 0, "Press any key to exit...")?;
    screen.refresh()?;
    screen.getch()?;

    Ok(())
}
