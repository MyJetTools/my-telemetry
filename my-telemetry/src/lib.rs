extern crate my_telemetry_core;

pub use my_telemetry_core::*;

#[cfg(feature = "my-telemetry-writer")]
pub extern crate my_telemetry_writer;
