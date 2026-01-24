//! Window management example showing how to create and use multiple windows.

use ncurses_rs::*;

fn main() -> Result<()> {
    let mut screen = Screen::init()?;

    // Initialize colors if available
    if screen.has_colors() {
        screen.start_color()?;
        screen.init_pair(1, COLOR_WHITE, COLOR_BLUE)?; // Title window
        screen.init_pair(2, COLOR_BLACK, COLOR_WHITE)?; // Content window
        screen.init_pair(3, COLOR_YELLOW, COLOR_BLACK)?; // Status window
    }

    screen.keypad(true);
    screen.noecho()?;
    screen.cbreak()?;

    let max_y = screen.lines();
    let max_x = screen.cols();

    // Clear the main screen with a background
    screen.clear()?;
    screen.mvaddstr(0, 0, "Window Demo - Press 'q' to quit, arrow keys to move")?;

    // Create a title window at the top
    let mut title_win = screen.newwin(3, max_x - 4, 2, 2)?;
    if screen.has_colors() {
        title_win.bkgd(attr::color_pair(1))?;
    }
    title_win.box_(0, 0)?;
    title_win.mvaddstr(1, 2, "Title Window - ncurses-rs Multi-Window Demo")?;

    // Create a content window in the middle
    let content_height = max_y - 12;
    let content_width = max_x - 4;
    let mut content_win = screen.newwin(content_height, content_width, 5, 2)?;
    if screen.has_colors() {
        content_win.bkgd(attr::color_pair(2))?;
    }
    content_win.box_(0, 0)?;
    content_win.mvaddstr(1, 2, "Content Window")?;
    content_win.mvaddstr(3, 2, "This window demonstrates:")?;
    content_win.mvaddstr(4, 4, "- Creating windows with newwin()")?;
    content_win.mvaddstr(5, 4, "- Setting window backgrounds with bkgd()")?;
    content_win.mvaddstr(6, 4, "- Drawing borders with box_()")?;
    content_win.mvaddstr(7, 4, "- Moving cursor with mv()")?;
    content_win.mvaddstr(8, 4, "- Adding text with addstr()")?;

    // Create a status window at the bottom
    let mut status_win = screen.newwin(3, max_x - 4, max_y - 5, 2)?;
    if screen.has_colors() {
        status_win.bkgd(attr::color_pair(3))?;
    }
    status_win.box_(0, 0)?;
    status_win.mvaddstr(1, 2, "Status: Ready")?;

    // Create a movable box
    let box_height = 5;
    let box_width = 20;
    let mut box_y = (content_height - box_height) / 2 + 5;
    let mut box_x = (content_width - box_width) / 2 + 2;
    let mut movable_win = screen.newwin(box_height, box_width, box_y, box_x)?;

    // Initial refresh of all windows
    screen.refresh()?;
    screen.wrefresh(&mut title_win)?;
    screen.wrefresh(&mut content_win)?;
    screen.wrefresh(&mut status_win)?;
    draw_movable_box(&mut movable_win, box_y, box_x)?;
    screen.wrefresh(&mut movable_win)?;

    // Main loop
    loop {
        let ch = screen.getch()?;

        let (old_y, old_x) = (box_y, box_x);
        let mut moved = false;

        match ch {
            k if k == key::KEY_UP => {
                if box_y > 6 {
                    box_y -= 1;
                    moved = true;
                }
            }
            k if k == key::KEY_DOWN => {
                if box_y < max_y - box_height - 6 {
                    box_y += 1;
                    moved = true;
                }
            }
            k if k == key::KEY_LEFT => {
                if box_x > 3 {
                    box_x -= 1;
                    moved = true;
                }
            }
            k if k == key::KEY_RIGHT => {
                if box_x < max_x - box_width - 3 {
                    box_x += 1;
                    moved = true;
                }
            }
            k if k == b'q' as i32 || k == b'Q' as i32 => {
                break;
            }
            _ => {}
        }

        if moved {
            // Erase old position by refreshing the content window
            content_win.touchwin();
            screen.wrefresh(&mut content_win)?;

            // Move and redraw the movable window
            movable_win = screen.newwin(box_height, box_width, box_y, box_x)?;
            draw_movable_box(&mut movable_win, box_y, box_x)?;
            screen.wrefresh(&mut movable_win)?;

            // Update status
            status_win.mv(1, 2)?;
            status_win.clrtoeol()?;
            status_win.box_(0, 0)?;
            status_win.mvaddstr(
                1,
                2,
                &format!(
                    "Status: Moved from ({},{}) to ({},{})",
                    old_y, old_x, box_y, box_x
                ),
            )?;
            screen.wrefresh(&mut status_win)?;
        }
    }

    Ok(())
}

fn draw_movable_box(win: &mut Window, y: i32, x: i32) -> Result<()> {
    win.erase()?;
    win.attron(attr::A_BOLD)?;
    win.box_(0, 0)?;
    win.mvaddstr(1, 2, "Movable Box")?;
    win.attroff(attr::A_BOLD)?;
    win.mvaddstr(2, 2, &format!("Pos: ({},{})", y, x))?;
    win.mvaddstr(3, 2, "Use arrows")?;
    Ok(())
}
