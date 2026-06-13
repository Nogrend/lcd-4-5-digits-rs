#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! `no_std` driver for a 4½-digit static LCD driven by a chain of 74HC595 shift
//! registers, ported from the Arduino library
//! [`Nogrend/lcd-4-5-digits`](https://github.com/Nogrend/lcd-4-5-digits).
//!
//! The MCU shifts five bytes out three wires (latch, clock, data), MSB first.
//! Because LCD segments must be driven with AC, a 50–60 Hz backplane square
//! wave is XOR-ed with each segment bit *in hardware*; this crate only decides
//! which segments are on and shifts the bytes out.
//!
//! # Example
//!
//! ```
//! # use embedded_hal::digital::{ErrorType, OutputPin};
//! # struct Pin;
//! # impl ErrorType for Pin { type Error = core::convert::Infallible; }
//! # impl OutputPin for Pin {
//! #     fn set_low(&mut self) -> Result<(), Self::Error> { Ok(()) }
//! #     fn set_high(&mut self) -> Result<(), Self::Error> { Ok(()) }
//! # }
//! use lcd_4_5_digits::{Lcd45Digits, Symbol};
//!
//! // Pins come from your HAL, already configured as push-pull outputs.
//! # let (latch, clock, data) = (Pin, Pin, Pin);
//! let mut lcd = Lcd45Digits::new_bitbang(latch, clock, data);
//!
//! lcd.init()?; // blank the panel
//! lcd.set_integer(1234)?; // 1234
//! lcd.set_float(3.14, 2)?; // 3.14
//! lcd.set_timer(90)?; // 01:30
//! lcd.set_symbol(Symbol::LoBat, true)?; // low-battery indicator, persists
//! # Ok::<(), core::convert::Infallible>(())
//! ```

#[cfg(test)]
extern crate std;

mod backend;
mod frame;
mod segments;

pub use backend::{BitBang, FrameWriter};
pub use segments::{FRAME_LEN, Symbol};

use embedded_hal::digital::OutputPin;
use frame::Frame;

/// Driver for the 4½-digit static LCD.
///
/// Generic over any [`FrameWriter`]; use [`Lcd45Digits::new_bitbang`] for the
/// bundled three-pin software backend.
#[derive(Debug)]
pub struct Lcd45Digits<W: FrameWriter> {
    frame: Frame,
    writer: W,
}

impl<W: FrameWriter> Lcd45Digits<W> {
    /// Wrap a [`FrameWriter`]. The display is not touched until the first
    /// drawing call; use [`init`](Self::init) to blank it explicitly.
    pub fn new(writer: W) -> Self {
        Self {
            frame: Frame::new(),
            writer,
        }
    }

    /// Consume the driver and return the underlying [`FrameWriter`].
    ///
    /// Lets you reclaim the transport — e.g. then call [`BitBang::release`] to
    /// get the GPIO pins back when you need them elsewhere.
    pub fn release(self) -> W {
        self.writer
    }

    /// Blank the panel and flush.
    pub fn init(&mut self) -> Result<(), W::Error> {
        self.clear()
    }

    /// Turn every segment off.
    pub fn clear(&mut self) -> Result<(), W::Error> {
        self.frame.clear();
        self.flush()
    }

    /// Turn every segment on (useful for a display self-test).
    pub fn all_on(&mut self) -> Result<(), W::Error> {
        self.frame.all_on();
        self.flush()
    }

    /// Show a signed integer.
    ///
    /// Values outside `-19999..=19999` show the overflow pattern (four dashes).
    pub fn set_integer(&mut self, value: i16) -> Result<(), W::Error> {
        self.frame.set_integer(value);
        self.flush()
    }

    /// Show a value with `decimals` fractional digits (clamped to `0..=4`).
    ///
    /// The rounded value must fit `-19999..=19999`, otherwise the overflow
    /// pattern is shown.
    pub fn set_float(&mut self, value: f32, decimals: u8) -> Result<(), W::Error> {
        self.frame.set_float(value, decimals);
        self.flush()
    }

    /// Show a `MM:SS` timer for the given number of seconds.
    pub fn set_timer(&mut self, seconds: u16) -> Result<(), W::Error> {
        self.frame.set_timer(seconds);
        self.flush()
    }

    /// Set or clear an indicator [`Symbol`].
    ///
    /// Indicator bits persist across later numeric updates; only
    /// [`clear`](Self::clear), [`all_on`](Self::all_on) and an overflow change
    /// them.
    pub fn set_symbol(&mut self, symbol: Symbol, on: bool) -> Result<(), W::Error> {
        self.frame.set_symbol(symbol, on);
        self.flush()
    }

    fn flush(&mut self) -> Result<(), W::Error> {
        self.writer.write_frame(self.frame.as_bytes())
    }
}

impl<Latch, Clock, Data, E> Lcd45Digits<BitBang<Latch, Clock, Data>>
where
    Latch: OutputPin<Error = E>,
    Clock: OutputPin<Error = E>,
    Data: OutputPin<Error = E>,
{
    /// Build a driver using the software bit-bang backend, in the same
    /// `(latch, clock, data)` order as the original library's constructor.
    ///
    /// All three pins must share one `OutputPin::Error` type — the norm for
    /// every common HAL, where GPIO writes report a single error type.
    pub fn new_bitbang(latch: Latch, clock: Clock, data: Data) -> Self {
        Self::new(BitBang::new(latch, clock, data))
    }
}
