//! STM32F7 wiring example (e.g. STM32F767 Nucleo / Discovery).
//!
//! Not built in CI — reference for real hardware. Cortex-M7F (hardware FPU) →
//! target `thumbv7em-none-eabihf`.
//!
//! NOTE: the latest published `stm32f7xx-hal` (0.8) still targets embedded-hal
//! **0.2**, so its pins are bridged to embedded-hal 1.0 with
//! `embedded-hal-compat`'s `.forward()`.
//!
//! ```text
//! # Add to Cargo.toml (pin current releases):
//! # [dependencies]
//! # stm32f7xx-hal       = { version = "0.8", features = ["stm32f767"] }
//! # embedded-hal-compat = "0.13"
//! # cortex-m-rt         = "0.7"
//! # panic-halt          = "1"
//! #
//! # rustup target add thumbv7em-none-eabihf
//! cargo build --example stm32f7 --features stm32f7 --target thumbv7em-none-eabihf
//! ```

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use embedded_hal_compat::{Forward, ForwardCompat, markers::ForwardOutputPin};
use lcd_4_5_digits::{Lcd45Digits, Symbol};
use panic_halt as _;
use stm32f7xx_hal::{pac, prelude::*};

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let gpiob = dp.GPIOB.split();

    // stm32f7xx-hal is still embedded-hal 0.2; `.forward()` adapts each pin to
    // embedded-hal 1.0. (latch, clock, data) — any three push-pull outputs.
    let latch: Forward<_, ForwardOutputPin> = gpiob.pb5.into_push_pull_output().forward();
    let clock: Forward<_, ForwardOutputPin> = gpiob.pb6.into_push_pull_output().forward();
    let data: Forward<_, ForwardOutputPin> = gpiob.pb7.into_push_pull_output().forward();

    let mut lcd = Lcd45Digits::new_bitbang(latch, clock, data);

    lcd.init().unwrap();
    lcd.set_integer(1234).unwrap();
    lcd.set_float(3.14, 2).unwrap();
    lcd.set_symbol(Symbol::LoBat, true).unwrap();

    loop {}
}
