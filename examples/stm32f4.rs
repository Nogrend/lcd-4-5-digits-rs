//! STM32F4 wiring example (e.g. STM32F411 "Black Pill" / Nucleo-F4).
//!
//! Not built in CI — reference for real hardware. Cortex-M4F → target
//! `thumbv7em-none-eabihf`. `stm32f4xx-hal` implements embedded-hal 1.0, so its
//! pins work with this driver directly.
//!
//! ```text
//! # Add to Cargo.toml (pin a current release):
//! # [dependencies]
//! # stm32f4xx-hal = { version = "0.23", features = ["stm32f411"] }
//! # cortex-m-rt   = "0.7"
//! # panic-halt    = "1"
//! #
//! # rustup target add thumbv7em-none-eabihf
//! cargo build --example stm32f4 --features stm32f4 --target thumbv7em-none-eabihf
//! ```

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use lcd_4_5_digits::{Lcd45Digits, Symbol};
use panic_halt as _;
use stm32f4xx_hal::{pac, prelude::*};

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let mut rcc = dp.RCC.constrain();
    let gpioa = dp.GPIOA.split(&mut rcc);

    // (latch, clock, data) — any three push-pull outputs. Distinct pin types,
    // but they share `Error = Infallible`, which is all `new_bitbang` needs.
    let latch = gpioa.pa5.into_push_pull_output();
    let clock = gpioa.pa6.into_push_pull_output();
    let data = gpioa.pa7.into_push_pull_output();

    let mut lcd = Lcd45Digits::new_bitbang(latch, clock, data);

    lcd.init().unwrap();
    lcd.set_integer(1234).unwrap();
    lcd.set_float(3.14, 2).unwrap();
    lcd.set_symbol(Symbol::LoBat, true).unwrap();

    loop {}
}
