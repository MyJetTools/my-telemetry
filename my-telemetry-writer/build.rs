fn main() {
    let mut config = prost_build::Config::new();
    config.protoc_arg("--experimental_allow_proto3_optional");

    let url = "https://raw.githubusercontent.com/MyJetTools/my-telemetry-server/main/proto/";
    ci_utils::sync_and_build_proto_file(url, "TelemetryWriter.proto");
}
