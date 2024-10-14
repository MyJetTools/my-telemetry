use rust_extensions::{date_time::DateTimeAsMicroseconds, StrOrString};

use crate::EventDurationTracker;

#[derive(Debug, Clone)]
pub enum MyTelemetryContext {
    Empty,
    Single(i64),
    Multiple(Vec<i64>),
}

impl MyTelemetryContext {
    pub fn create_empty() -> Self {
        Self::Empty
    }

    #[deprecated(note = "Use MyTelemetryContext::start_duration_tracking('my-process-name')")]
    pub fn new() -> Self {
        Self::Single(DateTimeAsMicroseconds::now().unix_microseconds)
    }

    #[deprecated(note = "Use MyTelemetryContext::start_duration_tracking('my-process-name')")]
    pub fn track_timer_duration(
        process_name: impl Into<StrOrString<'static>>,
    ) -> EventDurationTracker {
        EventDurationTracker::new(process_name, None)
    }

    pub fn start_duration_tracking(
        process_name: impl Into<StrOrString<'static>>,
    ) -> EventDurationTracker {
        EventDurationTracker::new(process_name, None)
    }

    pub fn start_event_tracking(
        &self,
        event_name: impl Into<StrOrString<'static>>,
    ) -> EventDurationTracker {
        EventDurationTracker {
            my_telemetry: self.clone(),
            event_name: Some(event_name.into()),
            started: DateTimeAsMicroseconds::now(),
            ok_result: None,
            fail_result: None,
            tags: None,
            ignore_this_event: false,
        }
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
                    MyTelemetryContext::Empty => {}
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
                MyTelemetryContext::Empty => {}
            },
            MyTelemetryContext::Empty => {
                *self = other.clone();
            }
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
            MyTelemetryContext::Empty => {
                DateTimeAsMicroseconds::now().unix_microseconds.to_string()
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
            MyTelemetryContext::Empty => None,
        }
    }
}

impl Into<MyTelemetryContext> for Option<&MyTelemetryContext> {
    fn into(self) -> MyTelemetryContext {
        if let Some(ctx) = self {
            ctx.clone()
        } else {
            MyTelemetryContext::Single(DateTimeAsMicroseconds::now().unix_microseconds).into()
        }
    }
}
