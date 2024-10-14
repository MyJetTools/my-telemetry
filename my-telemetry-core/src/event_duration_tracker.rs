use rust_extensions::{date_time::DateTimeAsMicroseconds, StrOrString};

use crate::{my_telemetry_event::TelemetryEventTag, MyTelemetryContext, TelemetryEvent};

pub struct EventDurationTracker {
    pub my_telemetry: MyTelemetryContext,
    pub event_name: Option<StrOrString<'static>>,
    pub started: DateTimeAsMicroseconds,
    pub ok_result: Option<String>,
    pub fail_result: Option<String>,
    pub tags: Option<Vec<TelemetryEventTag>>,
    pub ignore_this_event: bool,
}

impl EventDurationTracker {
    pub fn new(event_name: impl Into<StrOrString<'static>>, ok_result: Option<String>) -> Self {
        let event_name = event_name.into();
        let now = DateTimeAsMicroseconds::now();
        Self {
            my_telemetry: MyTelemetryContext::Single(now.unix_microseconds),
            event_name: Some(event_name),
            started: now,
            ok_result,
            fail_result: None,
            tags: None,
            ignore_this_event: false,
        }
    }
    pub fn set_fail_result(&mut self, result: String) {
        self.fail_result = Some(result);
        self.ok_result = None;
    }

    pub fn set_ok_result(&mut self, result: String) {
        self.ok_result = Some(result);
    }
    pub fn ignore_this_event(&mut self) {
        self.ignore_this_event = true;
    }

    pub fn do_not_ignore_this_event(&mut self) {
        self.ignore_this_event = false;
    }

    pub fn add_tag(
        mut self,
        key: impl Into<StrOrString<'static>>,
        value: impl Into<StrOrString<'static>>,
    ) -> Self {
        if self.tags.is_none() {
            self.tags = Some(Vec::new());
        }

        self.tags.as_mut().unwrap().push(TelemetryEventTag {
            key: key.into().to_string(),
            value: value.into().to_string(),
        });

        self
    }
}

impl Drop for EventDurationTracker {
    fn drop(&mut self) {
        if self.ignore_this_event {
            return;
        }

        if !crate::TELEMETRY_INTERFACE.is_telemetry_set_up() {
            return;
        }

        let mut success = self.ok_result.take();
        let fail = self.fail_result.take();

        if fail.is_some() {
            success = None;
        } else if success.is_none() {
            success = Some("Duration tracking".to_string());
        }

        if let Some(event_name) = self.event_name.take() {
            match &self.my_telemetry {
                MyTelemetryContext::Single(process_id) => {
                    let event = TelemetryEvent {
                        process_id: *process_id,
                        started: self.started.unix_microseconds,
                        finished: DateTimeAsMicroseconds::now().unix_microseconds,
                        data: event_name.as_str().to_string(),
                        success,
                        fail,
                        tags: None,
                    };
                    tokio::spawn(async move {
                        crate::TELEMETRY_INTERFACE
                            .write_telemetry_event(event)
                            .await
                    });
                }
                MyTelemetryContext::Multiple(ids) => {
                    let mut events = Vec::with_capacity(ids.len());

                    for process_id in ids {
                        let event = TelemetryEvent {
                            process_id: *process_id,
                            started: self.started.unix_microseconds,
                            finished: DateTimeAsMicroseconds::now().unix_microseconds,
                            data: event_name.as_str().to_string(),
                            success: success.clone(),
                            fail: fail.clone(),
                            tags: None,
                        };

                        events.push(event);
                    }
                    tokio::spawn(async move {
                        crate::TELEMETRY_INTERFACE
                            .write_telemetry_events(events)
                            .await
                    });
                }

                MyTelemetryContext::Empty => {}
            }
        }
    }
}
