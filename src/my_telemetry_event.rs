pub struct TelemetryEvent {
    pub process_id: i64,
    pub started: i64,
    pub finished: i64,
    pub data: String,
    pub success: bool,
    pub status_code: i32,
}
