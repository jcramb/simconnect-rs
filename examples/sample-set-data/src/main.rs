use std::ffi::{CString, c_void};
use std::sync::atomic::{AtomicBool, Ordering};

use simconnect_sys::*;

const GROUP_6: u32 = 1;
const INPUT_6: u32 = 1;
const EVENT_SIM_START: u32 = 1;
const EVENT_6: u32 = 2;
const DEFINITION_6: u32 = 1;
// const REQUEST_6: u32 = 1;

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
            match (*event).uEventID {
                EVENT_SIM_START => {
                    println!("Sim Started!");
                },
                EVENT_6 => {
                    let mut init = SIMCONNECT_DATA_INITPOSITION {
                        Altitude: 5000.0,
                        Latitude: 47.64210,
                        Longitude: -122.13010,
                        Pitch: 0.0,
                        Bank: -1.0,
                        Heading: 180.0,
                        OnGround: 0,
                        Airspeed: 60
                    };
                    if unsafe { SimConnect_SetDataOnSimObject(HANDLE,
                        DEFINITION_6,
                        SIMCONNECT_OBJECT_ID_USER,
                        0, 0,
                        std::mem::size_of::<SIMCONNECT_DATA_INITPOSITION>() as u32,
                        std::ptr::addr_of_mut!(init) as *mut c_void,
                    ) } != 0 {
                        eprintln!("Error: SimConnect_SetDataOnSimObject failed!");
                    }
                    println!("EVENT_6 received and data sent.")
                },
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
        _ => {
            println!("DATA RECEIVED: data={:p} cb_data={:x}", data, cb_data);
        }
    }
}

fn main() -> Result<(), &'static str> {

    // open simconnect
    let name = CString::new("Set Data").unwrap();
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

    // Set up a data definition for positioning data
    let name = CString::new("Initial Position").unwrap();
    let unit = CString::new("percent").unwrap();
    if unsafe { SimConnect_AddToDataDefinition(HANDLE, 
        DEFINITION_6, 
        name.as_ptr(), 
        unit.as_ptr(), 
        SIMCONNECT_DATATYPE_INITPOSITION, 
        0.0, 
        SIMCONNECT_UNUSED
    ) } != 0 {
        return Err("Error: SimConnect_AddToDataDefinition failed!");
    }

    // Request a simulation started event
    let name = CString::new("SimStart").unwrap();
    if unsafe { SimConnect_SubscribeToSystemEvent(HANDLE, 
        EVENT_SIM_START, 
        name.as_ptr()
    ) } != 0 {
        return Err("Error: SimConnect_SubscribeToSystemEvent failed!");
    }

    // Create a custom event
    let name = CString::new("My.z").unwrap();
    if unsafe { SimConnect_MapClientEventToSimEvent(HANDLE,
        EVENT_6,
        name.as_ptr(),
    ) } != 0 {
        return Err("Error: SimConnect_MapClientEventToSimEvent failed!");
    }

    // Link the custom event to some keyboard keys, and turn the input event on
    let key = CString::new("Z").unwrap();
    if unsafe { SimConnect_MapInputEventToClientEvent(HANDLE,
        INPUT_6,
        key.as_ptr(),
        EVENT_6,
        0, SIMCONNECT_UNUSED, 0, 0
    ) } != 0 {
        return Err("Error: SimConnect_MapInputEventToClientEvent failed!");
    }
    if unsafe { SimConnect_SetInputGroupState(HANDLE,
        INPUT_6,
        SIMCONNECT_STATE_ON as u32,
    ) } != 0 {
        return Err("Error: SimConnect_SetInputGroupState failed!");
    }

    // Sign up for notifications for EVENT_6
    if unsafe { SimConnect_AddClientEventToNotificationGroup(HANDLE,
        GROUP_6,
        EVENT_6,
        0
    ) } != 0 {
        return Err("Error: SimConnect_MapInputEventToClientEvent failed!");
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