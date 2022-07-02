pub trait MyTelemetry {
    fn track(
        &self,
        process_id: i64,
        started: i64,
        finished: i64,
        data: String,
        success: bool,
        status_code: i32,
    );
}
