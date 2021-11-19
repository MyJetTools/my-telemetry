use std::time::Duration;

pub trait MyTelemetry {
    fn track_url_duration(
        &self,
        method: hyper::Method,
        uri: hyper::Uri,
        http_code: u16,
        duration: Duration,
    );

    fn track_dependency_duration(
        &self,
        dependency_name: String,
        method: hyper::Method,
        uri: hyper::Uri,
        success: bool,
        duration: Duration,
    );
}
