//! Output transport.
//!
//! The driver talks to the hardware only through [`FrameWriter`]. [`BitBang`] is
//! the bundled software shift-out backend; an SPI backend can be added here
//! later without touching the public driver API.

use crate::segments::FRAME_LEN;
use embedded_hal::digital::OutputPin;

/// Shifts a fully-computed frame out to the display.
pub trait FrameWriter {
    /// Error returned by the underlying transport (e.g. a GPIO error).
    type Error;

    /// Shift all [`FRAME_LEN`] bytes out (byte 0 first, MSB first) and latch
    /// them to the register outputs in a single transfer.
    fn write_frame(&mut self, frame: &[u8; FRAME_LEN]) -> Result<(), Self::Error>;
}

/// Software bit-bang backend driving the 74HC595 chain over three GPIO pins.
///
/// Equivalent to the Arduino `shiftOut(MSBFIRST)` for each byte followed by a
/// single latch pulse.
pub struct BitBang<Latch, Clock, Data> {
    latch: Latch,
    clock: Clock,
    data: Data,
}

impl<Latch, Clock, Data> BitBang<Latch, Clock, Data> {
    /// Build a bit-bang writer from the latch, clock and data pins.
    pub fn new(latch: Latch, clock: Clock, data: Data) -> Self {
        Self { latch, clock, data }
    }
}

impl<Latch, Clock, Data, E> FrameWriter for BitBang<Latch, Clock, Data>
where
    Latch: OutputPin<Error = E>,
    Clock: OutputPin<Error = E>,
    Data: OutputPin<Error = E>,
{
    type Error = E;

    fn write_frame(&mut self, frame: &[u8; FRAME_LEN]) -> Result<(), E> {
        self.latch.set_low()?;
        for &byte in frame {
            for bit in (0..8).rev() {
                if ((byte >> bit) & 1) == 1 {
                    self.data.set_high()?;
                } else {
                    self.data.set_low()?;
                }
                self.clock.set_high()?; // 74HC595 shifts on the rising SRCLK edge
                self.clock.set_low()?;
            }
        }
        self.latch.set_high()?; // one rising RCLK edge transfers to the outputs
        self.latch.set_low()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_hal_mock::eh1::digital::{Mock, State, Transaction};
    use std::vec::Vec;

    /// Expected data and clock transactions for one frame, MSB first.
    fn expected(frame: &[u8; FRAME_LEN]) -> (Vec<Transaction>, Vec<Transaction>) {
        let mut data = Vec::new();
        let mut clock = Vec::new();
        for &byte in frame {
            for bit in (0..8).rev() {
                let level = if ((byte >> bit) & 1) == 1 {
                    State::High
                } else {
                    State::Low
                };
                data.push(Transaction::set(level));
                clock.push(Transaction::set(State::High));
                clock.push(Transaction::set(State::Low));
            }
        }
        (data, clock)
    }

    fn assert_shifts(frame: [u8; FRAME_LEN]) {
        let (data_tx, clock_tx) = expected(&frame);
        let latch_tx = [
            Transaction::set(State::Low),
            Transaction::set(State::High),
            Transaction::set(State::Low),
        ];

        let mut writer = BitBang::new(
            Mock::new(&latch_tx),
            Mock::new(&clock_tx),
            Mock::new(&data_tx),
        );
        writer.write_frame(&frame).unwrap();

        // Verify every expected transaction was consumed.
        writer.latch.done();
        writer.clock.done();
        writer.data.done();
    }

    #[test]
    fn clear_frame_shifts_forty_low_bits() {
        assert_shifts([0x00; FRAME_LEN]);
    }

    #[test]
    fn msb_first_bit_and_byte_order() {
        // Only the LSB of byte 0 set: data goes low x7 then high, then low x32.
        assert_shifts([0x01, 0x00, 0x00, 0x00, 0x00]);
        assert_shifts([0x40, 0x6F, 0x00, 0x00, 0x3F]);
    }
}
