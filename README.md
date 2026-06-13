# lcd-4-5-digits

A `no_std` Rust driver for a 4½-digit **static** LCD driven by a chain of
74HC595 shift registers. It is a behaviour-preserving port of the Arduino
library [`Nogrend/lcd-4-5-digits`](https://github.com/Nogrend/lcd-4-5-digits),
rebuilt on the [`embedded-hal`](https://crates.io/crates/embedded-hal) 1.0
traits so it runs on any MCU.

The MCU shifts **five bytes** out **three wires** (latch, clock, data), MSB
first — exactly like Arduino `shiftOut(MSBFIRST)`. Because an LCD segment must be
driven with AC, a 50–60 Hz backplane square wave is XOR-ed with each segment bit
**in hardware** (a 74HC86); the firmware only decides which segments are on and
shifts the bytes out. This crate never deals with the backplane or timing.

## Features

- `#![no_std]`, no heap, no `unsafe`, no panics on valid input.
- Generic over any `embedded-hal` 1.0 `OutputPin` via a software bit-bang
  backend; the transport is isolated behind a `FrameWriter` trait so an SPI
  backend can be added later without changing the public API.
- Integers, fixed-point floats, a `MM:SS` timer, and persistent indicator
  symbols (low-battery + colons).

## Install

```toml
[dependencies]
lcd-4-5-digits = "0.1"
embedded-hal = "1.0"
```

## Usage

```rust
use lcd_4_5_digits::{Lcd45Digits, Symbol};

// `latch`, `clock`, `data` are OutputPins from your HAL, configured as outputs.
let mut lcd = Lcd45Digits::new_bitbang(latch, clock, data);

lcd.init()?;                          // blank the panel
lcd.set_integer(-1234)?;              // -1234
lcd.set_float(3.14, 2)?;              // 3.14
lcd.set_timer(90)?;                   // 01:30
lcd.set_symbol(Symbol::LoBat, true)?; // low-battery glyph (stays on)
```

See [`examples/stm32f4.rs`](examples/stm32f4.rs),
[`examples/stm32f7.rs`](examples/stm32f7.rs), and
[`examples/esp32.rs`](examples/esp32.rs) for STM32F4, STM32F7, and ESP32-WROOM
wiring (each built with that chip's HAL; not part of CI). The STM32F7 HAL is
still on embedded-hal 0.2, so that example bridges with `embedded-hal-compat`.

## Display model

A frame is five bytes: `[symbol, thousands, hundreds, tens, units]`. The
ten-thousands place is a **half digit** that can only be blank or `1`, so the
representable integer range is **−19999 ..= 19999**. Out-of-range values show
four middle dashes.

Each digit byte is a standard seven-segment map
(`bit0=A … bit6=G, bit7=DP`):

| Digit | Byte | | Digit | Byte |
| ----- | ---- | --- | ----- | ---- |
| 0 | `0x3F` | | 5 | `0x6D` |
| 1 | `0x06` | | 6 | `0x7D` |
| 2 | `0x5B` | | 7 | `0x07` |
| 3 | `0x4F` | | 8 | `0x7F` |
| 4 | `0x66` | | 9 | `0x6F` |

The symbol byte (byte 0) holds the indicators. `set_symbol` controls the
low-battery glyph and the three colons; the minus sign, the half-digit `1` and
the decimal point are driven by the numeric methods and are not user-settable.

| Bit | Mask | Meaning | Settable via `set_symbol` |
| --- | ---- | ------- | ------------------------- |
| 0 | `0x01` | minus sign | no (numeric) |
| 1 | `0x02` | left colon | yes (`Symbol::ColonLeft`) |
| 2 | `0x04` | middle colon | yes (`Symbol::ColonMiddle`) |
| 3 | `0x08` | right colon | yes (`Symbol::ColonRight`) |
| 4 | `0x10` | half-digit "1" | no (numeric) |
| 6 | `0x40` | low battery | yes (`Symbol::LoBat`) |
| 7 | `0x80` | decimal point | no (numeric) |

Indicator symbols **persist** across `set_integer` / `set_float` / `set_timer`;
they are cleared only by `clear`, `all_on`, or an overflow.

> **Note:** On the OD-454 glass the decimal point renders to the *left* of its
> digit, so frames are verified by their byte values, not by an imagined
> left-to-right string.

## Wiring

| Driver pin | 74HC595 | Notes |
| ---------- | ------- | ----- |
| latch | ST_CP / RCLK (pin 12) | one rising edge transfers the shift register to the outputs |
| clock | SH_CP / SRCLK (pin 11) | data shifts on the rising edge |
| data | DS / SER (pin 14) | MSB first |

## Testing

```sh
cargo test                                   # pure logic + bit-bang transactions
cargo build --target thumbv7em-none-eabihf   # proves no_std
```

The formatting logic is covered by golden frame vectors derived from the
original C++; the bit-bang backend is verified with `embedded-hal-mock`.

## License

MIT OR Apache-2.0.

Hardware design, datasheets, and the original Arduino implementation live in the
[upstream repository](https://github.com/Nogrend/lcd-4-5-digits).
