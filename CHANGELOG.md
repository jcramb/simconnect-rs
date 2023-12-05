# Change Log

All notable changes to this project will be documented in this file.

## [Unreleased]

## [0.23.1] - 2023-12-05

### Added
* Added GitHub action to perform daily check for new SDK versions.
* Added GitHub action to run tests and build examples for `simconnect-sys` crate.
* Added Rust versions of the SimConnect SDK samples, using `simconnect-sys` bindings.
    * `sample-ai-objects-and-waypoints`
    * `sample-ai-traffic`
    * `sample-facility-data-definition` (WIP)
    * `sample-facility-data-definition2` (WIP)
    * `sample-input-event`
    * `sample-joystick-input`
    * `sample-no-callback`
    * `sample-open-and-close`
    * `sample-request-data`
    * `sample-send-event-a`
    * `sample-send-event-b`
    * `sample-send-event-c`
    * `sample-set-data`
    * `sample-system-event`
    * `sample-tagged-data`
    * `sample-throttle-control`
    * `sample-tracking-errors`
    * `sample-windows-event`

## [0.22.3] - 2023-11-25

### Added
* Added `static` and `vendored` features.
* Added `SIMCONNECT_DIR` and `SIMCONNECT_NO_VENDOR` environment variables.
* Added `examples/sys-basic` demonstrating how the FFI bindings can be used.

### Changed
* Improved README.md with maintenance status, usage instructions and documenting features and environment variables used.
* Updated crate version to track the SimConnect SDK version it targets.
* Updated bindgen to not prepend enum names.

## [0.1.0] - 2023-11-23

### Added
* Added `wrapper.h` and `build.rs` to automatically generate bindings using `bindgen`.
* Added `SimConnect.h` and `SimConnect.lib` for version `0.22.3`.

[unreleased]: https://github.com/jcramb/simconnect-rs/compare/v0.23.1...HEAD
[0.23.1]: https://github.com/jcramb/simconnect-rs/compare/v0.22.3...v0.23.1
[0.22.3]: https://github.com/jcramb/simconnect-rs/compare/v0.1.0...v0.22.3
[0.1.0]: https://github.com/jcramb/simconnect-rs/releases/tag/v0.1.0