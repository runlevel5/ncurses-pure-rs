//! ACS (Alternate Character Set) characters for ncurses-pure.
//!
//! This module provides Unicode box-drawing characters as alternatives to
//! the traditional VT100 line-drawing characters. Modern terminals support
//! these characters directly without needing alternate character set mode.
//!
//! The `ACS_*` constants are `char`s. Use them directly with strings and the
//! wide-character functions. For the functions that take a `ChType`, such as
//! `addch`, `hline` and `border`, convert them with [`acs_char`]: a cast
//! (`ACS_HLINE as ChType`) does not work, because only the low byte of a
//! `ChType` holds its character.

use crate::attr::A_ALTCHARSET;
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

/// The VT100 line-drawing codes and the characters they stand for.
///
/// A code with `A_ALTCHARSET` is how a `ChType` carries one of these
/// characters: only the low byte of a `ChType` holds its character, which
/// is too small for the Unicode value.
const VT100_CODES: [(u8, char); 32] = [
    (b'l', ACS_ULCORNER),
    (b'm', ACS_LLCORNER),
    (b'k', ACS_URCORNER),
    (b'j', ACS_LRCORNER),
    (b't', ACS_LTEE),
    (b'u', ACS_RTEE),
    (b'w', ACS_TTEE),
    (b'v', ACS_BTEE),
    (b'q', ACS_HLINE),
    (b'x', ACS_VLINE),
    (b'n', ACS_PLUS),
    (b'o', ACS_S1),
    (b'p', ACS_S3),
    (b'r', ACS_S7),
    (b's', ACS_S9),
    (b'`', ACS_DIAMOND),
    (b'a', ACS_CKBOARD),
    (b'f', ACS_DEGREE),
    (b'g', ACS_PLMINUS),
    (b'~', ACS_BULLET),
    (b',', ACS_LARROW),
    (b'+', ACS_RARROW),
    (b'.', ACS_DARROW),
    (b'-', ACS_UARROW),
    (b'h', ACS_BOARD),
    (b'i', ACS_LANTERN),
    (b'0', ACS_BLOCK),
    (b'y', ACS_LEQUAL),
    (b'z', ACS_GEQUAL),
    (b'{', ACS_PI),
    (b'|', ACS_NEQUAL),
    (b'}', ACS_STERLING),
];

/// The VT100 line-drawing code for an ACS character.
///
/// The double, heavy and rounded variants have no code of their own and
/// share the code of the plain character.
fn vt100_code(c: char) -> Option<u8> {
    let plain = match c {
        '╔' | '┏' | '╭' => ACS_ULCORNER,
        '╚' | '┗' | '╰' => ACS_LLCORNER,
        '╗' | '┓' | '╮' => ACS_URCORNER,
        '╝' | '┛' | '╯' => ACS_LRCORNER,
        '╠' | '┣' => ACS_LTEE,
        '╣' | '┫' => ACS_RTEE,
        '╦' | '┳' => ACS_TTEE,
        '╩' | '┻' => ACS_BTEE,
        '═' | '━' => ACS_HLINE,
        '║' | '┃' => ACS_VLINE,
        '╬' | '╋' => ACS_PLUS,
        _ => c,
    };
    VT100_CODES
        .iter()
        .find(|(_, acs)| *acs == plain)
        .map(|(code, _)| *code)
}

/// Convert an ACS character to a `ChType` for `addch`, `hline`, `border`
/// and the other functions that take one.
///
/// The result is the character's VT100 line-drawing code with
/// `A_ALTCHARSET`, which those functions draw as the character itself.
/// Casting the constant instead (`ACS_HLINE as ChType`) does not work:
/// only the low byte of a `ChType` holds its character.
///
/// The double, heavy and rounded variants (`ACS_D_*`, `ACS_HEAVY_*`,
/// `ACS_ROUND_*`) have no code and come out as the plain character. To draw
/// them, use the wide-character functions with a `CCharT`.
///
/// A character that is not an ACS character is returned as it is when it
/// fits in a `ChType` (U+00FF and below), and as `?` otherwise.
pub fn acs_char(c: char) -> ChType {
    match vt100_code(c) {
        Some(code) => ChType::from(code) | A_ALTCHARSET,
        None => ChType::from(u8::try_from(c).unwrap_or(b'?')),
    }
}

/// The ACS character a VT100 line-drawing code stands for.
///
/// This is the reverse of [`acs_char`]: `acs_unicode(b'q')` is `ACS_HLINE`.
pub fn acs_unicode(code: u8) -> Option<char> {
    VT100_CODES
        .iter()
        .find(|(vt100, _)| *vt100 == code)
        .map(|(_, c)| *c)
}

/// The VT100 line-drawing code, with `A_ALTCHARSET`, of a character read
/// back from a window, if it is an ACS character.
pub(crate) fn acs_chtype(c: char) -> Option<ChType> {
    VT100_CODES
        .iter()
        .find(|(_, acs)| *acs == c)
        .map(|(code, _)| ChType::from(*code) | A_ALTCHARSET)
}

/// Get the ACS map.
///
/// The entry for a VT100 line-drawing code is that code with
/// `A_ALTCHARSET`, as [`acs_char`] returns it; the other entries are 0.
pub fn acs_map() -> [ChType; 128] {
    let mut map = [0 as ChType; 128];
    for (code, _) in VT100_CODES {
        map[usize::from(code)] = ChType::from(code) | A_ALTCHARSET;
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
        assert_eq!(acs_char(ACS_HLINE), ChType::from(b'q') | A_ALTCHARSET);
        assert_eq!(acs_char(ACS_ULCORNER), ChType::from(b'l') | A_ALTCHARSET);
        assert_eq!(acs_char(ACS_STERLING), ChType::from(b'}') | A_ALTCHARSET);
    }

    #[test]
    fn test_acs_char_keeps_the_character_in_the_low_byte() {
        for (code, c) in VT100_CODES {
            let ch = acs_char(c);
            assert_eq!(ch & crate::attr::A_CHARTEXT, ChType::from(code), "{c}");
            assert_eq!(ch & !crate::attr::A_CHARTEXT, A_ALTCHARSET, "{c}");
        }
    }

    #[test]
    fn test_acs_codes_and_characters_are_distinct() {
        for (i, (code, c)) in VT100_CODES.iter().enumerate() {
            for (other_code, other_c) in &VT100_CODES[i + 1..] {
                assert_ne!(code, other_code);
                assert_ne!(c, other_c);
            }
            assert!(code.is_ascii_graphic());
        }
    }

    #[test]
    fn test_acs_unicode_reverses_acs_char() {
        for (code, c) in VT100_CODES {
            assert_eq!(acs_unicode(code), Some(c));
            assert_eq!(acs_chtype(c), Some(acs_char(c)));
        }
        assert_eq!(acs_unicode(b'A'), None);
        assert_eq!(acs_unicode(0), None);
        assert_eq!(acs_chtype('A'), None);
        assert_eq!(acs_chtype('═'), None);
    }

    #[test]
    fn test_acs_char_variants_fall_back_to_the_plain_character() {
        for (variants, plain) in [
            (
                [ACS_D_ULCORNER, ACS_HEAVY_ULCORNER, ACS_ROUND_ULCORNER],
                ACS_ULCORNER,
            ),
            (
                [ACS_D_LLCORNER, ACS_HEAVY_LLCORNER, ACS_ROUND_LLCORNER],
                ACS_LLCORNER,
            ),
            (
                [ACS_D_URCORNER, ACS_HEAVY_URCORNER, ACS_ROUND_URCORNER],
                ACS_URCORNER,
            ),
            (
                [ACS_D_LRCORNER, ACS_HEAVY_LRCORNER, ACS_ROUND_LRCORNER],
                ACS_LRCORNER,
            ),
        ] {
            for variant in variants {
                assert_eq!(acs_char(variant), acs_char(plain), "{variant}");
            }
        }
        for (double, heavy, plain) in [
            (ACS_D_HLINE, ACS_HEAVY_HLINE, ACS_HLINE),
            (ACS_D_VLINE, ACS_HEAVY_VLINE, ACS_VLINE),
            (ACS_D_LTEE, ACS_HEAVY_LTEE, ACS_LTEE),
            (ACS_D_RTEE, ACS_HEAVY_RTEE, ACS_RTEE),
            (ACS_D_TTEE, ACS_HEAVY_TTEE, ACS_TTEE),
            (ACS_D_BTEE, ACS_HEAVY_BTEE, ACS_BTEE),
            (ACS_D_PLUS, ACS_HEAVY_PLUS, ACS_PLUS),
        ] {
            assert_eq!(acs_char(double), acs_char(plain), "{double}");
            assert_eq!(acs_char(heavy), acs_char(plain), "{heavy}");
        }
    }

    #[test]
    fn test_acs_char_of_other_characters() {
        assert_eq!(acs_char('A'), ChType::from(b'A'));
        assert_eq!(acs_char('é'), 0xE9);
        assert_eq!(acs_char('日'), ChType::from(b'?'));
    }

    #[test]
    fn test_acs_map() {
        let map = acs_map();
        for (code, c) in VT100_CODES {
            assert_eq!(map[usize::from(code)], acs_char(c), "{c}");
        }
        let set = map.iter().filter(|entry| **entry != 0).count();
        assert_eq!(set, VT100_CODES.len());
    }
}
