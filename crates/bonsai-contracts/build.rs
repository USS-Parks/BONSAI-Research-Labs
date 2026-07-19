use std::env;
use std::error::Error;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    let manifest_dir =
        PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").ok_or("missing manifest dir")?);
    let proto_root = manifest_dir.join("../../proto");
    let envelope = proto_root.join("bonsai/event/v1/envelope.proto");
    let governor_decision = proto_root.join("bonsai/governor/v1/decision.proto");
    let artifact_lineage = proto_root.join("bonsai/artifact/v1/lineage.proto");
    let adapter = proto_root.join("bonsai/adapter/v1/adapter.proto");
    let descriptor = PathBuf::from(env::var_os("OUT_DIR").ok_or("missing output dir")?)
        .join("bonsai-descriptor.bin");

    println!("cargo:rerun-if-changed={}", envelope.display());
    println!("cargo:rerun-if-changed={}", governor_decision.display());
    println!("cargo:rerun-if-changed={}", artifact_lineage.display());
    println!("cargo:rerun-if-changed={}", adapter.display());
    let mut config = prost_build::Config::new();
    config
        .protoc_executable(protoc_bin_vendored::protoc_bin_path()?)
        .file_descriptor_set_path(descriptor)
        .include_file("bonsai.rs")
        .compile_protos(
            &[envelope, governor_decision, artifact_lineage, adapter],
            &[proto_root],
        )?;
    Ok(())
}
