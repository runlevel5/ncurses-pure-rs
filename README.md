# ncurses-pure

A **pure Rust implementation** of the ncurses library, providing full API compatibility with **ncurses 6.6** while following the X/Open XSI Curses standard.

## Overview

ncurses-pure provides terminal UI capabilities without requiring any C dependencies. It's designed as a drop-in replacement for C ncurses with an idiomatic Rust API.

### Key Features

- **Pure Rust** - No FFI bindings or C dependencies
- **ncurses 6.6 Compatible** - Matches the ncurses 6.6 API
- **XSI Curses Compliant** - Follows the X/Open standard
- **Full Unicode Support** - Wide character support via the `wide` feature
- **Mouse Support** - Terminal mouse event handling
- **Extended Colors** - Support for >256 color pairs
- **Panels Library** - Window stacking and management
- **Menu Library** - Selection interfaces
- **Form Library** - Data entry forms
- **Cross-Platform** - Works on Linux, macOS, and BSD

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ncurses-pure = { git = "https://github.com/runlevel5/ncurses-pure-rust.git" }
```

### Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `wide` | Yes | Wide character (Unicode) support |
| `mouse` | Yes | Mouse event handling |
| `ext-colors` | Yes | Extended colors (>256 pairs) |
| `slk` | No | Soft function key labels |
| `panels` | No | Panels library for window stacking |
| `menu` | No | Menu library for selection interfaces |
| `form` | No | Form library for data entry |
| `full` | No | Enable all features |

Enable specific features:

```toml
[dependencies]
ncurses-pure = { git = "https://github.com/runlevel5/ncurses-pure-rust.git", features = ["panels", "menu"] }
```

Or enable all features:

```toml
[dependencies]
ncurses-pure = { git = "https://github.com/runlevel5/ncurses-pure-rust.git", features = ["full"] }
```

## Quick Start

### Hello World

```rust
use ncurses::*;

fn main() -> Result<()> {
    // Initialize the screen
    let mut screen = Screen::init()?;
    
    // Print a message
    screen.addstr("Hello, ncurses-pure!")?;
    screen.addstr("\nPress any key to exit...")?;
    
    // Refresh to show output
    screen.refresh()?;
    
    // Wait for a key press
    screen.getch()?;
    
    Ok(())
    // Screen is automatically cleaned up on drop
}
```

### Using Colors

```rust
use ncurses::*;

fn main() -> Result<()> {
    let mut screen = Screen::init()?;
    
    if screen.has_colors() {
        screen.start_color()?;
        
        // Define color pairs
        screen.init_pair(1, COLOR_RED, COLOR_BLACK)?;
        screen.init_pair(2, COLOR_GREEN, COLOR_BLACK)?;
        screen.init_pair(3, COLOR_YELLOW, COLOR_BLUE)?;
        
        // Use colors
        screen.attron(attr::color_pair(1))?;
        screen.addstr("Red text\n")?;
        screen.attroff(attr::color_pair(1))?;
        
        screen.attron(attr::color_pair(2) | attr::A_BOLD)?;
        screen.addstr("Bold green text\n")?;
    }
    
    screen.refresh()?;
    screen.getch()?;
    Ok(())
}
```

### Multiple Windows

```rust
use ncurses::*;

fn main() -> Result<()> {
    let mut screen = Screen::init()?;
    screen.cbreak()?;
    screen.noecho()?;
    
    // Create a window (height, width, y, x)
    let mut win = screen.newwin(10, 40, 5, 10)?;
    
    // Draw a border
    win.box_(0, 0)?;
    
    // Add content
    win.mvaddstr(1, 2, "Window Title")?;
    win.mvaddstr(3, 2, "This is a window!")?;
    
    // Refresh both windows
    screen.refresh()?;
    screen.wrefresh(&mut win)?;
    
    screen.getch()?;
    Ok(())
}
```

### Handling Keyboard Input

```rust
use ncurses::*;

fn main() -> Result<()> {
    let mut screen = Screen::init()?;
    screen.keypad(true);  // Enable special keys
    screen.noecho()?;     // Don't echo input
    screen.cbreak()?;     // Disable line buffering
    
    screen.addstr("Press arrow keys (q to quit)\n")?;
    screen.refresh()?;
    
    loop {
        let ch = screen.getch()?;
        
        let msg = match ch {
            k if k == key::KEY_UP => "Up arrow",
            k if k == key::KEY_DOWN => "Down arrow", 
            k if k == key::KEY_LEFT => "Left arrow",
            k if k == key::KEY_RIGHT => "Right arrow",
            k if k == b'q' as i32 => break,
            _ => "Other key",
        };
        
        screen.clear()?;
        screen.addstr(&format!("You pressed: {}\n", msg))?;
        screen.refresh()?;
    }
    
    Ok(())
}
```

### Mouse Support

```rust
use ncurses::*;

fn main() -> Result<()> {
    let mut screen = Screen::init()?;
    screen.keypad(true);
    
    // Enable mouse events
    screen.mousemask(BUTTON1_CLICKED | BUTTON1_DOUBLE_CLICKED)?;
    
    screen.addstr("Click anywhere (q to quit)\n")?;
    screen.refresh()?;
    
    loop {
        let ch = screen.getch()?;
        
        if ch == key::KEY_MOUSE {
            if let Ok(event) = screen.getmouse() {
                screen.mvaddstr(2, 0, &format!(
                    "Mouse clicked at ({}, {})", 
                    event.x, event.y
                ))?;
                screen.refresh()?;
            }
        } else if ch == b'q' as i32 {
            break;
        }
    }
    
    Ok(())
}
```

### Using Panels

```rust
use ncurses::*;
use ncurses::panels::*;

fn main() -> Result<()> {
    let mut screen = Screen::init()?;
    
    // Create windows
    let win1 = screen.newwin(10, 30, 2, 2)?;
    let win2 = screen.newwin(10, 30, 4, 6)?;
    
    // Create panels from windows
    let mut deck = PanelDeck::new();
    let panel1 = deck.new_panel(win1);
    let panel2 = deck.new_panel(win2);
    
    // Draw on panels
    deck.panel_mut(panel1).unwrap().window_mut().box_(0, 0)?;
    deck.panel_mut(panel1).unwrap().window_mut().mvaddstr(1, 1, "Panel 1 (bottom)")?;
    
    deck.panel_mut(panel2).unwrap().window_mut().box_(0, 0)?;
    deck.panel_mut(panel2).unwrap().window_mut().mvaddstr(1, 1, "Panel 2 (top)")?;
    
    // Refresh and display
    screen.refresh()?;
    deck.update_panels(&mut screen)?;
    
    screen.getch()?;
    Ok(())
}
```

## API Reference

### Core Types

| Type | Description |
|------|-------------|
| `Screen` | Main screen handle, manages terminal state |
| `Window` | A rectangular region for text output |
| `ChType` | Character with attributes (non-wide mode) |
| `AttrT` | Video attributes (bold, underline, etc.) |
| `CCharT` | Complex character with attributes (wide mode) |

### Common Functions

#### Screen Management
- `Screen::init()` - Initialize curses mode
- `screen.endwin()` - End curses mode (called automatically on drop)
- `screen.refresh()` - Update the physical screen
- `screen.clear()` - Clear the screen

#### Output
- `addch(ch)` - Add a character
- `addstr(s)` - Add a string
- `mvaddch(y, x, ch)` - Move and add character
- `mvaddstr(y, x, s)` - Move and add string
- `printw(fmt, ...)` - Formatted output

#### Input
- `getch()` - Read a character
- `getstr()` - Read a string
- `keypad(true)` - Enable function key reading

#### Attributes
- `attron(attr)` - Turn on attribute
- `attroff(attr)` - Turn off attribute
- `attrset(attr)` - Set attributes

#### Windows
- `newwin(h, w, y, x)` - Create a window
- `box_(v, h)` - Draw a border
- `wrefresh(win)` - Refresh a window

### Attribute Constants

```rust
use ncurses::attr::*;

A_NORMAL      // Normal display
A_STANDOUT    // Best highlighting mode
A_UNDERLINE   // Underlined text
A_REVERSE     // Reverse video
A_BLINK       // Blinking text
A_DIM         // Half bright
A_BOLD        // Bold text
A_ITALIC      // Italic text
A_INVIS       // Invisible text
```

### Color Constants

```rust
use ncurses::*;

COLOR_BLACK   // 0
COLOR_RED     // 1
COLOR_GREEN   // 2
COLOR_YELLOW  // 3
COLOR_BLUE    // 4
COLOR_MAGENTA // 5
COLOR_CYAN    // 6
COLOR_WHITE   // 7
```

### Key Constants

```rust
use ncurses::key::*;

KEY_UP, KEY_DOWN, KEY_LEFT, KEY_RIGHT  // Arrow keys
KEY_HOME, KEY_END                       // Navigation
KEY_PPAGE, KEY_NPAGE                    // Page Up/Down
KEY_BACKSPACE, KEY_DC                   // Backspace, Delete
KEY_F0..KEY_F(63)                       // Function keys
KEY_MOUSE                               // Mouse event
KEY_RESIZE                              // Terminal resize
```

## Examples

Run the included examples:

```bash
cargo run --example hello      # Basic hello world
cargo run --example colors     # Color demonstration
cargo run --example keys       # Keyboard input demo
cargo run --example windows    # Multi-window demo
cargo run --example mouse --features mouse    # Mouse demo
cargo run --example panels --features panels  # Panels demo
```

## Differences from C ncurses

1. **Memory Safety**: No manual memory management needed
2. **Error Handling**: Functions return `Result<T>` instead of error codes
3. **RAII**: Screen is automatically cleaned up when dropped
4. **Method Syntax**: `win.addstr("text")` instead of `waddstr(win, "text")`
5. **No Global State**: Screen state is contained in the `Screen` struct

## API Styles

ncurses-pure provides **two API styles** to accommodate different use cases:

### Idiomatic Rust API (Recommended)

The recommended approach uses methods on `Screen`, `Window`, `Menu`, `Form`, and `Panel` structs:

```rust
use ncurses::*;

fn main() -> Result<()> {
    let mut screen = Screen::init()?;
    
    // Method-based API
    screen.cbreak()?;
    screen.noecho()?;
    screen.addstr("Hello, World!")?;
    screen.refresh()?;
    screen.getch()?;
    
    Ok(())
}
```

This approach provides:
- Better IDE autocompletion
- Clearer ownership semantics
- More Rust-idiomatic code
- Compile-time safety

### ncurses-Compatible Free Functions

For easier porting of existing C code, ncurses-pure also provides **free functions** that match the traditional ncurses C API signatures:

```rust
use ncurses::*;
use ncurses::menu::*;  // Menu free functions
use ncurses::form::*;  // Form free functions

// These match the C ncurses API:
let menu = new_menu(items);
set_menu_mark(&mut menu, "> ");
post_menu(&mut menu)?;
menu_driver(&mut menu, REQ_DOWN_ITEM)?;

let form = new_form(fields);
set_form_fields(&mut form, new_fields)?;
let fields = form_fields(&form);
```

**Note**: While free functions are provided for compatibility, we encourage migrating to the idiomatic Rust API for new code. The method-based API is more ergonomic and provides better integration with Rust's ownership system.

## Compatibility

ncurses-pure provides API compatibility with **ncurses 6.6**, including:

- **Core curses functions** - Window management, input/output, attributes, colors
- **Wide character support** - Full Unicode via `cchar_t` equivalents
- **Mouse handling** - All standard mouse events and protocols
- **Panels library** - Complete panel deck implementation
- **Menu library** - Full menu system with items, hooks, and drivers
- **Form library** - Complete form system with fields and validation
- **Thread-safe functions** - `use_screen()` and `use_window()` equivalents

### Porting from C ncurses

Most programs using C ncurses can be ported by:

1. Replace `initscr()` with `Screen::init()`
2. Replace `endwin()` with dropping the Screen (or call `screen.endwin()`)
3. Replace window functions like `waddstr(win, s)` with `win.addstr(s)`
4. Handle `Result` return values instead of checking for `ERR`
5. Use the free functions (e.g., `new_menu()`, `form_driver()`) for quick ports

### Intentional Differences

Some low-level functions are provided as no-op stubs for API compatibility:

- **Termcap functions** (`tgetent`, `tgetstr`, etc.) - Termcap is obsolete
- **Terminfo low-level** (`setupterm`, `vidattr`, etc.) - Handled internally
- **Global screen state** (`set_term`) - Each Screen is independent in Rust

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

## Acknowledgments

- Based on the [ncurses](https://invisible-island.net/ncurses/) library by Thomas E. Dickey and [contributors](https://invisible-island.net/ncurses/ncurses-license.html#players)
- XSI Curses standard from The Open Group
