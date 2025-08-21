fn main() {
    tonic_prost_build::configure()
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile_protos(&["proto/TelemetryWriter.proto"], &["proto"])
        .unwrap();
}
