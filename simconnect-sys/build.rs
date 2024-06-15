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
    #[cfg(not(feature = "c_msfs_sdk"))] {
        feature_vendored = true && 
            env::var(ENV_SIMCONNECT_NO_VENDOR).map_or(true, |s| s == "0");
    }
    #[cfg(feature = "c_msfs_sdk")] {
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

    // emit linking config for simconnect dependencies
    for lib in vec![
        "shlwapi",
        "user32",
        "Ws2_32"
    ] {
        println!("cargo:rustc-link-lib={}", lib);
    }

    // emit linking configuration
    println!("cargo:rustc-link-lib={}", simconnect_lib);
    println!("cargo:rustc-link-search={}", simconnect_lib_dir.display());

    // hack to ensure DLL is copied into deps directory to make `cargo run` work
    if !feature_static {
        let dll = "SimConnect.dll";
        let profile = env::var("PROFILE").unwrap();
        let target_dir = get_cargo_target_dir().unwrap().join(profile).join("deps");
        let _ = std::fs::copy(simconnect_lib_dir.join(dll), target_dir.join(dll));
    }

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
        .clang_args(&["-x", "c++", "-std=c++17"])
        .derive_default(true)
        .generate()
        .expect("Unable to generate SimConnect bindings");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Couldn't write SimConnect bindings!");
        // panic!("test")
}

fn get_cargo_target_dir() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let skip_triple = std::env::var("TARGET")? == std::env::var("HOST")?;
    let skip_parent_dirs = if skip_triple { 4 } else { 5 };
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR")?);
    let mut current = out_dir.as_path();
    for _ in 0..skip_parent_dirs {
        current = current.parent().ok_or("not found")?;
    }
    Ok(std::path::PathBuf::from(current))
}