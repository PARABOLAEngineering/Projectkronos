fn main() {
    println!("cargo:rustc-link-search=native=/usr/local/lib");  // Add library search path
    println!("cargo:rustc-link-search=native=/usr/lib");        // Add system library path
    println!("cargo:rustc-link-lib=static=swe");                // Try static linking first
    println!("cargo:rustc-link-lib=dylib=swe");                // Fall back to dynamic if needed
    println!("cargo:rerun-if-changed=wrapper.h");
}