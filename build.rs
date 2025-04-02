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

    // Tell Cargo where to find and how to link the required libraries
    // Specify the search directory for libraries
    println!("cargo:rustc-link-search={}", lib_path.display());
    // Link against the WFA library
    println!("cargo:rustc-link-lib=wfa");
    // Link against OpenMP for parallel execution support
    println!("cargo:rustc-link-lib=gomp");
    // Rerun build script if the WFA library file changes
    println!(
        "cargo:rerun-if-changed={}",
        lib_path.join("libwfa.a").display()
    );
}
