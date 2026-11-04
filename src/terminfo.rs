//! Minimal reader for compiled terminfo entries.
//!
//! Only the numeric capabilities needed for colour support are read:
//! `max_colors` and `max_pairs`. Both the legacy format (magic `0o432`,
//! 16-bit numbers) and the extended number format (magic `0o1036`, 32-bit
//! numbers) are understood. Everything else in the entry is skipped.

use std::path::{Path, PathBuf};

/// Legacy compiled format: 16-bit numeric capabilities
const MAGIC_LEGACY: u16 = 0o432;
/// Extended compiled format: 32-bit numeric capabilities
const MAGIC_EXTENDED: u16 = 0o1036;

/// Position of `max_colors` in the numeric capability table
const INDEX_MAX_COLORS: usize = 13;
/// Position of `max_pairs` in the numeric capability table
const INDEX_MAX_PAIRS: usize = 14;

/// Colour-related numeric capabilities of a terminfo entry
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ColorCapabilities {
    /// `max_colors`, when the entry defines it
    pub colors: Option<i32>,
    /// `max_pairs`, when the entry defines it
    pub pairs: Option<i32>,
}

/// Look up the colour capabilities of `term` in the terminfo database.
///
/// Returns `None` when no compiled entry can be found or parsed, in which
/// case the caller falls back to its own knowledge of the terminal name.
pub(crate) fn color_capabilities(term: &str) -> Option<ColorCapabilities> {
    let path = find_entry(term, &search_dirs())?;
    let bytes = std::fs::read(path).ok()?;
    parse_color_capabilities(&bytes)
}

/// The directories ncurses searches, in order
fn search_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Ok(dir) = std::env::var("TERMINFO") {
        dirs.push(PathBuf::from(dir));
    }
    if let Some(home) = std::env::var_os("HOME") {
        dirs.push(Path::new(&home).join(".terminfo"));
    }
    if let Ok(list) = std::env::var("TERMINFO_DIRS") {
        dirs.extend(list.split(':').filter(|d| !d.is_empty()).map(PathBuf::from));
    }
    for dir in [
        "/etc/terminfo",
        "/lib/terminfo",
        "/usr/share/terminfo",
        "/usr/lib/terminfo",
        "/usr/local/share/terminfo",
        "/opt/homebrew/share/terminfo",
    ] {
        dirs.push(PathBuf::from(dir));
    }
    dirs
}

/// Find the compiled entry for `term` under the given directories.
///
/// Entries live in a subdirectory named after the first character of the
/// terminal name, either literally (`x/xterm`) or, on macOS, as that
/// character's hex code (`78/xterm`).
fn find_entry(term: &str, dirs: &[PathBuf]) -> Option<PathBuf> {
    let first = term.chars().next()?;
    if term.contains('/') || term.contains("..") {
        return None;
    }
    let subdirs = [first.to_string(), format!("{:x}", first as u32)];
    dirs.iter()
        .flat_map(|dir| subdirs.iter().map(move |sub| dir.join(sub).join(term)))
        .find(|path| path.is_file())
}

/// Parse the colour capabilities out of a compiled terminfo entry
fn parse_color_capabilities(bytes: &[u8]) -> Option<ColorCapabilities> {
    let header = |i: usize| -> Option<usize> {
        let v = i16::from_le_bytes([*bytes.get(i * 2)?, *bytes.get(i * 2 + 1)?]);
        usize::try_from(v).ok()
    };
    let magic = u16::from_le_bytes([*bytes.first()?, *bytes.get(1)?]);
    let number_size = match magic {
        MAGIC_LEGACY => 2,
        MAGIC_EXTENDED => 4,
        _ => return None,
    };
    let name_size = header(1)?;
    let bool_count = header(2)?;
    let num_count = header(3)?;

    // The numbers start after the names and booleans, padded to an even offset
    let mut offset = 12 + name_size + bool_count;
    if offset % 2 == 1 {
        offset += 1;
    }

    let number = |index: usize| -> Option<i32> {
        if index >= num_count {
            return None;
        }
        let start = offset + index * number_size;
        let raw = bytes.get(start..start + number_size)?;
        let value = match number_size {
            2 => i32::from(i16::from_le_bytes([raw[0], raw[1]])),
            _ => i32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]),
        };
        // -1 means absent and -2 means cancelled
        (value >= 0).then_some(value)
    };

    Some(ColorCapabilities {
        colors: number(INDEX_MAX_COLORS),
        pairs: number(INDEX_MAX_PAIRS),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a compiled entry with the given numeric table
    fn entry(magic: u16, name: &str, bools: usize, numbers: &[i32]) -> Vec<u8> {
        let number_size = if magic == MAGIC_LEGACY { 2 } else { 4 };
        let mut out = Vec::new();
        for v in [
            magic as i16,
            (name.len() + 1) as i16,
            bools as i16,
            numbers.len() as i16,
            0,
            0,
        ] {
            out.extend_from_slice(&v.to_le_bytes());
        }
        out.extend_from_slice(name.as_bytes());
        out.push(0);
        out.extend(std::iter::repeat_n(0u8, bools));
        if out.len() % 2 == 1 {
            out.push(0);
        }
        for &n in numbers {
            if number_size == 2 {
                out.extend_from_slice(&(n as i16).to_le_bytes());
            } else {
                out.extend_from_slice(&n.to_le_bytes());
            }
        }
        out
    }

    fn numbers_with(colors: i32, pairs: i32) -> Vec<i32> {
        let mut n = vec![-1; 16];
        n[INDEX_MAX_COLORS] = colors;
        n[INDEX_MAX_PAIRS] = pairs;
        n
    }

    #[test]
    fn reads_legacy_format() {
        let bytes = entry(
            MAGIC_LEGACY,
            "xterm-ghostty|Ghostty",
            29,
            &numbers_with(256, 32767),
        );
        assert_eq!(
            parse_color_capabilities(&bytes),
            Some(ColorCapabilities {
                colors: Some(256),
                pairs: Some(32767)
            })
        );
    }

    #[test]
    fn reads_extended_format_with_32_bit_numbers() {
        let bytes = entry(
            MAGIC_EXTENDED,
            "xterm-direct|direct",
            30,
            &numbers_with(16777216, 65536),
        );
        assert_eq!(
            parse_color_capabilities(&bytes),
            Some(ColorCapabilities {
                colors: Some(16777216),
                pairs: Some(65536)
            })
        );
    }

    #[test]
    fn honours_even_alignment_before_numbers() {
        // name + NUL + bools is odd here, so a pad byte precedes the numbers
        let odd = entry(MAGIC_LEGACY, "ab", 28, &numbers_with(8, 64));
        assert_eq!(parse_color_capabilities(&odd).unwrap().colors, Some(8));
        let even = entry(MAGIC_LEGACY, "abc", 28, &numbers_with(16, 256));
        assert_eq!(parse_color_capabilities(&even).unwrap().colors, Some(16));
    }

    #[test]
    fn absent_and_cancelled_values_are_none() {
        let absent = entry(MAGIC_LEGACY, "vt100", 29, &numbers_with(-1, -1));
        assert_eq!(parse_color_capabilities(&absent).unwrap().colors, None);
        let cancelled = entry(MAGIC_LEGACY, "vt100", 29, &numbers_with(-2, -2));
        assert_eq!(parse_color_capabilities(&cancelled).unwrap().colors, None);
        let short = entry(MAGIC_LEGACY, "tiny", 29, &[80, 0, 24]);
        assert_eq!(parse_color_capabilities(&short).unwrap().colors, None);
    }

    #[test]
    fn rejects_unknown_magic_and_truncated_data() {
        assert_eq!(parse_color_capabilities(&[0, 0, 0, 0]), None);
        assert_eq!(parse_color_capabilities(&[]), None);
        let mut bytes = entry(MAGIC_LEGACY, "xterm", 29, &numbers_with(8, 64));
        bytes.truncate(20);
        assert_eq!(parse_color_capabilities(&bytes).unwrap().colors, None);
    }

    #[test]
    fn finds_entries_in_letter_and_hex_subdirectories() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("x")).unwrap();
        std::fs::create_dir_all(dir.path().join("78")).unwrap();
        std::fs::write(dir.path().join("x").join("xterm-foo"), b"x").unwrap();
        std::fs::write(dir.path().join("78").join("xterm-bar"), b"x").unwrap();
        let dirs = vec![dir.path().to_path_buf()];
        assert!(find_entry("xterm-foo", &dirs).is_some());
        assert!(find_entry("xterm-bar", &dirs).is_some());
        assert!(find_entry("xterm-missing", &dirs).is_none());
        assert!(find_entry("../x/xterm-foo", &dirs).is_none());
        assert!(find_entry("", &dirs).is_none());
    }
}
