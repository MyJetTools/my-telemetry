use crate::MyTelemetry;

pub struct MyTelemetryToConsole {}

impl MyTelemetry for MyTelemetryToConsole {
    fn track(
        &self,
        process_id: i64,
        started: i64,
        finished: i64,
        data: String,
        success: bool,
        status_code: i32,
    ) {
        println!(
            "MyTelemetryToConsole: process_id: {}, started: {}, finished: {}, data: {}, success: {}, status_code: {}",
            process_id, started, finished, data, success, status_code
        );
    }
}
