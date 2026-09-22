#![cfg_attr(not(any(test, feature = "std")), no_std)]

#[macro_use]
pub(crate) mod fmt;

use thiserror::Error;

device_driver::compile!(
    options: "--rust-defmt-feature=defmt",
    manifest: "device.ddsl"
);
pub const FT6336U_I2C_ADDRESS: u8 = 0x38;

#[derive(Debug, Error)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Ft6336uError<I2cErr> {
    #[error("I2C error")]
    I2c(I2cErr),
    #[error("Not supported: {0}")]
    NotSupported(&'static str),
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

/// The address/error types are shared between the blocking and async register
/// interfaces, so this impl lives outside the two `bisync` modules.
impl<I2CBus, E> device_driver::RegisterInterfaceBase for Ft6336uInterface<I2CBus>
where
    I2CBus: embedded_hal::i2c::ErrorType<Error = E>,
    E: core::fmt::Debug,
{
    type AddressType = u8;
    type Error = Ft6336uError<E>;
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
