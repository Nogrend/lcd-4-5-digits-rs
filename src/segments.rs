//! Segment maps, symbol bit masks, and the digit glyph table.
//!
//! Everything here is pure data plus tiny helpers — no I/O — so it is trivially
//! unit-testable on the host.

/// Number of bytes shifted out per display update.
///
/// Byte 0 is the symbol byte; bytes 1..=4 are the four full digits
/// (thousands, hundreds, tens, units).
pub const FRAME_LEN: usize = 5;

// --- Symbol byte (`frame[0]`) bit masks -------------------------------------

/// Leading minus sign. Owned by the numeric methods.
pub(crate) const MINUS: u8 = 0x01;
/// Left colon indicator.
pub(crate) const COLON_LEFT: u8 = 0x02;
/// Middle colon indicator.
pub(crate) const COLON_MIDDLE: u8 = 0x04;
/// Right colon indicator (the `MM:SS` separator used by the timer).
pub(crate) const COLON_RIGHT: u8 = 0x08;
/// Half-digit "1" in the ten-thousands place. Owned by the numeric methods.
pub(crate) const NUMBER_1: u8 = 0x10;
/// Low-battery indicator.
pub(crate) const LOBAT: u8 = 0x40;
/// Decimal point (also the DP bit of any digit byte). Owned by the numeric methods.
pub(crate) const DOT: u8 = 0x80;

/// Seven-segment glyphs for digits `0..=9`.
///
/// Bit layout: `bit0=A, bit1=B, bit2=C, bit3=D, bit4=E, bit5=F, bit6=G, bit7=DP`.
pub(crate) const GLYPH: [u8; 10] = [
    0x3F, // 0
    0x06, // 1
    0x5B, // 2
    0x4F, // 3
    0x66, // 4
    0x6D, // 5
    0x7D, // 6
    0x07, // 7
    0x7F, // 8
    0x6F, // 9
];

/// A user-settable indicator segment.
///
/// These overlay the numeric display and **persist** across
/// [`set_integer`](crate::Lcd45Digits::set_integer),
/// [`set_float`](crate::Lcd45Digits::set_float) and
/// [`set_timer`](crate::Lcd45Digits::set_timer); they are cleared only by
/// [`clear`](crate::Lcd45Digits::clear), [`all_on`](crate::Lcd45Digits::all_on)
/// or an out-of-range overflow.
///
/// The minus sign, the half-digit "1" and the decimal point are deliberately
/// not exposed here — they are driven by the numeric methods.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Symbol {
    /// Low-battery glyph.
    LoBat,
    /// Left colon.
    ColonLeft,
    /// Middle colon.
    ColonMiddle,
    /// Right colon (the same segment the timer uses as its `MM:SS` separator).
    ColonRight,
}

impl Symbol {
    /// The symbol-byte bit mask for this indicator.
    pub(crate) const fn mask(self) -> u8 {
        match self {
            Symbol::LoBat => LOBAT,
            Symbol::ColonLeft => COLON_LEFT,
            Symbol::ColonMiddle => COLON_MIDDLE,
            Symbol::ColonRight => COLON_RIGHT,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glyph_table_is_correct() {
        assert_eq!(GLYPH.len(), 10);
        assert_eq!(GLYPH[0], 0x3F);
        assert_eq!(GLYPH[7], 0x07);
        assert_eq!(GLYPH[8], 0x7F);
        assert_eq!(GLYPH[9], 0x6F);
    }

    #[test]
    fn symbol_masks_match_the_datasheet() {
        assert_eq!(Symbol::LoBat.mask(), 0x40);
        assert_eq!(Symbol::ColonLeft.mask(), 0x02);
        assert_eq!(Symbol::ColonMiddle.mask(), 0x04);
        assert_eq!(Symbol::ColonRight.mask(), 0x08);
    }
}
