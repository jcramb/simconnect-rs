use std::ffi::CString;

use simconnect_sys::*;

fn main() {

    // open simconnect
    let mut handle = std::ptr::null_mut();
    let name = CString::new("Open and Close").unwrap();
    let hr = unsafe { SimConnect_Open(
        &mut handle, name.as_ptr(), 
        std::ptr::null_mut(), 
        0, 
        std::ptr::null_mut(), 
        0
    ) };
    if hr != 0 {
        eprintln!("Error: SimConnect_Open failed!");
        return;
    }
    println!("Connected to Flight Simulator!");

    // close simconnect
    let hr = unsafe { SimConnect_Close(handle) };
    if hr != 0 {
        eprintln!("Error: SimConnect_Close failed!");
        return;
    }
    println!("Disconnected from Flight Simulator");
}