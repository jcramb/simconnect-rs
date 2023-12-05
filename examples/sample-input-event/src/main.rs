use std::ffi::{CString, c_void};
use std::sync::atomic::{AtomicBool, Ordering};

use simconnect_sys::*;

const GROUP_0: u32 = 1;
const INPUT_0: u32 = 1;
const EVENT_BRAKES: u32 = 1;

static mut HANDLE: *mut c_void = std::ptr::null_mut(); 
static QUIT: AtomicBool = AtomicBool::new(false);

unsafe extern "C" fn my_dispatch_proc(
    data: *mut SIMCONNECT_RECV,
    cb_data: DWORD,
    _context: *mut c_void,
) {
    match (*data).dwID as i32 {
        SIMCONNECT_RECV_ID_EVENT => {
            let event = data as *const SIMCONNECT_RECV_EVENT;
            let data = (*event).dwData;
            match (*event).uEventID {
                EVENT_BRAKES => println!("Event brakes: {}", data),
                _ => {
                    let event_id = (*event).uEventID;
                    let event_data = (*event).dwData;
                    println!("SIMCONNECT_RECV_EVENT: {:x} {:x} {:x}", event_id, event_data, cb_data);
                }
            }

        },
        SIMCONNECT_RECV_ID_OPEN => {
            println!("SIMCONNECT_RECV_OPEN: data={:p} cb_data={:x}", data, cb_data);
        },
        SIMCONNECT_RECV_ID_QUIT => {
            QUIT.store(true, Ordering::Relaxed);
        },
        _ => {}
    }
}

fn main() -> Result<(), &'static str> {

    // open simconnect
    let name = CString::new("Input Event").unwrap();
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
    println!("Instructions: Launch a flight, then press 'z' to trigger the brakes.");

    let name = CString::new("brakes").unwrap();
    if unsafe { SimConnect_MapClientEventToSimEvent(HANDLE, EVENT_BRAKES, name.as_ptr()) } != 0 {
        return Err("Error: SimConnect_MapClientEventToSimEvent failed!");
    }
    if unsafe { SimConnect_AddClientEventToNotificationGroup(HANDLE, GROUP_0, EVENT_BRAKES, 0) } != 0 {
        return Err("Error: SimConnect_MapClientEventToSimEvent failed!");
    }
    if unsafe { SimConnect_SetNotificationGroupPriority(HANDLE, GROUP_0, SIMCONNECT_GROUP_PRIORITY_HIGHEST) } != 0 {
        return Err("Error: SimConnect_SetNotificationGroupPriority failed!");
    }

    // NOTE: This does not override '.' for brakes - both will be transmitted

    let key = CString::new("z").unwrap();
    if unsafe { SimConnect_MapInputEventToClientEvent(HANDLE, 
        INPUT_0, 
        key.as_ptr(), 
        EVENT_BRAKES,
        0, SIMCONNECT_UNUSED, 0, 0
    ) } != 0 {
        return Err("Error: SimConnect_MapInputEventToClientEvent failed!");
    }

    // Turn on the Z key
    if unsafe { SimConnect_SetInputGroupState(HANDLE, INPUT_0, SIMCONNECT_STATE_ON as u32) } != 0 {
        return Err("Error: SimConnect_SetInputGroupState failed!");
    }

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