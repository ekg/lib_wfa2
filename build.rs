// extern crate bindgen;

use std::{path::PathBuf, process::Command};

struct BuildPaths {
    wfa_src: PathBuf,
}

impl BuildPaths {
    fn new() -> Self {
        Self {
            wfa_src: PathBuf::from("WFA2-lib"),
        }
    }

    fn wfa_lib_dir(&self) -> PathBuf {
        self.wfa_src.join("lib")
    }
}

fn build_wfa() -> Result<(), Box<dyn std::error::Error>> {
    let paths = BuildPaths::new();

    // Check if WFA2-lib exists and has Makefile
    if !paths.wfa_src.join("Makefile").exists() {
        return Err("WFA2-lib/Makefile not found. Make sure the submodule is initialized.".into());
    }

    // Build WFA2-lib in place
    let output = Command::new("make")
        .args(["clean", "all"])
        .current_dir(&paths.wfa_src)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Make failed: {}", stderr).into());
    }

    Ok(())
}

fn setup_linking() {
    let paths = BuildPaths::new();

    // Link the WFA library
    println!("cargo:rustc-link-lib=static=wfa");
    println!("cargo:rustc-link-lib=gomp");

    // Set library search path
    println!(
        "cargo:rustc-link-search=native={}",
        paths.wfa_lib_dir().display()
    );

    // Rerun if WFA library changes
    println!("cargo:rerun-if-changed=WFA2-lib");
    println!(
        "cargo:rerun-if-changed={}/libwfa.a",
        paths.wfa_lib_dir().display()
    );

    // Generate bindings
    // let bindings = bindgen::Builder::default()
    //     // Generate bindings for this header file.
    //     // .header("../wfa2/wavefront/wavefront_align.h")
    //     .header("WFA2-lib/wavefront/wavefront_align.h")
    //     // Add this directory to the include path to find included header files.
    //     // .clang_arg("-I../wfa2")
    //     .clang_arg(format!("-I{}", build_paths.wfa_src().display()))
    //     // Generate bindings for all functions starting with `wavefront_`.
    //     .allowlist_function("wavefront_.*")
    //     // Generate bindings for all variables starting with `wavefront_`.
    //     .allowlist_var("wavefront_.*")
    //     // Invalidate the built crate whenever any of the included header files
    //     // changed.
    //     .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
    //     // Finish the builder and generate the bindings.
    //     .generate()
    //     // Unwrap the Result and panic on failure.
    //     .expect("Unable to generate bindings");
    // // Write the bindings to the $OUT_DIR/bindings_wfa.rs file.
    // bindings
    //     .write_to_file(build_paths.out_dir().join("bindings_wfa.rs"))
    //     .expect("Couldn't write bindings!");
}

fn main() {
    if let Err(e) = build_wfa() {
        panic!("Failed to build WFA2-lib: {}", e);
    }
    setup_linking();
}
