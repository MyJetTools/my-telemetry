mod grpc_writer;
mod http_writer;
mod my_telemetry_writer;
mod settings;
mod write_mode;
pub use my_telemetry_writer::MyTelemetryWriter;
pub use settings::MyTelemetrySettings;

mod writer_grpc {
    tonic::include_proto!("writer");
}
