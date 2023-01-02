use std::sync::atomic::AtomicBool;

use rust_extensions::date_time::DateTimeAsMicroseconds;
use tokio::sync::Mutex;

use crate::{EventDurationTracker, TelemetryEvent, TelemtryCollector};

#[derive(Debug, Clone)]
pub enum MyTelemetryContext {
    Single(i64),
    Multiple(Vec<i64>),
}

impl MyTelemetryContext {
    pub fn new() -> Self {
        Self::Single(DateTimeAsMicroseconds::now().unix_microseconds)
    }

    pub fn compile<'s, TIter: Iterator<Item = &'s MyTelemetryContext>>(items: TIter) -> Self {
        let mut result: Option<MyTelemetryContext> = None;

        for item in items {
            if let Some(ctx) = &mut result {
                ctx.merge_process(item);
            } else {
                result = Some(item.clone());
            }
        }

        result.unwrap()
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

    pub fn parse_from_string(str: &str) -> Result<Self, String> {
        let index = str.find(',');
        if index.is_none() {
            match str.parse::<i64>() {
                Ok(result) => {
                    return Ok(Self::Single(result));
                }
                Err(err) => return Err(format!("{}", err)),
            }
        }

        let mut ids = Vec::new();
        for id in str.split(',') {
            match id.parse::<i64>() {
                Ok(result) => {
                    ids.push(result);
                }
                Err(err) => return Err(format!("{}", err)),
            }
        }
        Ok(Self::Multiple(ids))
    }

    pub fn as_string(&self) -> String {
        match self {
            MyTelemetryContext::Single(value) => value.to_string(),
            MyTelemetryContext::Multiple(values) => {
                let mut result = String::new();
                for (no, value) in values.into_iter().enumerate() {
                    if no > 0 {
                        result.push(',');
                    }
                    result.push_str(&value.to_string());
                }

                result
            }
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
        data: String,
        success: String,
        ip: Option<String>,
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
                    ip,
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
                        ip: ip.clone(),
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
                    ip: ip,
                };

                events.push(event);

                let mut write_access = self.telemetry_collector.lock().await;
                write_access.write_events(events)
            }
        }
    }

    pub async fn write_fail(
        &self,
        ctx: &MyTelemetryContext,
        started: DateTimeAsMicroseconds,
        data: String,
        fail: String,
        ip: Option<String>,
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
                    ip,
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
                        ip: ip.clone(),
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
                    ip: ip,
                };

                events.push(event);

                let mut write_access = self.telemetry_collector.lock().await;
                write_access.write_events(events)
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
        }
    }

    pub fn compile(self) -> MyTelemetryContext {
        if self.items.len() == 0 {
            panic!("Can not comile telemetry context with no items");
        }

        if self.items.len() == 1 {
            return MyTelemetryContext::Single(self.items[0]);
        }

        return MyTelemetryContext::Multiple(self.items);
    }
}

impl Into<MyTelemetryContext> for Option<&MyTelemetryContext> {
    fn into(self) -> MyTelemetryContext {
        if let Some(ctx) = self {
            ctx.clone()
        } else {
            MyTelemetryContext::new()
        }
    }
}
