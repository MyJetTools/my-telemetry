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

#[derive(Clone, Debug)]
pub struct TelemetryEventTag {
    pub key: String,
    pub value: String,
}

#[derive(Debug)]
pub struct TelemetryEventTagsBuilder {
    pub tags: Option<Vec<TelemetryEventTag>>,
}

impl TelemetryEventTagsBuilder {
    pub fn new() -> Self {
        Self {
            tags: Some(Vec::new()),
        }
    }

    pub fn add(
        mut self,
        key: impl Into<StrOrString<'static>>,
        value: impl Into<StrOrString<'static>>,
    ) -> Self {
        if self.tags.is_none() {
            self.tags = Some(Vec::new());
        }
        self.tags.as_mut().unwrap().push(TelemetryEventTag {
            key: key.into().into(),
            value: value.into().into(),
        });

        self
    }

    pub fn add_ip(self, ip: impl Into<StrOrString<'static>>) -> Self {
        self.add("ip", ip)
    }

    pub fn build(self) -> Option<Vec<TelemetryEventTag>> {
        self.tags
    }

    pub fn take_tags(&mut self) -> Self {
        Self {
            tags: self.tags.take(),
        }
    }

    pub fn clone(&mut self) -> Self {
        Self {
            tags: self.tags.clone(),
        }
    }
}

impl Into<Option<Vec<TelemetryEventTag>>> for TelemetryEventTagsBuilder {
    fn into(self) -> Option<Vec<TelemetryEventTag>> {
        self.build()
    }
}
