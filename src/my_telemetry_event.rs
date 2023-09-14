pub struct TelemetryEvent {
    pub process_id: i64,
    pub started: i64,
    pub finished: i64,
    pub data: String,
    pub success: Option<String>,
    pub fail: Option<String>,
    pub ip: Option<String>,
    pub tags: Option<Vec<TelemetryEventTag>>,
}

pub struct TelemetryEventTag {
    pub key: String,
    pub value: String,
}
