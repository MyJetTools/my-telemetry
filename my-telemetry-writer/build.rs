fn main() {
    //let url =
    // "https://raw.githubusercontent.com/MyJetTools/my-telemetry-server/refs/heads/main/proto/";
    // ci_utils::sync_and_build_proto_file(url, "TelemetryWriter.proto");

    let includes: Vec<String> = Vec::new();
    tonic_build::configure()
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile_protos(&["proto/TelemetryWriter.proto"], &includes)
        .unwrap();

    //    tonic_build::compile_protos("proto/TelemetryWriter.proto").unwrap();
}
