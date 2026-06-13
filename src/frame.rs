//! Pure frame computation: turning numbers, floats, timers and symbols into the
//! five raw bytes the shift-register chain expects.
//!
//! No hardware access lives here, which keeps every formatting rule
//! host-testable. The index arithmetic mirrors the original Arduino library
//! exactly — it is validated against the physical glass (whose decimal point
//! sits to the *left* of its digit), so the byte values are the source of
//! truth, not any imagined left-to-right string.

use crate::segments::{COLON_RIGHT, DOT, FRAME_LEN, GLYPH, MINUS, NUMBER_1, Symbol};

/// Inclusive range the 4½ digits can represent.
const MIN: i16 = -19999;
const MAX: i16 = 19999;

/// Segment G only (the middle bar), shown on all four digits for overflow.
const OVERFLOW_GLYPH: u8 = 0x40;

/// The raw display frame: `[symbol, thousands, hundreds, tens, units]`.
///
/// Digit `i` (0 = units … 3 = thousands) lives at `bytes[4 - i]`. The symbol
/// byte is persistent state: indicator bits set via [`Frame::set_symbol`]
/// survive numeric updates because the numeric methods only touch the bits they
/// own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Frame([u8; FRAME_LEN]);

impl Frame {
    /// A blank frame (all segments off).
    pub(crate) const fn new() -> Self {
        Self([0x00; FRAME_LEN])
    }

    /// The raw bytes, ready to shift out (index 0 first, MSB first).
    pub(crate) fn as_bytes(&self) -> &[u8; FRAME_LEN] {
        &self.0
    }

    pub(crate) fn clear(&mut self) {
        self.0 = [0x00; FRAME_LEN];
    }

    pub(crate) fn all_on(&mut self) {
        self.0 = [0xFF; FRAME_LEN];
    }

    pub(crate) fn set_integer(&mut self, value: i16) {
        if !in_range(value) {
            self.overflow();
            return;
        }
        self.set_symbol_bit(MINUS, value < 0);
        self.write_digits(value.unsigned_abs());
        self.suppress_leading_zeros();
    }

    pub(crate) fn set_float(&mut self, value: f32, decimals: u8) {
        let decimals = decimals.min(4);
        let multiplier = [1.0_f32, 10.0, 100.0, 1000.0, 10000.0][decimals as usize];
        // `as i16` saturates (no UB); the range check below rejects overflow.
        let rounded = libm::roundf(value * multiplier) as i16;

        if !in_range(rounded) {
            self.overflow();
            return;
        }
        if decimals == 0 {
            self.set_integer(rounded);
            return;
        }
        self.set_symbol_bit(MINUS, rounded < 0);
        self.write_digits(rounded.unsigned_abs());
        self.set_decimal_dot(decimals);
        self.suppress_leading_zeros();
    }

    pub(crate) fn set_timer(&mut self, seconds: u16) {
        let secs = seconds % 60;
        let mins = seconds / 60;
        let digits = [secs % 10, secs / 10 % 10, mins % 10, mins / 10 % 10];

        self.0[0] |= COLON_RIGHT;
        for (i, &digit) in digits.iter().enumerate() {
            self.0[4 - i] = GLYPH[digit as usize];
        }
    }

    pub(crate) fn set_symbol(&mut self, symbol: Symbol, on: bool) {
        self.set_symbol_bit(symbol.mask(), on);
    }

    // --- internals ----------------------------------------------------------

    /// Write the four full digits of `abs` and the half-digit "1".
    fn write_digits(&mut self, abs: u16) {
        let digits = [abs % 10, abs / 10 % 10, abs / 100 % 10, abs / 1000 % 10];
        for (i, &digit) in digits.iter().enumerate() {
            self.0[4 - i] = GLYPH[digit as usize];
        }
        self.set_symbol_bit(NUMBER_1, abs / 10000 > 0);
    }

    /// Blank leading-zero digits left to right, stopping at the first non-zero
    /// digit or a digit carrying a decimal point. The units digit is never
    /// blanked.
    fn suppress_leading_zeros(&mut self) {
        for i in 1..4 {
            if self.0[i] == GLYPH[0] && (self.0[i + 1] & DOT) == 0 {
                self.0[i] = 0x00;
            } else {
                break;
            }
        }
    }

    /// Place the decimal point for a float with `pos` fractional digits
    /// (`1..=4`). The DP renders to the *left* of its digit on this glass, so a
    /// bare fractional value gets a leading "0." rather than ".".
    fn set_decimal_dot(&mut self, pos: u8) {
        let pos = pos as usize;
        self.0[FRAME_LEN - pos] |= DOT;
        if pos != 4 && self.0[FRAME_LEN - 1 - pos] == 0x00 {
            self.0[FRAME_LEN - 1 - pos] = GLYPH[0];
        }
    }

    /// Out-of-range pattern: clear the symbol byte, four middle bars on the digits.
    fn overflow(&mut self) {
        self.0[0] = 0x00;
        for digit in &mut self.0[1..] {
            *digit = OVERFLOW_GLYPH;
        }
    }

    /// Set or clear bits in the symbol byte without disturbing the others —
    /// this is what lets indicator symbols persist across numeric updates.
    fn set_symbol_bit(&mut self, mask: u8, on: bool) {
        if on {
            self.0[0] |= mask;
        } else {
            self.0[0] &= !mask;
        }
    }
}

fn in_range(value: i16) -> bool {
    (MIN..=MAX).contains(&value)
}

#[cfg(test)]
mod tests {
    // `3.14` etc. are sample fixtures, not approximations of PI.
    #![allow(clippy::approx_constant)]

    use super::*;

    /// Render a fresh frame after applying `build`, returning the raw bytes.
    fn render(build: impl FnOnce(&mut Frame)) -> [u8; FRAME_LEN] {
        let mut frame = Frame::new();
        build(&mut frame);
        *frame.as_bytes()
    }

    #[test]
    fn integers() {
        assert_eq!(render(|f| f.set_integer(0)), [0x00, 0x00, 0x00, 0x00, 0x3F]);
        assert_eq!(render(|f| f.set_integer(5)), [0x00, 0x00, 0x00, 0x00, 0x6D]);
        assert_eq!(render(|f| f.set_integer(-5)), [0x01, 0x00, 0x00, 0x00, 0x6D]);
        assert_eq!(render(|f| f.set_integer(-1)), [0x01, 0x00, 0x00, 0x00, 0x06]);
        assert_eq!(render(|f| f.set_integer(123)), [0x00, 0x00, 0x06, 0x5B, 0x4F]);
        assert_eq!(render(|f| f.set_integer(-123)), [0x01, 0x00, 0x06, 0x5B, 0x4F]);
        assert_eq!(render(|f| f.set_integer(1000)), [0x00, 0x06, 0x3F, 0x3F, 0x3F]);
        assert_eq!(render(|f| f.set_integer(9999)), [0x00, 0x6F, 0x6F, 0x6F, 0x6F]);
    }

    #[test]
    fn half_digit_and_overflow() {
        // Exact powers blank the interior zeros (preserved original quirk).
        assert_eq!(render(|f| f.set_integer(10000)), [0x10, 0x00, 0x00, 0x00, 0x3F]);
        assert_eq!(render(|f| f.set_integer(19999)), [0x10, 0x6F, 0x6F, 0x6F, 0x6F]);
        assert_eq!(render(|f| f.set_integer(-19999)), [0x11, 0x6F, 0x6F, 0x6F, 0x6F]);
        assert_eq!(render(|f| f.set_integer(20000)), [0x00, 0x40, 0x40, 0x40, 0x40]);
        assert_eq!(render(|f| f.set_integer(-20000)), [0x00, 0x40, 0x40, 0x40, 0x40]);
        assert_eq!(render(|f| f.set_integer(i16::MAX)), [0x00, 0x40, 0x40, 0x40, 0x40]);
        assert_eq!(render(|f| f.set_integer(i16::MIN)), [0x00, 0x40, 0x40, 0x40, 0x40]);
    }

    #[test]
    fn floats() {
        assert_eq!(render(|f| f.set_float(0.0, 1)), [0x00, 0x00, 0x00, 0x3F, 0xBF]);
        assert_eq!(render(|f| f.set_float(1.5, 1)), [0x00, 0x00, 0x00, 0x06, 0xED]);
        assert_eq!(render(|f| f.set_float(-1.5, 1)), [0x01, 0x00, 0x00, 0x06, 0xED]);
        assert_eq!(render(|f| f.set_float(3.14, 2)), [0x00, 0x00, 0x4F, 0x86, 0x66]);
        assert_eq!(render(|f| f.set_float(12.34, 2)), [0x00, 0x06, 0x5B, 0xCF, 0x66]);
        assert_eq!(render(|f| f.set_float(0.0, 2)), [0x00, 0x00, 0x3F, 0xBF, 0x3F]);
        assert_eq!(render(|f| f.set_float(-0.05, 2)), [0x01, 0x00, 0x3F, 0xBF, 0x6D]);
        assert_eq!(render(|f| f.set_float(19.999, 3)), [0x10, 0x6F, 0xEF, 0x6F, 0x6F]);
    }

    #[test]
    fn float_decimals_zero_delegates_to_integer() {
        assert_eq!(render(|f| f.set_float(42.4, 0)), render(|f| f.set_integer(42)));
        assert_eq!(render(|f| f.set_float(-42.6, 0)), render(|f| f.set_integer(-43)));
    }

    #[test]
    fn float_decimals_above_four_are_clamped() {
        assert_eq!(render(|f| f.set_float(1.5, 9)), render(|f| f.set_float(1.5, 4)));
        assert_eq!(render(|f| f.set_float(1.5, 4)), [0x10, 0xED, 0x3F, 0x3F, 0x3F]);
    }

    #[test]
    fn float_out_of_range_overflows() {
        assert_eq!(render(|f| f.set_float(2.5, 4)), [0x00, 0x40, 0x40, 0x40, 0x40]);
        assert_eq!(render(|f| f.set_float(99999.0, 0)), [0x00, 0x40, 0x40, 0x40, 0x40]);
    }

    #[test]
    fn timer() {
        assert_eq!(render(|f| f.set_timer(0)), [0x08, 0x3F, 0x3F, 0x3F, 0x3F]);
        assert_eq!(render(|f| f.set_timer(65)), [0x08, 0x3F, 0x06, 0x3F, 0x6D]);
        assert_eq!(render(|f| f.set_timer(599)), [0x08, 0x3F, 0x6F, 0x6D, 0x6F]);
        // No upper bound: tens-of-minutes wraps via `% 10`.
        assert_eq!(render(|f| f.set_timer(3600)), [0x08, 0x7D, 0x3F, 0x3F, 0x3F]);
    }

    #[test]
    fn clear_and_all_on() {
        assert_eq!(render(|f| f.clear()), [0x00; FRAME_LEN]);
        assert_eq!(render(|f| f.all_on()), [0xFF; FRAME_LEN]);
    }

    #[test]
    fn symbols_set_expected_bits() {
        assert_eq!(render(|f| f.set_symbol(Symbol::LoBat, true)), [0x40, 0, 0, 0, 0]);
        assert_eq!(render(|f| f.set_symbol(Symbol::ColonLeft, true)), [0x02, 0, 0, 0, 0]);
        assert_eq!(render(|f| f.set_symbol(Symbol::ColonMiddle, true)), [0x04, 0, 0, 0, 0]);
        assert_eq!(render(|f| f.set_symbol(Symbol::ColonRight, true)), [0x08, 0, 0, 0, 0]);
    }

    #[test]
    fn symbols_persist_across_numeric_updates() {
        let mut f = Frame::new();
        f.set_symbol(Symbol::LoBat, true);
        f.set_integer(5);
        assert_eq!(*f.as_bytes(), [0x40, 0x00, 0x00, 0x00, 0x6D]);
        f.set_float(3.14, 2);
        assert_eq!(*f.as_bytes(), [0x40, 0x00, 0x4F, 0x86, 0x66]);
        f.set_timer(65);
        assert_eq!(*f.as_bytes(), [0x48, 0x3F, 0x06, 0x3F, 0x6D]);
    }

    #[test]
    fn toggling_one_symbol_off_leaves_others_set() {
        let mut f = Frame::new();
        f.set_symbol(Symbol::LoBat, true);
        f.set_symbol(Symbol::ColonLeft, true);
        f.set_symbol(Symbol::LoBat, false);
        assert_eq!(f.as_bytes()[0], 0x02);
    }

    #[test]
    fn overflow_and_clear_wipe_indicators() {
        let mut f = Frame::new();
        f.set_symbol(Symbol::LoBat, true);
        f.set_integer(20000); // overflow
        assert_eq!(*f.as_bytes(), [0x00, 0x40, 0x40, 0x40, 0x40]);

        f.set_symbol(Symbol::LoBat, true);
        f.clear();
        assert_eq!(*f.as_bytes(), [0x00; FRAME_LEN]);
    }
}
