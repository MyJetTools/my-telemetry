mod event_duration_tracker;
mod my_telemetry_event;

pub use event_duration_tracker::*;
mod ctx;
mod telemetry_collector;
pub use ctx::*;
pub use my_telemetry_event::*;
pub use telemetry_collector::TelemetryCollector;
mod telemetry_interface;
pub use telemetry_interface::*;

#[cfg(feature = "my-telemetry-writer")]
pub extern crate my_telemetry_writer as telemetry_server_writer;

lazy_static::lazy_static! {
    pub static ref TELEMETRY_INTERFACE: TelemetryInterface =
        TelemetryInterface::new();
}
