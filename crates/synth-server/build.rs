use std::path::Path;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=proto/synth.proto");

    // Check if protoc is available
    let protoc_available = Command::new("protoc")
        .arg("--version")
        .output()
        .is_ok();

    // Check if generated file already exists
    let generated_file = Path::new("src/grpc/synth.synth.rs");

    if !protoc_available && generated_file.exists() {
        // Use existing pre-generated file
        println!("cargo:warning=protoc not found, using pre-generated proto code");
        return Ok(());
    }

    if !protoc_available {
        println!("cargo:warning=protoc not found and no pre-generated code exists.");
        println!("cargo:warning=Please install protoc or run the build on a system with protoc installed.");
        println!("cargo:warning=Building without gRPC support.");
        // Create a stub file
        std::fs::write(
            "src/grpc/synth.synth.rs",
            "// Proto code not generated - protoc not available\n",
        )?;
        return Ok(());
    }

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir("src/grpc")
        .compile_protos(&["proto/synth.proto"], &["proto"])?;

    Ok(())
}
