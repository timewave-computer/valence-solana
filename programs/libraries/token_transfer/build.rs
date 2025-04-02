fn main() {
    // Tell cargo to ignore the specific errors in our dependencies
    println!("cargo:rustc-env=RUSTFLAGS=--allow=improper_ctypes");
    
    // Rebuild if this build script changes
    println!("cargo:rerun-if-changed=build.rs");
} 