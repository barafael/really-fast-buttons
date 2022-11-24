#![cfg_attr(not(test), no_std)]

pub use heapless::Vec;
pub use postcard::from_bytes;
pub use postcard::to_vec;

#[cfg(feature = "actuator")]
mod actuator;

#[cfg(feature = "sensor")]
mod sensor;

pub mod error;

#[cfg(feature = "actuator")]
pub use actuator::Message as ActuatorMessage;

#[cfg(feature = "sensor")]
pub use sensor::Message as SensorMessage;
