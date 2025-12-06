fn main() {
    tonic_build::compile_protos("proto/event.proto")
        .unwrap();
}
