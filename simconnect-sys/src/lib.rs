#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
mod ffi {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}
pub use ffi::*;

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    pub fn it_works() -> Result<()> {
        assert_eq!(SIMCONNECT_UNUSED, u32::MAX);
        Ok(())
    }
}