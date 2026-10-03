use std::sync::atomic::AtomicBool;

use parking_lot::Mutex;
use rust_extensions::date_time::DateTimeAsMicroseconds;

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

    pub fn write_success(
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
                let mut write_access = self.telemetry_collector.lock();
                write_access.write(event)
            }
            MyTelemetryContext::Multiple(ids) => {
                // An empty list names no process: nothing to write, as for `Empty`.
                let Some((last_id, other_ids)) = ids.split_last() else {
                    return;
                };

                let mut events = Vec::with_capacity(ids.len());
                for process_id in other_ids {
                    let event = TelemetryEvent {
                        process_id: *process_id,
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
                    process_id: *last_id,
                    started: started.unix_microseconds,
                    finished: DateTimeAsMicroseconds::now().unix_microseconds,
                    data: data,
                    success: Some(success),
                    fail: None,
                    tags,
                };

                events.push(event);

                let mut write_access = self.telemetry_collector.lock();
                write_access.write_events(events)
            }

            MyTelemetryContext::Empty => {}
        }
    }

    pub fn write_fail(
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
                let mut write_access = self.telemetry_collector.lock();
                write_access.write(event)
            }
            MyTelemetryContext::Multiple(ids) => {
                // An empty list names no process: nothing to write, as for `Empty`.
                let Some((last_id, other_ids)) = ids.split_last() else {
                    return;
                };

                let mut events = Vec::with_capacity(ids.len());
                for process_id in other_ids {
                    let event = TelemetryEvent {
                        process_id: *process_id,
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
                    process_id: *last_id,
                    started: started.unix_microseconds,
                    finished: DateTimeAsMicroseconds::now().unix_microseconds,
                    data,
                    success: None,
                    fail: Some(fail),
                    tags,
                };

                events.push(event);

                let mut write_access = self.telemetry_collector.lock();
                write_access.write_events(events)
            }
            MyTelemetryContext::Empty => {}
        }
    }

    pub async fn write_telemetry_event(&self, event: TelemetryEvent) {
        let mut write_access = self.telemetry_collector.lock();
        write_access.write(event)
    }

    pub async fn write_telemetry_events(&self, events: Vec<TelemetryEvent>) {
        let mut write_access = self.telemetry_collector.lock();

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

#[cfg(test)]
mod tests {
    use std::sync::atomic::Ordering;

    use rust_extensions::date_time::DateTimeAsMicroseconds;

    use super::*;

    fn interface_with_writer() -> TelemetryInterface {
        let interface = TelemetryInterface::new();
        interface.writer_is_set.store(true, Ordering::Relaxed);
        interface
    }

    fn written_process_ids(interface: &TelemetryInterface) -> Vec<i64> {
        interface
            .telemetry_collector
            .lock()
            .get_events()
            .unwrap_or_default()
            .into_iter()
            .map(|event| event.process_id)
            .collect()
    }

    #[test]
    fn an_empty_multiple_context_writes_nothing() {
        let interface = interface_with_writer();
        let ctx = MyTelemetryContext::Multiple(vec![]);

        interface.write_success(
            &ctx,
            DateTimeAsMicroseconds::now(),
            "data".to_string(),
            "ok".to_string(),
            None,
        );
        interface.write_fail(
            &ctx,
            DateTimeAsMicroseconds::now(),
            "data".to_string(),
            "failed".to_string(),
            None,
        );

        assert!(written_process_ids(&interface).is_empty());
    }

    #[test]
    fn a_multiple_context_writes_one_event_per_process() {
        let interface = interface_with_writer();
        let ctx = MyTelemetryContext::Multiple(vec![1, 2, 3]);

        interface.write_success(
            &ctx,
            DateTimeAsMicroseconds::now(),
            "data".to_string(),
            "ok".to_string(),
            None,
        );
        assert_eq!(written_process_ids(&interface), vec![1, 2, 3]);

        interface.write_fail(
            &ctx,
            DateTimeAsMicroseconds::now(),
            "data".to_string(),
            "failed".to_string(),
            None,
        );
        assert_eq!(written_process_ids(&interface), vec![1, 2, 3]);
    }
}
