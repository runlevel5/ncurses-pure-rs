//! ACS (Alternate Character Set) characters for ncurses-pure.
//!
//! This module provides Unicode box-drawing characters as alternatives to
//! the traditional VT100 line-drawing characters. Modern terminals support
//! these characters directly without needing alternate character set mode.

use crate::types::ChType;

// ============================================================================
// Unicode Box-Drawing Characters
// ============================================================================

// These are the Unicode characters that correspond to VT100 line-drawing
// characters. We use the actual Unicode values instead of ASCII fallbacks.

/// Upper left corner: ┌
pub const ACS_ULCORNER: char = '┌';
/// Lower left corner: └
pub const ACS_LLCORNER: char = '└';
/// Upper right corner: ┐
pub const ACS_URCORNER: char = '┐';
/// Lower right corner: ┘
pub const ACS_LRCORNER: char = '┘';
/// Tee pointing left: ┤
pub const ACS_RTEE: char = '┤';
/// Tee pointing right: ├
pub const ACS_LTEE: char = '├';
/// Tee pointing down: ┬
pub const ACS_TTEE: char = '┬';
/// Tee pointing up: ┴
pub const ACS_BTEE: char = '┴';
/// Horizontal line: ─
pub const ACS_HLINE: char = '─';
/// Vertical line: │
pub const ACS_VLINE: char = '│';
/// Large plus or crossover: ┼
pub const ACS_PLUS: char = '┼';

// Additional ACS characters

/// Scan line 1 (top): ⎺
pub const ACS_S1: char = '⎺';
/// Scan line 3: ⎻
pub const ACS_S3: char = '⎻';
/// Scan line 7: ⎼
pub const ACS_S7: char = '⎼';
/// Scan line 9 (bottom): ⎽
pub const ACS_S9: char = '⎽';

/// Diamond: ◆
pub const ACS_DIAMOND: char = '◆';
/// Checkerboard (stipple): ▒
pub const ACS_CKBOARD: char = '▒';
/// Degree symbol: °
pub const ACS_DEGREE: char = '°';
/// Plus/minus: ±
pub const ACS_PLMINUS: char = '±';
/// Bullet: ·
pub const ACS_BULLET: char = '·';

/// Arrow pointing left: ←
pub const ACS_LARROW: char = '←';
/// Arrow pointing right: →
pub const ACS_RARROW: char = '→';
/// Arrow pointing down: ↓
pub const ACS_DARROW: char = '↓';
/// Arrow pointing up: ↑
pub const ACS_UARROW: char = '↑';

/// Board of squares: ▓
pub const ACS_BOARD: char = '▓';
/// Lantern symbol: §
pub const ACS_LANTERN: char = '§';
/// Solid square block: █
pub const ACS_BLOCK: char = '█';

/// Less-than-or-equal-to: ≤
pub const ACS_LEQUAL: char = '≤';
/// Greater-than-or-equal-to: ≥
pub const ACS_GEQUAL: char = '≥';
/// Pi: π
pub const ACS_PI: char = 'π';
/// Not-equal: ≠
pub const ACS_NEQUAL: char = '≠';
/// Sterling (UK pound): £
pub const ACS_STERLING: char = '£';

// ============================================================================
// Double-line box drawing characters (extensions)
// ============================================================================

/// Double upper left corner: ╔
pub const ACS_D_ULCORNER: char = '╔';
/// Double lower left corner: ╚
pub const ACS_D_LLCORNER: char = '╚';
/// Double upper right corner: ╗
pub const ACS_D_URCORNER: char = '╗';
/// Double lower right corner: ╝
pub const ACS_D_LRCORNER: char = '╝';
/// Double horizontal line: ═
pub const ACS_D_HLINE: char = '═';
/// Double vertical line: ║
pub const ACS_D_VLINE: char = '║';
/// Double tee pointing left: ╣
pub const ACS_D_RTEE: char = '╣';
/// Double tee pointing right: ╠
pub const ACS_D_LTEE: char = '╠';
/// Double tee pointing down: ╦
pub const ACS_D_TTEE: char = '╦';
/// Double tee pointing up: ╩
pub const ACS_D_BTEE: char = '╩';
/// Double plus/crossover: ╬
pub const ACS_D_PLUS: char = '╬';

// ============================================================================
// Heavy/thick line box drawing characters (extensions)
// ============================================================================

/// Heavy upper left corner: ┏
pub const ACS_HEAVY_ULCORNER: char = '┏';
/// Heavy lower left corner: ┗
pub const ACS_HEAVY_LLCORNER: char = '┗';
/// Heavy upper right corner: ┓
pub const ACS_HEAVY_URCORNER: char = '┓';
/// Heavy lower right corner: ┛
pub const ACS_HEAVY_LRCORNER: char = '┛';
/// Heavy horizontal line: ━
pub const ACS_HEAVY_HLINE: char = '━';
/// Heavy vertical line: ┃
pub const ACS_HEAVY_VLINE: char = '┃';
/// Heavy tee pointing left: ┫
pub const ACS_HEAVY_RTEE: char = '┫';
/// Heavy tee pointing right: ┣
pub const ACS_HEAVY_LTEE: char = '┣';
/// Heavy tee pointing down: ┳
pub const ACS_HEAVY_TTEE: char = '┳';
/// Heavy tee pointing up: ┻
pub const ACS_HEAVY_BTEE: char = '┻';
/// Heavy plus/crossover: ╋
pub const ACS_HEAVY_PLUS: char = '╋';

// ============================================================================
// Rounded corner box drawing characters (extensions)
// ============================================================================

/// Rounded upper left corner: ╭
pub const ACS_ROUND_ULCORNER: char = '╭';
/// Rounded lower left corner: ╰
pub const ACS_ROUND_LLCORNER: char = '╰';
/// Rounded upper right corner: ╮
pub const ACS_ROUND_URCORNER: char = '╮';
/// Rounded lower right corner: ╯
pub const ACS_ROUND_LRCORNER: char = '╯';

// ============================================================================
// Helper functions
// ============================================================================

/// Convert an ACS character to a ChType with A_ALTCHARSET.
///
/// For wide character support, this returns the Unicode character directly.
/// For non-wide mode, it returns an ASCII fallback with A_ALTCHARSET.
#[cfg(feature = "wide")]
pub fn acs_char(c: char) -> ChType {
    c as ChType
}

/// Convert an ACS character to a ChType.
///
/// In non-wide mode, returns ASCII fallbacks.
#[cfg(not(feature = "wide"))]
pub fn acs_char(c: char) -> ChType {
    use crate::attr::A_ALTCHARSET;

    // Map Unicode characters to ASCII fallbacks with A_ALTCHARSET
    let ascii = match c {
        '┌' | '╔' | '┏' | '╭' => 'l',
        '└' | '╚' | '┗' | '╰' => 'm',
        '┐' | '╗' | '┓' | '╮' => 'k',
        '┘' | '╝' | '┛' | '╯' => 'j',
        '├' | '╠' | '┣' => 't',
        '┤' | '╣' | '┫' => 'u',
        '┬' | '╦' | '┳' => 'w',
        '┴' | '╩' | '┻' => 'v',
        '─' | '═' | '━' => 'q',
        '│' | '║' | '┃' => 'x',
        '┼' | '╬' | '╋' => 'n',
        '◆' => '`',
        '▒' => 'a',
        '°' => 'f',
        '±' => 'g',
        '·' => '~',
        '←' => ',',
        '→' => '+',
        '↓' => '.',
        '↑' => '-',
        '▓' => 'h',
        '█' => '0',
        '≤' => 'y',
        '≥' => 'z',
        'π' => '{',
        '≠' => '|',
        '£' => '}',
        _ => c,
    };

    (ascii as u8 as ChType) | A_ALTCHARSET
}

/// Get the ACS map for the current terminal.
///
/// This returns an array of ChType values that map ACS indices to
/// terminal-specific character codes.
pub fn acs_map() -> [ChType; 128] {
    let mut map = [0 as ChType; 128];

    // Standard VT100 line-drawing mappings
    // Index is the 'j' through 'z' and special characters

    #[cfg(feature = "wide")]
    {
        // Use Unicode characters directly
        map[b'j' as usize] = ACS_LRCORNER as ChType; // ┘
        map[b'k' as usize] = ACS_URCORNER as ChType; // ┐
        map[b'l' as usize] = ACS_ULCORNER as ChType; // ┌
        map[b'm' as usize] = ACS_LLCORNER as ChType; // └
        map[b'n' as usize] = ACS_PLUS as ChType; // ┼
        map[b'q' as usize] = ACS_HLINE as ChType; // ─
        map[b't' as usize] = ACS_LTEE as ChType; // ├
        map[b'u' as usize] = ACS_RTEE as ChType; // ┤
        map[b'v' as usize] = ACS_BTEE as ChType; // ┴
        map[b'w' as usize] = ACS_TTEE as ChType; // ┬
        map[b'x' as usize] = ACS_VLINE as ChType; // │
        map[b'`' as usize] = ACS_DIAMOND as ChType; // ◆
        map[b'a' as usize] = ACS_CKBOARD as ChType; // ▒
        map[b'f' as usize] = ACS_DEGREE as ChType; // °
        map[b'g' as usize] = ACS_PLMINUS as ChType; // ±
        map[b'~' as usize] = ACS_BULLET as ChType; // ·
        map[b',' as usize] = ACS_LARROW as ChType; // ←
        map[b'+' as usize] = ACS_RARROW as ChType; // →
        map[b'.' as usize] = ACS_DARROW as ChType; // ↓
        map[b'-' as usize] = ACS_UARROW as ChType; // ↑
        map[b'h' as usize] = ACS_BOARD as ChType; // ▓
        map[b'0' as usize] = ACS_BLOCK as ChType; // █
    }

    #[cfg(not(feature = "wide"))]
    {
        use crate::attr::A_ALTCHARSET;
        // Use VT100 alternate character set codes
        map[b'j' as usize] = (b'j' as ChType) | A_ALTCHARSET;
        map[b'k' as usize] = (b'k' as ChType) | A_ALTCHARSET;
        map[b'l' as usize] = (b'l' as ChType) | A_ALTCHARSET;
        map[b'm' as usize] = (b'm' as ChType) | A_ALTCHARSET;
        map[b'n' as usize] = (b'n' as ChType) | A_ALTCHARSET;
        map[b'q' as usize] = (b'q' as ChType) | A_ALTCHARSET;
        map[b't' as usize] = (b't' as ChType) | A_ALTCHARSET;
        map[b'u' as usize] = (b'u' as ChType) | A_ALTCHARSET;
        map[b'v' as usize] = (b'v' as ChType) | A_ALTCHARSET;
        map[b'w' as usize] = (b'w' as ChType) | A_ALTCHARSET;
        map[b'x' as usize] = (b'x' as ChType) | A_ALTCHARSET;
        map[b'`' as usize] = (b'`' as ChType) | A_ALTCHARSET;
        map[b'a' as usize] = (b'a' as ChType) | A_ALTCHARSET;
        map[b'f' as usize] = (b'f' as ChType) | A_ALTCHARSET;
        map[b'g' as usize] = (b'g' as ChType) | A_ALTCHARSET;
        map[b'~' as usize] = (b'~' as ChType) | A_ALTCHARSET;
        map[b',' as usize] = (b',' as ChType) | A_ALTCHARSET;
        map[b'+' as usize] = (b'+' as ChType) | A_ALTCHARSET;
        map[b'.' as usize] = (b'.' as ChType) | A_ALTCHARSET;
        map[b'-' as usize] = (b'-' as ChType) | A_ALTCHARSET;
        map[b'h' as usize] = (b'h' as ChType) | A_ALTCHARSET;
        map[b'0' as usize] = (b'0' as ChType) | A_ALTCHARSET;
    }

    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acs_characters() {
        // Test that ACS characters are the expected Unicode values
        assert_eq!(ACS_ULCORNER, '┌');
        assert_eq!(ACS_LRCORNER, '┘');
        assert_eq!(ACS_HLINE, '─');
        assert_eq!(ACS_VLINE, '│');
        assert_eq!(ACS_PLUS, '┼');
    }

    #[test]
    fn test_acs_char() {
        let ch = acs_char(ACS_HLINE);
        // Just test that it returns something
        assert!(ch != 0);
    }

    #[test]
    fn test_acs_map() {
        let map = acs_map();
        // Check that the map entries for line drawing are set
        assert!(map[b'j' as usize] != 0);
        assert!(map[b'k' as usize] != 0);
        assert!(map[b'l' as usize] != 0);
        assert!(map[b'm' as usize] != 0);
        assert!(map[b'q' as usize] != 0);
        assert!(map[b'x' as usize] != 0);
    }
}
