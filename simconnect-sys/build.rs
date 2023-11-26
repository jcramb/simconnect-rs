use std::env;
use std::path::PathBuf;

// env vars
const ENV_SIMCONNECT_DIR: &'static str = "SIMCONNECT_DIR";
const ENV_SIMCONNECT_NO_VENDOR: &'static str = "SIMCONNECT_NO_VENDOR";

// defaults
const DEFAULT_SIMCONNECT_DIR: &'static str = "C:\\MSFS SDK\\SimConnect SDK";

fn main() {

    // rebuild if env vars change
    println!("cargo:rerun-if-env-changed={ENV_SIMCONNECT_DIR}");
    println!("cargo:rerun-if-env-changed={ENV_SIMCONNECT_NO_VENDOR}");
        
    // convert feature flags to booleans
    let feature_vendored;
    let feature_static;
    #[cfg(feature = "vendored")] {
        feature_vendored = true && 
            env::var(ENV_SIMCONNECT_NO_VENDOR).map_or(true, |s| s == "0");
    }
    #[cfg(not(feature = "vendored"))] {
        feature_vendored = false;
    }
    #[cfg(feature = "static")] {
        feature_static = true;
    }
    #[cfg(not(feature = "static"))] {
        feature_static = false;
    }

    // determine which sdk directory to use
    let simconnect_dir = match feature_vendored {
        true => PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("sdk"),
        false => PathBuf::from(env::var("SIMCONNECT_DIR")
            .unwrap_or(DEFAULT_SIMCONNECT_DIR.to_string()))
    };

    // determine path for SimConnect header
    let simconnect_header = simconnect_dir.join("include").join("SimConnect.h")
        .display().to_string();

    // determine which sdk lib to use
    let simconnect_lib_dir: PathBuf;
    let simconnect_lib = if feature_static {
        simconnect_lib_dir = simconnect_dir.join("lib").join("static");
        "static=SimConnect"
    } else {
        simconnect_lib_dir = simconnect_dir.join("lib");
        "SimConnect"
    };

    // emit linking configuration
    println!("cargo:rustc-link-lib={}", simconnect_lib);
    println!("cargo:rustc-link-search={}", simconnect_lib_dir.display());

    // generate bindings using bindgen and clang
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .header(simconnect_header)
        .prepend_enum_name(false)
        // .layout_tests(false)
        .allowlist_var("MAX_.*")
        .allowlist_var("INITPOSITION_.*")
        .allowlist_item("(?i)SIMCONNECT.*")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .clang_args(&["-x", "c++"])
        .generate()
        .expect("Unable to generate SimConnect bindings");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Couldn't write SimConnect bindings!");
}