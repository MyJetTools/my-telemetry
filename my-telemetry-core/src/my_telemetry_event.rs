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

#[derive(Debug, Clone, Default)]
pub struct TelemetryEventTagsBuilder {
    pub tags: Vec<TelemetryEventTag>,
}

impl TelemetryEventTagsBuilder {
    pub fn new() -> Self {
        Self { tags: Vec::new() }
    }

    pub fn add_as_ref(
        &mut self,
        key: impl Into<StrOrString<'static>>,
        value: impl Into<StrOrString<'static>>,
    ) {
        self.tags.push(TelemetryEventTag {
            key: key.into().into(),
            value: value.into().into(),
        });
    }

    pub fn add(
        mut self,
        key: impl Into<StrOrString<'static>>,
        value: impl Into<StrOrString<'static>>,
    ) -> Self {
        self.add_as_ref(key, value);
        self
    }

    pub fn add_ip(self, ip: impl Into<StrOrString<'static>>) -> Self {
        self.add("ip", ip)
    }

    pub fn build(self) -> Option<Vec<TelemetryEventTag>> {
        if self.tags.is_empty() {
            return None;
        }
        Some(self.tags)
    }

    pub fn take_tags(&mut self) -> Self {
        Self {
            tags: std::mem::take(&mut self.tags),
        }
    }
}

impl Into<Option<Vec<TelemetryEventTag>>> for TelemetryEventTagsBuilder {
    fn into(self) -> Option<Vec<TelemetryEventTag>> {
        self.build()
    }
}
