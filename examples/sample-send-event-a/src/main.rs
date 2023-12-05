use std::ffi::{CString, c_void};
use std::sync::atomic::{AtomicBool, Ordering};

use simconnect_sys::*;

const GROUP_A: u32 = 1;
const EVENT_BRAKES: u32 = 1;
const EVENT_MY_EVENT: u32 = 2;
const EVENT_MASKABLE: u32 = 3;

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
                EVENT_BRAKES => {
                    let event_data = (*event).dwData;
                    println!("Event brakes: {}", event_data);

                    // Send the two events to all other client groups
                    // This is achieved by setting the priority of the
                    // message to SIMCONNECT_GROUP_PRIORITY_HIGHEST.
                    // This is the priority of the first client group
                    // that will be sent the message.

                    if unsafe { SimConnect_TransmitClientEvent(HANDLE,
                        0,
                        EVENT_MY_EVENT,
                        0,
                        SIMCONNECT_GROUP_PRIORITY_HIGHEST,
                        SIMCONNECT_EVENT_FLAG_GROUPID_IS_PRIORITY
                    ) } != 0 {
                        eprintln!("Error: SimConnect_TransmitClientEvent failed!");
                    }
                    if unsafe { SimConnect_TransmitClientEvent(HANDLE,
                        0,
                        EVENT_MASKABLE,
                        0,
                        SIMCONNECT_GROUP_PRIORITY_HIGHEST,
                        SIMCONNECT_EVENT_FLAG_GROUPID_IS_PRIORITY
                    ) } != 0 {
                        eprintln!("Error: SimConnect_TransmitClientEvent failed!");
                    }
                },
                EVENT_MY_EVENT => {
                    println!("Send Event A received My.event");
                },
                EVENT_MASKABLE => {
                    println!("Send Event A received My.maskable.event");
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
    let name = CString::new("Send Event A").unwrap();
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

    // Set up to receive the "brakes" notification
    let name = CString::new("brakes").unwrap();
    if unsafe { SimConnect_MapClientEventToSimEvent(HANDLE, 
        EVENT_BRAKES, 
        name.as_ptr()
    ) } != 0 {
        return Err("Error: SimConnect_MapClientEventToSimEvent failed!");
    }
    if unsafe { SimConnect_AddClientEventToNotificationGroup(HANDLE, 
        GROUP_A, 
        EVENT_BRAKES,
        0
    ) } != 0 {
        return Err("Error: SimConnect_AddClientEventToNotificationGroup failed!");
    }

    // Define two custom events, both of which this client will not mask
    let name = CString::new("My.event").unwrap();
    if unsafe { SimConnect_MapClientEventToSimEvent(HANDLE,
        EVENT_MY_EVENT,
        name.as_ptr(),
    ) } != 0 {
        return Err("Error: SimConnect_MapClientEventToSimEvent failed!");
    }
    if unsafe { SimConnect_AddClientEventToNotificationGroup(HANDLE, 
        GROUP_A, 
        EVENT_MY_EVENT,
        0
    ) } != 0 {
        return Err("Error: SimConnect_AddClientEventToNotificationGroup failed!");
    }
    let name = CString::new("My.maskable.event").unwrap();
    if unsafe { SimConnect_MapClientEventToSimEvent(HANDLE,
        EVENT_MASKABLE,
        name.as_ptr(),
    ) } != 0 {
        return Err("Error: SimConnect_MapClientEventToSimEvent failed!");
    }
    if unsafe { SimConnect_AddClientEventToNotificationGroup(HANDLE, 
        GROUP_A, 
        EVENT_MASKABLE,
        0
    ) } != 0 {
        return Err("Error: SimConnect_AddClientEventToNotificationGroup failed!");
    }

    // Set the priority of the group
    if unsafe { SimConnect_SetNotificationGroupPriority(HANDLE,
        GROUP_A,
        SIMCONNECT_GROUP_PRIORITY_HIGHEST
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