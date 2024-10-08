use std::sync::atomic::AtomicBool;

use rust_extensions::date_time::DateTimeAsMicroseconds;
use tokio::sync::Mutex;

use crate::{
    my_telemetry_event::TelemetryEventTag, MyTelemetryContext, TelemetryCollector, TelemetryEvent,
};

pub struct TelemetryInterface {
    pub telemetry_collector: Mutex<TelemetryCollector>,
    pub writer_is_set: AtomicBool,
}

impl TelemetryInterface {
    pub fn new() -> Self {
        Self {
            telemetry_collector: Mutex::new(TelemetryCollector::new()),
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
        data: String,
        success: String,
        tags: Option<Vec<TelemetryEventTag>>,
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
                    data,
                    success: Some(success),
                    fail: None,
                    tags,
                };
                let mut write_access = self.telemetry_collector.lock().await;
                write_access.write(event)
            }
            MyTelemetryContext::Multiple(ids) => {
                let mut events = Vec::with_capacity(ids.len());
                for i in 0..ids.len() - 1 {
                    let event = TelemetryEvent {
                        process_id: *ids.get(i).unwrap(),
                        started: started.unix_microseconds,
                        finished: DateTimeAsMicroseconds::now().unix_microseconds,
                        data: data.to_string(),
                        success: Some(success.to_string()),
                        fail: None,
                        tags: tags.clone(),
                    };

                    events.push(event);
                }

                let event = TelemetryEvent {
                    process_id: *ids.get(ids.len() - 1).unwrap(),
                    started: started.unix_microseconds,
                    finished: DateTimeAsMicroseconds::now().unix_microseconds,
                    data: data,
                    success: Some(success),
                    fail: None,
                    tags,
                };

                events.push(event);

                let mut write_access = self.telemetry_collector.lock().await;
                write_access.write_events(events)
            }

            MyTelemetryContext::Empty => {}
        }
    }

    pub async fn write_fail(
        &self,
        ctx: &MyTelemetryContext,
        started: DateTimeAsMicroseconds,
        data: String,
        fail: String,
        tags: Option<Vec<TelemetryEventTag>>,
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
                    data,
                    success: None,
                    fail: Some(fail),
                    tags,
                };
                let mut write_access = self.telemetry_collector.lock().await;
                write_access.write(event)
            }
            MyTelemetryContext::Multiple(ids) => {
                let mut events = Vec::with_capacity(ids.len());
                for i in 0..ids.len() - 1 {
                    let event = TelemetryEvent {
                        process_id: *ids.get(i).unwrap(),
                        started: started.unix_microseconds,
                        finished: DateTimeAsMicroseconds::now().unix_microseconds,
                        data: data.clone(),
                        success: None,
                        fail: Some(fail.clone()),
                        tags: tags.clone(),
                    };

                    events.push(event);
                }

                let event = TelemetryEvent {
                    process_id: *ids.get(ids.len() - 1).unwrap(),
                    started: started.unix_microseconds,
                    finished: DateTimeAsMicroseconds::now().unix_microseconds,
                    data,
                    success: None,
                    fail: Some(fail),
                    tags,
                };

                events.push(event);

                let mut write_access = self.telemetry_collector.lock().await;
                write_access.write_events(events)
            }
            MyTelemetryContext::Empty => {}
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

pub struct MyTelemetryCompiler {
    items: Vec<i64>,
}

impl MyTelemetryCompiler {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn add(&mut self, item: &MyTelemetryContext) {
        match item {
            MyTelemetryContext::Single(value) => self.items.push(*value),
            MyTelemetryContext::Multiple(values) => self.items.extend_from_slice(values.as_slice()),
            MyTelemetryContext::Empty => {}
        }
    }

    pub fn compile(self) -> MyTelemetryContext {
        if self.items.len() == 0 {
            panic!("Can not compile telemetry context with no items");
        }

        if self.items.len() == 1 {
            return MyTelemetryContext::Single(self.items[0]);
        }

        return MyTelemetryContext::Multiple(self.items);
    }
}
