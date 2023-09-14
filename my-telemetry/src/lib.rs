extern crate my_telemetry_core;

pub use my_telemetry_core::*;

#[cfg(feature = "my-telemetry-writer")]
extern crate my_telemetry_writer;
#[cfg(feature = "my-telemetry-writer")]
pub use my_telemetry_writer::*;
