use std::env;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=RUSTC");
    let compiler = env::var_os("RUSTC").expect("Cargo provides the actual compiler path");
    let output = Command::new(compiler)
        .arg("--version")
        .output()
        .expect("read compiler identity");
    assert!(output.status.success(), "compiler identity command failed");
    let version = String::from_utf8(output.stdout).expect("compiler version is UTF-8");
    println!("cargo:rustc-env=BONSAI_BUILD_COMPILER={}", version.trim());
}
