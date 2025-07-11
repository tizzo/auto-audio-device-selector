fn main() {
    // Link against macOS frameworks
    println!("cargo:rustc-link-lib=framework=CoreAudio");
    println!("cargo:rustc-link-lib=framework=CoreFoundation");
    println!("cargo:rustc-link-lib=framework=AudioUnit");
    
    // Only build on macOS
    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-lib=framework=IOKit");
    }
}