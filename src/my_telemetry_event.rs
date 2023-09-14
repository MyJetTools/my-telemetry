use rust_extensions::StrOrString;

pub struct TelemetryEvent {
    pub process_id: i64,
    pub started: i64,
    pub finished: i64,
    pub data: String,
    pub success: Option<String>,
    pub fail: Option<String>,
    pub tags: Option<Vec<TelemetryEventTag>>,
}

#[derive(Clone)]
pub struct TelemetryEventTag {
    pub key: String,
    pub value: String,
}

pub struct TelemetryEventTagsBuilder {
    pub events: Option<Vec<TelemetryEventTag>>,
}

impl TelemetryEventTagsBuilder {
    pub fn new() -> Self {
        Self {
            events: Some(Vec::new()),
        }
    }

    pub fn add(
        mut self,
        key: impl Into<StrOrString<'static>>,
        value: impl Into<StrOrString<'static>>,
    ) -> Self {
        if self.events.is_none() {
            self.events = Some(Vec::new());
        }
        self.events.as_mut().unwrap().push(TelemetryEventTag {
            key: key.into().into(),
            value: value.into().into(),
        });

        self
    }

    pub fn add_ip(self, ip: impl Into<StrOrString<'static>>) -> Self {
        self.add("ip", ip)
    }

    pub fn build(self) -> Option<Vec<TelemetryEventTag>> {
        self.events
    }

    pub fn take_tags(&mut self) -> Self {
        Self {
            events: self.events.take(),
        }
    }

    pub fn clone(&mut self) -> Self {
        Self {
            events: self.events.clone(),
        }
    }
}

impl Into<Option<Vec<TelemetryEventTag>>> for TelemetryEventTagsBuilder {
    fn into(self) -> Option<Vec<TelemetryEventTag>> {
        self.build()
    }
}
