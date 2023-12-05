use std::ffi::{CString, c_void};
use std::sync::atomic::{AtomicBool, Ordering};

use simconnect_sys::*;

const GROUP_C: u32 = 1;
const EVENT_MY_EVENTC: u32 = 1;
const EVENT_MASKABLEC: u32 = 2;

static mut HANDLE: *mut c_void = std::ptr::null_mut(); 
static QUIT: AtomicBool = AtomicBool::new(false);

unsafe extern "C" fn my_dispatch_proc(
    data: *mut SIMCONNECT_RECV,
    _cb_data: DWORD,
    _context: *mut c_void,
) {
    match (*data).dwID as i32 {
        SIMCONNECT_RECV_ID_EVENT => {
            let event = data as *const SIMCONNECT_RECV_EVENT;
            match (*event).uEventID {
                EVENT_MY_EVENTC => {
                    println!("Send Event C received My.event");
                },
                EVENT_MASKABLEC => {
                    println!("Send Event C received by My.maskable.event");
                },
                _ => {}
            }
        },
        SIMCONNECT_RECV_ID_QUIT => {
            QUIT.store(true, Ordering::Relaxed);
        },
        _ => {}
    }
}

fn main() -> Result<(), &'static str> {

    // open simconnect
    let name = CString::new("Send Event C").unwrap();
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

    // Set up to receive the "My.event" notification, without masking it
    let name = CString::new("My.event").unwrap();
    if unsafe { SimConnect_MapClientEventToSimEvent(HANDLE, 
        EVENT_MY_EVENTC, 
        name.as_ptr()
    ) } != 0 {
        return Err("Error: SimConnect_MapClientEventToSimEvent failed!");
    }
    if unsafe { SimConnect_AddClientEventToNotificationGroup(HANDLE, 
        GROUP_C, 
        EVENT_MY_EVENTC,
        0
    ) } != 0 {
        return Err("Error: SimConnect_AddClientEventToNotificationGroup failed!");
    }

    // Set up to receive the "My.maskable.event" notification, and mask it
    // from lower priority client groups
    let name = CString::new("My.maskable.event").unwrap();
    if unsafe { SimConnect_MapClientEventToSimEvent(HANDLE,
        EVENT_MASKABLEC,
        name.as_ptr(),
    ) } != 0 {
        return Err("Error: SimConnect_MapClientEventToSimEvent failed!");
    }
    if unsafe { SimConnect_AddClientEventToNotificationGroup(HANDLE, 
        GROUP_C, 
        EVENT_MASKABLEC,
        1
    ) } != 0 {
        return Err("Error: SimConnect_AddClientEventToNotificationGroup failed!");
    }

    // The group priority is set to low
    if unsafe { SimConnect_SetNotificationGroupPriority(HANDLE,
        GROUP_C,
        SIMCONNECT_GROUP_PRIORITY_DEFAULT
    ) } != 0 {
        return Err("Error: SimConnect_SetNotificationGroupPriority failed!");
    }

    // Run dispatch loop
    while QUIT.load(Ordering::Relaxed) == false {
        if unsafe { SimConnect_CallDispatch(HANDLE, 
            Some(my_dispatch_proc), 
            std::ptr::null_mut()
        ) } != 0 {
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