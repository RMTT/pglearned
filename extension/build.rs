fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_files = &["../proto/pgl_rpc.proto"];

    tonic_prost_build::configure()
        .build_server(false)
        .build_client(true)
        .compile_protos(proto_files, &["../proto"])?;

    Ok(())
}
