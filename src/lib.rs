mod my_telemetry_event;
use std::sync::atomic::AtomicBool;
mod telemetry_collector;
pub use my_telemetry_event::TelemetryEvent;
use rust_extensions::date_time::DateTimeAsMicroseconds;
pub use telemetry_collector::TelemtryCollector;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Copy)]
pub struct MyTelemetryContext {
    pub process_id: i64,
}

impl MyTelemetryContext {
    pub fn new() -> Self {
        Self {
            process_id: DateTimeAsMicroseconds::now().unix_microseconds,
        }
    }
    pub fn restore(process_id: i64) -> Self {
        Self { process_id }
    }

    pub fn start_event_tracking(&self, event_name: String) -> EventDurationTracker {
        EventDurationTracker {
            process_id: self.process_id,
            event_name: Some(event_name),
            started: DateTimeAsMicroseconds::now(),
        }
    }
}

pub struct EventDurationTracker {
    pub process_id: i64,
    pub event_name: Option<String>,
    pub started: DateTimeAsMicroseconds,
}

impl Drop for EventDurationTracker {
    fn drop(&mut self) {
        if crate::TELEMETRY_INTERFACE.is_telemetry_set_up() {
            if let Some(event_name) = self.event_name.take() {
                let event = TelemetryEvent {
                    process_id: self.process_id,
                    started: self.started.unix_microseconds,
                    finished: DateTimeAsMicroseconds::now().unix_microseconds,
                    data: event_name.to_string(),
                    success: Some("Duration tracking".to_string()),
                    fail: None,
                    ip: None,
                };
                tokio::spawn(
                    async move { crate::TELEMETRY_INTERFACE.write_telemetry_event(event) },
                );
            }
        }
    }
}

pub struct TelemetryInterface {
    pub telemetry_collector: Mutex<TelemtryCollector>,
    pub writer_is_set: AtomicBool,
}

impl TelemetryInterface {
    pub fn new() -> Self {
        Self {
            telemetry_collector: Mutex::new(TelemtryCollector::new()),
            writer_is_set: AtomicBool::new(false),
        }
    }

    pub fn is_telemetry_set_up(&self) -> bool {
        self.writer_is_set
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    pub async fn write_telemetry_event(&self, event: TelemetryEvent) {
        let mut write_access = self.telemetry_collector.lock().await;
        write_access.write(event)
    }
}

lazy_static::lazy_static! {
    pub static ref TELEMETRY_INTERFACE: TelemetryInterface = {
        TelemetryInterface{
            telemetry_collector: Mutex::new(TelemtryCollector::new()),
            writer_is_set: AtomicBool::new(false),
        }
    };
}
