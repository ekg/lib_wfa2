// Import the bindgen crate for generating Rust FFI bindings
extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    // Tell Cargo to rerun this build script if WFA2LIB_PATH environment variable changes
    println!("cargo:rerun-if-env-changed=WFA2LIB_PATH");

    // Get WFA2-lib base directory from environment variable or use default
    let base_dir = env::var("WFA2LIB_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("./WFA2-lib"));
    
    // Construct paths for library and header files
    let lib_path = base_dir.join("lib");
    let header_path = base_dir.join("wavefront/wavefront_align.h");

    // Tell Cargo where to find and how to link the required libraries
    // Specify the search directory for libraries
    println!("cargo:rustc-link-search={}", lib_path.display());
    // Link against the WFA library
    println!("cargo:rustc-link-lib=wfa");
    // Link against OpenMP for parallel execution support
    println!("cargo:rustc-link-lib=gomp");
    // Rerun build script if the WFA library file changes
    println!("cargo:rerun-if-changed={}", lib_path.join("libwfa.a").display());
    
    // Generate Rust FFI bindings from C header files
    bindgen::Builder::default()
        // Specify the main header file to generate bindings for
        .header(header_path.to_str().unwrap())
        // Add WFA2-lib root to include path for finding dependencies
        .clang_arg(format!("-I{}", base_dir.display()))
        // Only generate bindings for functions starting with "wavefront_"
        .allowlist_function("wavefront_.*")
        // Only generate bindings for variables starting with "wavefront_"
        .allowlist_var("wavefront_.*")
        // Use default Cargo callbacks for error handling and rebuilding
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Generate the bindings
        .generate()
        .expect("Unable to generate bindings")
        // Write bindings to the out directory specified by Cargo
        .write_to_file(PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings_wfa.rs"))
        .expect("Couldn't write bindings!");
}