//! ESP32-WROOM DevKit (classic Xtensa ESP32, 4 MB flash) using the no_std `esp-hal`.
//!
//! Not built in CI — reference for real hardware. **esp-hal's API changes
//! between releases**, so pin a version and follow that release's examples
//! (this targets the 1.0 beta line). Build for the Xtensa target with the espup
//! toolchain and flash with `espflash` (it auto-detects the 4 MB flash).
//!
//! ```text
//! # Add to Cargo.toml (pin a current release):
//! # [dependencies]
//! # esp-hal       = { version = "1.0.0-beta.1", features = ["esp32"] }
//! # esp-backtrace = { version = "0.15", features = ["esp32", "panic-handler", "println"] }
//! #
//! # espup installs the xtensa-esp32-none-elf target
//! cargo build --example esp32 --features esp32 --target xtensa-esp32-none-elf --release
//! ```

#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::gpio::{Level, Output, OutputConfig};
use lcd_4_5_digits::{Lcd45Digits, Symbol};

#[esp_hal::main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    // (latch, clock, data) — any three GPIOs as push-pull outputs.
    let latch = Output::new(peripherals.GPIO4, Level::Low, OutputConfig::default());
    let clock = Output::new(peripherals.GPIO5, Level::Low, OutputConfig::default());
    let data = Output::new(peripherals.GPIO6, Level::Low, OutputConfig::default());

    let mut lcd = Lcd45Digits::new_bitbang(latch, clock, data);

    lcd.init().unwrap();
    lcd.set_integer(1234).unwrap();
    lcd.set_symbol(Symbol::LoBat, true).unwrap();

    loop {}
}
