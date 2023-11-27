# `simconnect-sys`
[![docs](https://img.shields.io/docsrs/simconnect-sys?style=for-the-badge&logo=rust)](https://docs.rs/crate/simconnect-sys/latest)
[![license](https://img.shields.io/crates/l/simconnect-sys?style=for-the-badge)](https://crates.io/crates/simconnect-sys)
[![version](https://img.shields.io/crates/v/simconnect-sys?style=for-the-badge)](https://crates.io/crates/simconnect-sys)
[![downloads](https://img.shields.io/crates/d/simconnect-sys?style=for-the-badge)](https://crates.io/crates/simconnect-sys)
<picture><img alt="maintenance" src="https://img.shields.io/maintenance/active%20development/2023?style=for-the-badge"></picture>

FFI bindings for SimConnect. 

## Release Support 

Currently supported version of `simconnect-sys` is using SimConnect SDK `0.22.3`.

Crate will be updated for each new SimConnect SDK release with the version indicating the SDK version it targets.

## Usage

```toml
[dependencies]
simconnect-sys = { version = "0.22.3", features = [ "static", "vendored" ] }
```

### Getting Started

```rust
use simconnect_sys::*;

// open handle to SimConnect
let mut handle = std::ptr::null_mut();
let hr = unsafe { SimConnect_Open(
	&mut handle,
    CString::new("Example").as_ptr(),
    std::ptr::null_mut(),
    0,
    std::ptr::null_mut(),
    0,
) };
if hr != 0 || handle.is_null() {
	println!("SimConnect_Open failed");
}
```

See ![examples/sys-basic](https://github.com/jcramb/simconnect-rs/tree/main/examples/sys-basic) for a working example of using the FFI bindings for SimConnect.

### Features

* `static` - Statically link to SimConnect lib.
* `vendored` - Use vendored SimConnect lib.

### Environment Variables

* `SIMCONNECT_DIR` (_default=_`C:\MSFS SDK\SimConnect SDK`)
	* Directory containing the following files from the MSFS SimConnect SDK:
		```
        ├── include\
        │   └── SimConnect.h
        └── lib\
            ├── SimConnect.lib
     	    └── static\
                └── SimConnect.lib          
        ```
* `SIMCONNECT_NO_VENDOR` 
	* Provides an override of the `vendored` feature, ensuring vendored libs are not used. 

## License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed under the terms of both the Apache License,
Version 2.0 and the MIT license without any additional terms or conditions.