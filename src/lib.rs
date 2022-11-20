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
    pub fn new() -> Self {
        Self::Single(DateTimeAsMicroseconds::now().unix_microseconds)
    }
    pub fn restore(process_id: i64) -> Self {
        Self::Single(process_id)
    }

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

impl<'s> IntoIterator for &'s MyTelemetryContext {
    type Item = i64;

    type IntoIter = TelemetryContextIterator<'s>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter { ctx: self, pos: 0 }
    }
}

pub struct TelemetryContextIterator<'s> {
    ctx: &'s MyTelemetryContext,
    pos: usize,
}

impl<'s> Iterator for TelemetryContextIterator<'s> {
    type Item = i64;

    fn next(&mut self) -> Option<Self::Item> {
        match self.ctx {
            MyTelemetryContext::Single(id) => {
                if self.pos > 0 {
                    return None;
                }
                let result = *id;
                self.pos += 1;
                return Some(result);
            }
            MyTelemetryContext::Multiple(ids) => {
                let result = ids.get(self.pos)?;
                return Some(*result);
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

    pub async fn write_success(
        &self,
        ctx: &MyTelemetryContext,
        started: DateTimeAsMicroseconds,
        data: &str,
        success: &str,
        ip: Option<&str>,
    ) {
        if !self.is_telemetry_set_up() {
            return;
        }

        match ctx {
            MyTelemetryContext::Single(process_id) => {
                let event = TelemetryEvent {
                    process_id: *process_id,
                    started: started.unix_microseconds,
                    finished: DateTimeAsMicroseconds::now().unix_microseconds,
                    data: data.to_string(),
                    success: Some(success.to_string()),
                    fail: None,
                    ip: if let Some(ip) = ip {
                        Some(ip.to_string())
                    } else {
                        None
                    },
                };
                let mut write_access = self.telemetry_collector.lock().await;
                write_access.write(event)
            }
            MyTelemetryContext::Multiple(ids) => {
                let mut events = Vec::with_capacity(ids.len());
                for process_id in ids {
                    let event = TelemetryEvent {
                        process_id: *process_id,
                        started: started.unix_microseconds,
                        finished: DateTimeAsMicroseconds::now().unix_microseconds,
                        data: data.to_string(),
                        success: Some(success.to_string()),
                        fail: None,
                        ip: if let Some(ip) = ip {
                            Some(ip.to_string())
                        } else {
                            None
                        },
                    };

                    events.push(event);
                }

                let mut write_access = self.telemetry_collector.lock().await;
                write_access.write_events(events)
            }
        }
    }

    pub async fn write_fail(
        &self,
        ctx: &MyTelemetryContext,
        started: DateTimeAsMicroseconds,
        data: &str,
        fail: &str,
        ip: Option<&str>,
    ) {
        if !self.is_telemetry_set_up() {
            return;
        }
        let mut write_access = self.telemetry_collector.lock().await;
        match ctx {
            MyTelemetryContext::Single(process_id) => {
                let event = TelemetryEvent {
                    process_id: *process_id,
                    started: started.unix_microseconds,
                    finished: DateTimeAsMicroseconds::now().unix_microseconds,
                    data: data.to_string(),
                    success: None,
                    fail: Some(fail.to_string()),
                    ip: if let Some(ip) = ip {
                        Some(ip.to_string())
                    } else {
                        None
                    },
                };

                write_access.write(event)
            }
            MyTelemetryContext::Multiple(ids) => {
                for process_id in ids {
                    let event = TelemetryEvent {
                        process_id: *process_id,
                        started: started.unix_microseconds,
                        finished: DateTimeAsMicroseconds::now().unix_microseconds,
                        data: data.to_string(),
                        success: None,
                        fail: Some(fail.to_string()),
                        ip: if let Some(ip) = ip {
                            Some(ip.to_string())
                        } else {
                            None
                        },
                    };

                    write_access.write(event)
                }
            }
        }
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
