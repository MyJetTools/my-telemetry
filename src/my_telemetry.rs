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
        host: String,
        protocol: String,
        resource: String,
        success: bool,
        duration: Duration,
    );
}
