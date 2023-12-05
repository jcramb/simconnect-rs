use std::ffi::CString;
use std::sync::atomic::{AtomicBool, Ordering};

use simconnect_sys::*;

const GROUP_10: u32 = 1;
const EVENT_BRAKES_10: u32 = 1;

static QUIT: AtomicBool = AtomicBool::new(false);

fn main() -> Result<(), &'static str> {

    // open simconnect
    let mut handle = std::ptr::null_mut();
    let name = CString::new("No Callback").unwrap();
    let hr = unsafe { SimConnect_Open(
        &mut handle, name.as_ptr(), 
        std::ptr::null_mut(), 
        0, 
        std::ptr::null_mut(), 
        0
    ) };
    if hr != 0 {
        return Err("Error: SimConnect_Open failed!");
    }
    println!("Connected to Flight Simulator!");

    println!("Activate the brakes, this will be processed in the Main loop (without the need of Callbacks).");

    let name = CString::new("brakes").unwrap();
    if unsafe { SimConnect_MapClientEventToSimEvent(handle,
        EVENT_BRAKES_10,
        name.as_ptr()
    ) } != 0 {
        return Err("Error: SimConnect_MapClientEventToSimEvent failed!");
    }
    if unsafe { SimConnect_AddClientEventToNotificationGroup(handle,
        GROUP_10,
        EVENT_BRAKES_10,
        0
    ) } != 0 {
        return Err("Error: SimConnect_AddClientEventToNotificationGroup failed!");
    }
    if unsafe { SimConnect_SetNotificationGroupPriority(handle,
        GROUP_10,
        SIMCONNECT_GROUP_PRIORITY_HIGHEST
    ) } != 0 {
        return Err("Error: SimConnect_SetNotificationGroupPriority failed!");
    }

    // Run dispatch loop
    while QUIT.load(Ordering::Relaxed) == false {
        let mut data: *mut SIMCONNECT_RECV = std::ptr::null_mut();
        let mut cb_data: u32 = 0;
        if unsafe { SimConnect_GetNextDispatch(handle, 
            std::ptr::addr_of_mut!(data),
            std::ptr::addr_of_mut!(cb_data),
        ) } != 0 {
            // TODO this fails for some reason? brakes still work though!
            // eprintln!("Error: SimConnect_GetNextDispatch failed!");
            std::thread::sleep(std::time::Duration::from_secs(1));
        } else {
            unsafe {
                let event = data as *const SIMCONNECT_RECV_EVENT;
                match (*event).uEventID {
                    EVENT_BRAKES_10 => {
                        let event_data = (*event).dwData;
                        println!("Event brakes: {} ({} bytes)", event_data, cb_data);
                    },
                    _ => {}
                }
            }   
        }
    }

    // close simconnect
    let hr = unsafe { SimConnect_Close(handle) };
    if hr != 0 {
        return Err("Error: SimConnect_Close failed!");
    }
    println!("Disconnected from Flight Simulator");

    Ok(())
}