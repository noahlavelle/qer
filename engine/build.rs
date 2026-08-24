use std::{env, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let proto = [
        manifest_dir.join("../proto/qr/v1/engine.proto"),
        manifest_dir.join("proto/qr/v1/engine.proto"),
    ]
    .into_iter()
    .find(|path| path.exists())
    .ok_or("could not find proto/qr/v1/engine.proto")?;

    println!("cargo:rerun-if-changed={}", proto.display());
    tonic_prost_build::compile_protos(proto)?;
    Ok(())
}
