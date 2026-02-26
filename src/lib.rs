#![cfg_attr(not(any(test, feature = "std")), no_std)]

#[macro_use]
pub(crate) mod fmt;

use thiserror::Error;

device_driver::create_device!(device_name: Ft6336uLowLevel, manifest: "device.yaml");
pub const FT6336U_I2C_ADDRESS: u8 = 0x38;

#[derive(Debug, Error)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Ft6336uError<I2cErr> {
    #[error("I2C error")]
    I2c(I2cErr),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TouchStatus {
    Touch,
    Stream,
    Release,
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct TouchPoint {
    pub status: TouchStatus,
    pub x: u16,
    pub y: u16,
}

impl Default for TouchPoint {
    fn default() -> Self {
        Self {
            status: TouchStatus::Release,
            x: 0,
            y: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct TouchData {
    pub touch_count: u8,
    pub points: [TouchPoint; 2],
}

pub struct Ft6336uInterface<I2CBus> {
    i2c_bus: I2CBus,
}

impl<I2CBus> Ft6336uInterface<I2CBus> {
    pub fn new(i2c_bus: I2CBus) -> Self {
        Self { i2c_bus }
    }
}

#[path = "."]
mod asynchronous {
    use bisync::asynchronous::*;
    use device_driver::AsyncRegisterInterface as RegisterInterface;
    use embedded_hal_async::i2c::I2c;
    mod driver;
    pub use driver::*;
}
pub use asynchronous::Ft6336u as Ft6336uAsync;

#[path = "."]
mod blocking {
    use bisync::synchronous::*;
    use device_driver::RegisterInterface;
    use embedded_hal::i2c::I2c;
    #[allow(clippy::duplicate_mod)]
    mod driver;
    pub use driver::*;
}
pub use blocking::Ft6336u;
