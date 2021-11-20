use crate::MyTelemetry;

pub struct MyTelemetryToConsole {}

impl MyTelemetry for MyTelemetryToConsole {
    fn track_url_duration(
        &self,
        method: hyper::Method,
        uri: hyper::Uri,
        http_code: u16,
        duration: std::time::Duration,
    ) {
        println!(
            "Url duration: {} {} Status code:{} Duration:{:?}",
            method, uri, http_code, duration
        );
    }

    fn track_dependency_duration(
        &self,
        name: String,
        dependency_type: String,
        target: String,
        success: bool,
        duration: std::time::Duration,
    ) {
        println!(
            "Dependency {} duration: {} {} Success:{} Duration:{:?}",
            name, dependency_type, target, success, duration
        );
    }
}
