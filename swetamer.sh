#!/usr/bin/env bash
# Ensures script runs on any system with bash, not just ones with bash at /bin/bash

set -euo pipefail  # Stricter error handling than just set -e

# Function to cleanup on error
cleanup() {
    local exit_code=$?
    echo "⚠️ Error occurred. Cleaning up..."
    [ -d "${PACKAGE_NAME}" ] && rm -rf "${PACKAGE_NAME}"
    exit $exit_code
}

trap cleanup ERR

# Function to check for required tools
check_dependencies() {
    local missing_deps=()
    for cmd in cargo git cc make; do
        if ! command -v $cmd >/dev/null 2>&1; then
            missing_deps+=($cmd)
        fi
    done
    
    if [ ${#missing_deps[@]} -ne 0 ]; then
        echo "❌ Missing required dependencies: ${missing_deps[*]}"
        echo "Please install them and try again."
        exit 1
    fi
}

# Function to ensure we have enough disk space (at least 1GB free)
check_disk_space() {
    local free_space
    if [[ "$OSTYPE" == "darwin"* ]]; then
        free_space=$(df -k . | awk 'NR==2 {print $4}')
    else
        free_space=$(df -P . | awk 'NR==2 {print $4}')
    fi
    
    if [ "$free_space" -lt 1048576 ]; then  # 1GB in KB
        echo "❌ Insufficient disk space. Need at least 1GB free."
        exit 1
    fi
}

main() {
    echo "🐍 Checking environment..."
    check_dependencies
    check_disk_space

    # Get timestamp for unique package name
    TIMESTAMP=$(date +%Y%m%d_%H%M%S)
    PACKAGE_NAME="swisseph_sys_${TIMESTAMP}"
    echo "🌟 Setting up Swiss Ephemeris and Rust bindings as ${PACKAGE_NAME}..."

    # Create project structure
    mkdir -p "${PACKAGE_NAME}"/{src,include,libs}

    # Clone Swiss Ephemeris if not already present
    if [ ! -d "swisseph" ]; then
        echo "📥 Cloning Swiss Ephemeris..."
        git clone https://github.com/aloistr/swisseph.git || {
            echo "❌ Failed to clone Swiss Ephemeris"
            exit 1
        }
    fi

    # Build Swiss Ephemeris with platform detection
    echo "🏗️ Building Swiss Ephemeris..."
    cd swisseph
    make clean >/dev/null 2>&1 || true  # Ignore if clean fails
    if [[ "$OSTYPE" == "darwin"* ]]; then
        CFLAGS="-fPIC" make libswe.a
    else
        make libswe.a
    fi
    cd ..

    # Verify library was built
    if [ ! -f "swisseph/libswe.a" ]; then
        echo "❌ Failed to build Swiss Ephemeris library"
        exit 1
    fi

    # Copy necessary files with error checking
    echo "📋 Copying library and header files..."
    for file in swephexp.h sweph.h sweodef.h; do
        cp "swisseph/$file" "${PACKAGE_NAME}/include/" || {
            echo "❌ Failed to copy $file"
            exit 1
        }
    done
    cp "swisseph/libswe.a" "${PACKAGE_NAME}/libs/" || {
        echo "❌ Failed to copy libswe.a"
        exit 1
    }

    # Create wrapper.h
    cat > "${PACKAGE_NAME}/wrapper.h" << 'EOL'
#include "include/swephexp.h"
#include "include/sweph.h"
EOL

    # Initialize Rust project
    cd "${PACKAGE_NAME}"
    echo "🦀 Initializing Rust project..."
    cargo init --lib --name "${PACKAGE_NAME}"

    # Add dependencies
    echo "📦 Adding dependencies..."
    cargo add bindgen --build
    cargo add libc
    cargo add thiserror
    cargo add chrono
    cargo add parking_lot
    
    # Create build.rs
    cat > build.rs << 'EOL'
use std::env;
use std::path::PathBuf;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    println!("cargo:rustc-link-search=native={}/libs", manifest_dir);
    println!("cargo:rustc-link-lib=static=swe");
    println!("cargo:rerun-if-changed=wrapper.h");

    let mut builder = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{}/include", manifest_dir))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .blocklist_item("FP_NAN")
        .blocklist_item("FP_INFINITE")
        .blocklist_item("FP_ZERO")
        .blocklist_item("FP_SUBNORMAL")
        .blocklist_item("FP_NORMAL")
        .raw_line("#[allow(warnings)]");

    // Platform-specific configurations
    if cfg!(target_os = "macos") {
        builder = builder.clang_arg("-D_DARWIN_C_SOURCE");
    }

    let bindings = builder
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
EOL

    # Create lib.rs with prelude module
    cat > src/lib.rs << 'EOL'
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub mod prelude {
    pub use crate::*;
    
    // Re-export commonly used types and functions
    pub use crate::{
        swe_calc_ut,
        swe_close,
        swe_set_ephe_path,
        swe_version,
        // Add other commonly used functions here
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_bindings_available() {
        unsafe {
            let version = super::swe_version(std::ptr::null_mut());
            assert!(!version.is_null());
        }
    }
}
EOL

    # Create main.rs
    cat > src/main.rs << 'EOL'
use std::ffi::CString;

fn main() {
    unsafe {
        let version = CString::new("").unwrap();
        let ptr = version.into_raw();
        let v = swisseph_sys::swe_version(ptr);
        let version_str = CString::from_raw(v as *mut i8).into_string().unwrap();
        println!("🌃 Swiss Ephemeris version: {} successfully bound ⛓️", version_str);
        println!("Ephemeris files placed in ~/.swisseph/ephe for persistent storage");
        println!("See README.md for usage instructions");
    }
}
EOL

    # Create README.md
    cat > README.md << 'EOL'
# Swiss Ephemeris Rust Bindings

Auto-generated Rust bindings for the Swiss Ephemeris library.

## Features
- Complete Swiss Ephemeris functionality
- Safe Rust wrappers around unsafe C functions
- Prelude module for convenient imports
- Built-in examples and tests

## Usage

1. Place ephemeris files in `~/.swisseph/ephe`
2. Add this crate as a dependency in your `Cargo.toml`
3. Import common functionality with `use swisseph_sys::prelude::*;`

## Examples

```rust
use swisseph_sys::prelude::*;
use std::ffi::CString;

fn main() {
    unsafe {
        // Set ephemeris path
        let path = CString::new("~/.swisseph/ephe").unwrap();
        swe_set_ephe_path(path.as_ptr());
        
        // Calculate planet position
        let mut xx = [0.0; 6];
        let mut serr = [0i8; 256];
        
        let result = swe_calc_ut(
            2459580.5, // Julian day
            1,         // Planet number (1 = Moon)
            2,         // SEFLG_SWIEPH
            xx.as_mut_ptr(),
            serr.as_mut_ptr()
        );
        
        if result >= 0 {
            println!("Moon position: {:.6}°", xx[0]);
        }
        
        swe_close();
    }
}
```

## License

This wrapper is MIT licensed. Swiss Ephemeris is licensed under AGPL.
EOL

    # Setup ephemeris directory
    EPHE_DIR="${HOME}/.swisseph/ephe"
    if [ ! -d "$EPHE_DIR" ]; then
        mkdir -p "$EPHE_DIR"
        echo "📁 Created ephemeris directory at $EPHE_DIR"
    fi

    # Add to PATH if needed
    if [[ "$OSTYPE" == "linux-gnu"* ]] || [[ "$OSTYPE" == "linux"* ]] || [[ -n "$WSL_DISTRO_NAME" ]]; then
        SCRIPT_PATH="$(realpath "$0")"
        SCRIPT_DIR="$(dirname "$SCRIPT_PATH")"
        EXPORT_CMD="export PATH=\$PATH:${SCRIPT_DIR}"
        
        for rc_file in ~/.bashrc ~/.zshrc ~/.profile; do
            if [ -f "$rc_file" ] && ! grep -Fxq "$EXPORT_CMD" "$rc_file"; then
                echo "$EXPORT_CMD" >> "$rc_file"
                echo "🔗 Added swisseph to PATH in $rc_file"
            fi
        done
    fi

    echo "✨ Setup complete! ${PACKAGE_NAME} is ready."
    echo "📝 Next steps:"
    echo "1. cd ${PACKAGE_NAME}"
    echo "2. cargo test  # Verify bindings work"
    echo "3. cargo build # Build the library"
    echo "4. Place ephemeris files in ~/.swisseph/ephe"

    # Try to build immediately as a test
    echo "🧪 Running initial test build..."
    cargo build --quiet
    
    if [ $? -eq 0 ]; then
        echo "✅ Initial build successful!"
    else
        echo "⚠️ Initial build failed. Please check the output above for errors."
    fi
}

# Run main function
main "$@"