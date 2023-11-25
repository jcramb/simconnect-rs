use std::env;
use std::path::PathBuf;

fn main() {
    
    // create absolute path for static lib include directory
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let static_dir = dir.join("sdk").join("lib").join("static");

    // configure linking options
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rustc-link-lib=static:+bundle=SimConnect");
    println!("cargo:rustc-link-search={}", static_dir.display());

    // generate bindings using bindgen and clang
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .clang_args(&["-x", "c++"])
        .generate()
        .expect("Unable to generate SimConnect bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Couldn't write SimConnect bindings!");
}