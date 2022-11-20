mod event_duration_tracker;
mod my_telemetry_event;

pub use event_duration_tracker::*;
use std::sync::atomic::AtomicBool;
mod telemetry_collector;
pub use my_telemetry_event::TelemetryEvent;
use rust_extensions::date_time::DateTimeAsMicroseconds;
pub use telemetry_collector::TelemtryCollector;
use tokio::sync::Mutex;
#[derive(Debug, Clone)]
pub enum MyTelemetryContext {
    Single(i64),
    Multiple(Vec<i64>),
}

impl MyTelemetryContext {
    pub fn merge_process(&mut self, other: &MyTelemetryContext) {
        match self {
            MyTelemetryContext::Single(id) => {
                let mut new_ids = Vec::new();
                new_ids.push(*id);
                match other {
                    MyTelemetryContext::Single(other_id) => {
                        new_ids.push(*other_id);
                    }
                    MyTelemetryContext::Multiple(other_ids) => {
                        new_ids.extend_from_slice(other_ids);
                    }
                }
                *self = MyTelemetryContext::Multiple(new_ids);
            }
            MyTelemetryContext::Multiple(ids) => match other {
                MyTelemetryContext::Single(other_id) => {
                    ids.push(*other_id);
                }
                MyTelemetryContext::Multiple(other_ids) => {
                    ids.extend_from_slice(other_ids);
                }
            },
        }
    }
}

impl MyTelemetryContext {
    pub fn new() -> Self {
        Self::Single(DateTimeAsMicroseconds::now().unix_microseconds)
    }
    pub fn restore(process_id: i64) -> Self {
        Self::Single(process_id)
    }

    pub fn start_event_tracking(&self, event_name: String) -> EventDurationTracker {
        EventDurationTracker {
            process_id: self.clone(),
            event_name: Some(event_name),
            started: DateTimeAsMicroseconds::now(),
            ok_result: None,
            fail_result: None,
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

    pub async fn write_telemetry_events(&self, events: Vec<TelemetryEvent>) {
        let mut write_access = self.telemetry_collector.lock().await;

        for event in events {
            write_access.write(event);
        }
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
