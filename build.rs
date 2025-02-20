extern crate bindgen;
// extern crate openmp_sys; // might be useful for statically comiling with openmp

use std::{
    env,
    fs::{read_dir, remove_dir_all},
    path::PathBuf,
    process::Command,
};

/// To avoid any mistypes in paths, I chose to use this struct to set all paths once and use them for here on.
struct BuildPaths(PathBuf);

impl BuildPaths {
    const WFA2FOLDER: &str = "WFA2-lib";
    fn out_dir(&self) -> &PathBuf { &self.0 }
    // The path of the WFA2-lib right in the base folder of this project (source)
    fn wfa_src(&self) -> PathBuf { Self::WFA2FOLDER.into() }
    // Copy in OUT_DIR, this is where WFA is built!
    fn wfa_out(&self) -> PathBuf { self.out_dir().join(Self::WFA2FOLDER) }
    // Library path for WFA lib in build directory (wfa_out)
    // This is needed for the linker, otherwise wfalib will not be found.
    fn wfa_out_lib(&self) -> PathBuf { self.wfa_out().join("lib") }
}

impl Default for BuildPaths {
    fn default() -> Self {
        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        Self(out_dir)
    }
}

fn build_wfa() -> Option<()> {
    let build_paths = BuildPaths::default();

    let mut wfa_dir = read_dir(build_paths.wfa_src()).ok()?;
    if !wfa_dir.any(|f| f.unwrap().file_name() == "Makefile") {
        return None;
    }
    
    let _ = remove_dir_all(build_paths.wfa_out());

    // copy the WFA dir to OUT_PATH and build it there... clunky, but
    // don't want to pull in the entire 100MB WFA repo, since git2
    // doesn't seem to support shallow clones, and build scripts
    // should only modify things inside OUT_PATH. since the WFA folder
    // is just a couple MB, this is fine for now.
    let _cp_wfa = Command::new("cp")
        .arg("-r")
        .arg(build_paths.wfa_src())
        .arg(build_paths.out_dir())
        .output()
        .expect("Copy failed");

    // hotfix Makefile
    let _makefile_fix = Command::new("sed")
        .arg("-i")
        .arg("s/CC_FLAGS=-Wall -g/CC_FLAGS=-Wall -g -fPIC/g")
        .output()
        .expect("Failed hotfixing makefile");


    let output = Command::new("make")
        .arg("clean")
        .arg("all")
        .current_dir(build_paths.wfa_out())
        .output();

    match output {
        Ok(output) => { 
            if output.status.success() {
            Some(())
        } else {
            panic!("1) make error: {}", String::from_utf8_lossy(&output.stderr));
        }
    },
        Err(err) => { 
            panic!("2) make error: {}", err);
        },
    }
}


fn wfa() {
    let build_paths = BuildPaths::default();

    // 1. Link instructions for Cargo.

    // The directory of the WFA libraries, added to the search path.
    // println!("cargo:rustc-link-search=../wfa2/lib");
    // println!("cargo:rustc-link-search=WFA2-lib/lib");
    // Link the `wfa-lib` library.
    // Mind that the name of wfalib is set in the makefile, not here.
    // Relevant part in makefile:
    //  LIB_WFA=$(FOLDER_LIB)/libwfa.a
    //  LIB_WFA_CPP=$(FOLDER_LIB)/libwfacpp.a
    // Despite the folder being named WFA2-lib, the library has to be linked with wfa
    println!("cargo:rustc-link-lib=wfa");
    // Also link `omp`.
    println!("cargo:rustc-link-lib=gomp"); // omp does not work for some reason, gomp does.
    // Invalidate the built crate whenever the linked library changes.
    println!("cargo:rerun-if-changed={}/libwfa.a", build_paths.wfa_out_lib().display());
    // Rustc lib search for library in "OUT_DIR"
    println!("cargo:rustc-link-search={}", build_paths.wfa_out_lib().display());
    // 2. Generate bindings.

    let bindings = bindgen::Builder::default()
        // Generate bindings for this header file.
        // .header("../wfa2/wavefront/wavefront_align.h")
        .header("WFA2-lib/wavefront/wavefront_align.h")
        // Add this directory to the include path to find included header files.
        // .clang_arg("-I../wfa2")
        .clang_arg(format!("-I{}", build_paths.wfa_src().display()))
        // Generate bindings for all functions starting with `wavefront_`.
        .allowlist_function("wavefront_.*")
        // Generate bindings for all variables starting with `wavefront_`.
        .allowlist_var("wavefront_.*")
        // Invalidate the built crate whenever any of the included header files
        // changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings_wfa.rs file.
    bindings
        .write_to_file(build_paths.out_dir().join("bindings_wfa.rs"))
        .expect("Couldn't write bindings!");
}

fn main() {
    build_wfa();
    wfa();
}