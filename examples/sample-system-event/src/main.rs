use std::ffi::{CString, c_void};
use std::sync::atomic::{AtomicBool, Ordering};

use simconnect_sys::*;

const EVENT_FLIGHT_LOAD: u32 = 1;

static mut HANDLE: *mut c_void = std::ptr::null_mut(); 
static QUIT: AtomicBool = AtomicBool::new(false);

unsafe extern "C" fn my_dispatch_proc(
    data: *mut SIMCONNECT_RECV,
    cb_data: DWORD,
    _context: *mut c_void,
) {
    match (*data).dwID as i32 {
        SIMCONNECT_RECV_ID_EVENT_FILENAME => {
            let event_filename = data as *const SIMCONNECT_RECV_EVENT_FILENAME;
            match (*event_filename)._base.uEventID {
                EVENT_FLIGHT_LOAD => {
                    println!("New Flight Loaded!");
                },
                _ => {}
            }

        },
        SIMCONNECT_RECV_ID_OPEN => {
            println!("SIMCONNECT_RECV_OPEN: data={:p} cb_data={:x}", data, cb_data);
        },
        SIMCONNECT_RECV_ID_QUIT => {
            QUIT.store(true, Ordering::Relaxed);
        },
        _ => {
            println!("DATA RECEIVED: data={:p} cb_data={:x}", data, cb_data);
        }
    }
}

fn main() -> Result<(), &'static str> {

    // open simconnect
    let name = CString::new("System Event").unwrap();
    if unsafe { SimConnect_Open(
        &mut HANDLE, name.as_ptr(), 
        std::ptr::null_mut(), 
        0, 
        std::ptr::null_mut(), 
        0
    ) } != 0 {
        return Err("Error: SimConnect_Open failed!");
    }
    println!("Connected to Flight Simulator!");

    // Request a simulation started event
    let name = CString::new("FlightLoaded").unwrap();
    if unsafe { SimConnect_SubscribeToSystemEvent(HANDLE, EVENT_FLIGHT_LOAD, name.as_ptr()) } != 0 {
        return Err("Error: SimConnect_SubscribeToSystemEvent failed!");
    }
    println!("Please load a flight...");

    // Run dispatch loop
    while QUIT.load(Ordering::Relaxed) == false {
        if unsafe { SimConnect_CallDispatch(HANDLE, Some(my_dispatch_proc), std::ptr::null_mut()) } != 0 {
            return Err("Error: SimConnect_CallDispatch failed!");
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    // close simconnect
    if unsafe { SimConnect_Close(HANDLE) } != 0 {
        return Err("Error: SimConnect_Close failed!");
    }
    println!("Disconnected from Flight Simulator");

    Ok(())
}