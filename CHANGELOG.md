# Change Log

All notable changes to this project will be documented in this file.

## [Unreleased]

## [0.22.3] - 2023-11-25

### Added
* Added `static` and `vendored` features.
* Added `SIMCONNECT_DIR` and `SIMCONNECT_NO_VENDOR` environment variables.
* Added `examples/sys-basic` demonstrating how the FFI bindings can be used.

### Changed
* Improved README.md with current maintenance status, usage instructions 
and documenting the features and environment variables used.
* Updated crate version to track the SimConnect SDK version it targets.
* Updated bindgen to not prepend enum names.

## [0.1.0] - 2023-11-23

### Added
* Added `wrapper.h` and `build.rs` to automatically generate bindings using `bindgen`.
* Added `SimConnect.h` and `SimConnect.lib` for version `0.22.3`.

[unreleased]: https://github.com/jcramb/simconnect-rs/compare/v0.22.3...HEAD
[0.22.3]: https://github.com/jcramb/simconnect-rs/compare/v0.1.0...v0.22.3
[0.1.0]: https://github.com/jcramb/simconnect-rs/releases/tag/v0.1.0