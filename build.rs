fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("src/grpc/proto/tidybee_events.proto")?;
    Ok(())
}
