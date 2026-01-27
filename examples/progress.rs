//! Progress bar example demonstrating text UI components.
//!
//! This example shows how to create animated progress bars and
//! status indicators using ncurses-pure.
//!
//! Features demonstrated:
//! - Drawing progress bars
//! - Animated updates
//! - Status messages
//! - Box drawing characters
//! - Color pairs for status indication

use ncurses::*;
use std::thread;
use std::time::Duration;

fn main() -> Result<()> {
    let mut screen = Screen::init()?;

    // Initialize colors
    if screen.has_colors() {
        screen.start_color()?;
        screen.init_pair(1, COLOR_WHITE, COLOR_BLUE)?; // Header
        screen.init_pair(2, COLOR_GREEN, COLOR_BLACK)?; // Progress bar fill
        screen.init_pair(3, COLOR_WHITE, COLOR_BLACK)?; // Progress bar empty
        screen.init_pair(4, COLOR_YELLOW, COLOR_BLACK)?; // Warning
        screen.init_pair(5, COLOR_RED, COLOR_BLACK)?; // Error
        screen.init_pair(6, COLOR_CYAN, COLOR_BLACK)?; // Info
    }

    screen.curs_set(CursorVisibility::Invisible)?;
    screen.noecho()?;
    screen.cbreak()?;
    screen.nodelay(true); // Non-blocking input

    let max_y = screen.lines();
    let max_x = screen.cols();

    // Draw the interface
    draw_header(&mut screen, max_x)?;
    draw_frame(&mut screen, max_y, max_x)?;

    // Define tasks to simulate
    let tasks = [
        ("Downloading files...", 2000),
        ("Processing data...", 1500),
        ("Compiling modules...", 3000),
        ("Running tests...", 2500),
        ("Generating reports...", 1000),
    ];

    let progress_y = 6;
    let status_y = 10;

    // Process each task
    for (idx, (task_name, duration_ms)) in tasks.iter().enumerate() {
        // Show task name
        screen.mv(status_y, 4)?;
        screen.clrtoeol()?;
        if screen.has_colors() {
            screen.attron(attr::color_pair(6))?;
        }
        screen.addstr(&format!("Task {}/{}: {}", idx + 1, tasks.len(), task_name))?;
        if screen.has_colors() {
            screen.attroff(attr::color_pair(6))?;
        }

        // Animate progress bar
        let steps = 50;
        let delay = Duration::from_millis((*duration_ms as u64) / steps);

        for progress in 0..=steps {
            let percent = (progress as f64 / steps as f64) * 100.0;
            draw_progress_bar(&mut screen, progress_y, 4, max_x - 8, percent)?;

            // Show percentage
            screen.mv(progress_y + 1, 4)?;
            screen.clrtoeol()?;
            screen.addstr(&format!("{:.0}% complete", percent))?;

            screen.refresh()?;

            // Check for 'q' to quit (nodelay mode returns ERR if no input)
            let ch = screen.getch();
            if let Ok(ch) = ch {
                if ch == b'q' as i32 || ch == b'Q' as i32 {
                    return Ok(());
                }
            }

            thread::sleep(delay);
        }

        // Brief pause between tasks
        thread::sleep(Duration::from_millis(200));
    }

    // Show completion message
    screen.mv(status_y, 4)?;
    screen.clrtoeol()?;
    if screen.has_colors() {
        screen.attron(attr::color_pair(2) | attr::A_BOLD)?;
    }
    screen.addstr("All tasks completed successfully!")?;
    if screen.has_colors() {
        screen.attroff(attr::color_pair(2) | attr::A_BOLD)?;
    }

    // Show spinner animation while "finalizing"
    let spinner_chars = ['|', '/', '-', '\\'];
    screen.mv(status_y + 2, 4)?;
    screen.addstr("Finalizing ")?;

    for i in 0..20 {
        screen.mv(status_y + 2, 15)?;
        screen.addch(spinner_chars[i % 4] as ChType)?;
        screen.refresh()?;
        thread::sleep(Duration::from_millis(100));
    }

    screen.mv(status_y + 2, 4)?;
    screen.clrtoeol()?;
    screen.addstr("Done! Press any key to exit.")?;
    screen.refresh()?;

    // Wait for keypress
    screen.nodelay(false);
    screen.getch()?;

    Ok(())
}

fn draw_header(screen: &mut Screen, max_x: i32) -> Result<()> {
    if screen.has_colors() {
        screen.attron(attr::color_pair(1))?;
    }
    screen.attron(attr::A_BOLD)?;

    // Fill header line
    screen.mv(0, 0)?;
    for _ in 0..max_x {
        screen.addch(b' ' as ChType)?;
    }

    let title = " ncurses-pure Progress Demo ";
    let x = (max_x - title.len() as i32) / 2;
    screen.mvaddstr(0, x, title)?;

    screen.attroff(attr::A_BOLD)?;
    if screen.has_colors() {
        screen.attroff(attr::color_pair(1))?;
    }

    Ok(())
}

fn draw_frame(screen: &mut Screen, max_y: i32, max_x: i32) -> Result<()> {
    // Draw a box around the content area
    let box_y = 3;
    let box_height = max_y - 6;
    let box_width = max_x - 4;

    // Top border
    screen.mv(box_y, 2)?;
    screen.addch(ACS_ULCORNER as ChType)?;
    for _ in 0..box_width - 2 {
        screen.addch(ACS_HLINE as ChType)?;
    }
    screen.addch(ACS_URCORNER as ChType)?;

    // Side borders
    for y in 1..box_height - 1 {
        screen.mv(box_y + y, 2)?;
        screen.addch(ACS_VLINE as ChType)?;
        screen.mv(box_y + y, box_width + 1)?;
        screen.addch(ACS_VLINE as ChType)?;
    }

    // Bottom border
    screen.mv(box_y + box_height - 1, 2)?;
    screen.addch(ACS_LLCORNER as ChType)?;
    for _ in 0..box_width - 2 {
        screen.addch(ACS_HLINE as ChType)?;
    }
    screen.addch(ACS_LRCORNER as ChType)?;

    // Instructions at bottom
    screen.mv(max_y - 2, 2)?;
    if screen.has_colors() {
        screen.attron(attr::color_pair(4))?;
    }
    screen.addstr("Press 'q' to quit")?;
    if screen.has_colors() {
        screen.attroff(attr::color_pair(4))?;
    }

    Ok(())
}

fn draw_progress_bar(screen: &mut Screen, y: i32, x: i32, width: i32, percent: f64) -> Result<()> {
    let filled = ((percent / 100.0) * width as f64) as i32;
    let empty = width - filled;

    screen.mv(y, x)?;

    // Draw filled portion
    if screen.has_colors() {
        screen.attron(attr::color_pair(2))?;
    }
    for _ in 0..filled {
        screen.addch(ACS_CKBOARD as ChType)?;
    }
    if screen.has_colors() {
        screen.attroff(attr::color_pair(2))?;
    }

    // Draw empty portion
    if screen.has_colors() {
        screen.attron(attr::color_pair(3))?;
    }
    for _ in 0..empty {
        screen.addch(b'-' as ChType)?;
    }
    if screen.has_colors() {
        screen.attroff(attr::color_pair(3))?;
    }

    Ok(())
}
