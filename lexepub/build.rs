fn main() {
    // Generate FFI bindings using Diplomat
    println!("cargo:rerun-if-changed=src/");

    // Create include directory if it doesn't exist
    std::fs::create_dir_all("include").ok();

    // Use diplomat-tool to generate C bindings
    if std::process::Command::new("diplomat-tool")
        .args(["c", "include/"])
        .output()
        .is_ok()
    {
        println!("Diplomat C bindings generated successfully!");
    } else {
        println!("diplomat-tool not found, skipping FFI generation");
    }
}
