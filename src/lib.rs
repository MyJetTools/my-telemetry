mod event_duration_tracker;
mod my_telemetry_event;

pub use event_duration_tracker::*;
use std::sync::atomic::AtomicBool;
mod ctx;
mod telemetry_collector;
pub use ctx::*;
pub use my_telemetry_event::TelemetryEvent;
pub use telemetry_collector::TelemetryCollector;
use tokio::sync::Mutex;

lazy_static::lazy_static! {
    pub static ref TELEMETRY_INTERFACE: TelemetryInterface = {
        TelemetryInterface{
            telemetry_collector: Mutex::new(TelemetryCollector::new()),
            writer_is_set: AtomicBool::new(false),
        }
    };
}
