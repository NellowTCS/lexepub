fn main() {
    // Generate FFI bindings using Diplomat
    println!("cargo:rerun-if-changed=src/ffi.rs");
    println!("cargo:rerun-if-changed=diplomat.toml");

    // Always generate C bindings for distribution
    // Create include directory if it doesn't exist
    std::fs::create_dir_all("include").expect("Failed to create include directory");

    // Run diplomat-tool to generate C bindings
    match std::process::Command::new("diplomat-tool")
        .args(["c", "."])
        .output()
    {
        Ok(output) if output.status.success() => {
            println!("Diplomat C bindings generated successfully!");

            // Copy generated headers to include directory
            let headers = ["EpubExtractor.h", "EpubExtractor.d.h", "diplomat_runtime.h"];
            for header in &headers {
                if std::path::Path::new(header).exists() {
                    std::fs::copy(header, format!("include/{}", header))
                        .unwrap_or_else(|_| panic!("Failed to copy {}", header));
                    // Remove the header from the root directory
                    std::fs::remove_file(header)
                        .unwrap_or_else(|_| panic!("Failed to remove {}", header));
                }
            }
        }
        Ok(output) => {
            println!(
                "diplomat-tool failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            println!("Continuing build without generated C bindings");
        }
        Err(e) => {
            println!(
                "diplomat-tool not found: {}. Install with: cargo install diplomat-tool",
                e
            );
            println!("Continuing build without generated C bindings");
        }
    }
}
