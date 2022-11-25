#![cfg_attr(not(test), no_std)]

pub use heapless::Vec;
pub use postcard::from_bytes;
pub use postcard::to_vec;

#[cfg(feature = "sensor")]
mod sensor;

#[cfg(feature = "actuator")]
mod actuator;

pub mod error;

#[cfg(feature = "sensor")]
pub use sensor::Response as SensorResponse;

#[cfg(feature = "sensor")]
pub use sensor::Request as SensorRequest;

#[cfg(feature = "actuator")]
pub use actuator::Response as ActuatorResponse;

#[cfg(feature = "actuator")]
pub use actuator::Request as ActuatorRequest;
