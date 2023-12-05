use std::ffi::{CString, c_void};
use std::sync::atomic::{AtomicBool, Ordering};

use simconnect_sys::*;

const GROUP_KEYS: u32 = 1;
const INPUT_KEYS: u32 = 1;
const EVENT_SIM_START: u32 = 1;
const EVENT_A: u32 = 2;
const EVENT_Z: u32 = 3;
const DEFINITION_THROTTLE: u32 = 1;
const REQUEST_THROTTLE: u32 = 1;

static mut HANDLE: *mut c_void = std::ptr::null_mut(); 
static mut THROTTLE_PERCENT: f64 = 0.0;
static QUIT: AtomicBool = AtomicBool::new(false);

unsafe extern "C" fn my_dispatch_proc(
    data: *mut SIMCONNECT_RECV,
    cb_data: DWORD,
    _context: *mut c_void,
) {
    match (*data).dwID as i32 {
        SIMCONNECT_RECV_ID_SIMOBJECT_DATA => {
            let obj_data = data as *const SIMCONNECT_RECV_SIMOBJECT_DATA;
            match (*obj_data).dwRequestID {
                REQUEST_THROTTLE => {
                    // Read and set the initial throttle control value
                    let throttle = std::ptr::addr_of!((*obj_data).dwData) as *const f64;
                    THROTTLE_PERCENT = *throttle;
                    println!("REQUEST_USERID received, throttle = {:2.1}", THROTTLE_PERCENT);
                    
                    // Now turn the input events on
                    if unsafe { SimConnect_SetInputGroupState(HANDLE, 
                        INPUT_KEYS, 
                        SIMCONNECT_STATE_ON as u32
                    ) } != 0 {
                        eprintln!("Error: SimConnect_SetInputGroupState failed!")
                    }
                }
                _ => {}
            }
        },
        SIMCONNECT_RECV_ID_EVENT => {
            let event = data as *const SIMCONNECT_RECV_EVENT;
            match (*event).uEventID {
                EVENT_SIM_START => {
                    println!("Sim Started!");

                    // Send this request to get the user aircraft id
                    if unsafe { SimConnect_RequestDataOnSimObject(HANDLE,
                        REQUEST_THROTTLE,
                        DEFINITION_THROTTLE,
                        SIMCONNECT_OBJECT_ID_USER,
                        SIMCONNECT_PERIOD_ONCE,
                        0, 0, 0, 0
                    ) } != 0 {
                        eprintln!("Error: SimConnect_RequestDataOnSimObject failed!")
                    }
                },
                EVENT_A => {
                    // Increase the throttle
                    if THROTTLE_PERCENT <= 95.0 {
                        THROTTLE_PERCENT += 5.0;
                    }
                    if unsafe { SimConnect_SetDataOnSimObject(HANDLE,
                        DEFINITION_THROTTLE,
                        SIMCONNECT_OBJECT_ID_USER,
                        0, 0, 
                        std::mem::size_of_val(&THROTTLE_PERCENT) as u32, 
                        std::ptr::addr_of_mut!(THROTTLE_PERCENT) as *mut c_void
                    )} != 0 {
                        eprintln!("Error: SimConnect_SetDataOnSimObject(A) failed!")
                    }
                    println!("INC THROTTLE {:2.1}", THROTTLE_PERCENT);
                },
                EVENT_Z => {
                    // Decrease the throttle
                    if THROTTLE_PERCENT >= 5.0 {
                        THROTTLE_PERCENT -= 5.0;
                    }
                    if unsafe { SimConnect_SetDataOnSimObject(HANDLE,
                        DEFINITION_THROTTLE,
                        SIMCONNECT_OBJECT_ID_USER,
                        0, 0, 
                        std::mem::size_of_val(&THROTTLE_PERCENT) as u32, 
                        &mut THROTTLE_PERCENT as *mut f64 as *mut c_void
                    )} != 0 {
                        eprintln!("Error: SimConnect_SetDataOnSimObject(A) failed!")
                    }
                    println!("DEC THROTTLE {:2.1}", THROTTLE_PERCENT);
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
    let name = CString::new("Throttle Control").unwrap();
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

    // Set up a data definition for the throttle control
    let name = CString::new("GENERAL ENG THROTTLE LEVER POSITION:1").unwrap();
    let unit = CString::new("percent").unwrap();
    if unsafe { SimConnect_AddToDataDefinition(HANDLE, 
        DEFINITION_THROTTLE, 
        name.as_ptr(), 
        unit.as_ptr(), 
        SIMCONNECT_DATATYPE_FLOAT64, 
        0.0, 
        SIMCONNECT_UNUSED
    ) } != 0 {
        return Err("Error: SimConnect_AddToDataDefinition failed!");
    }

    // Request a simulation started event
    let name = CString::new("SimStart").unwrap();
    if unsafe { SimConnect_SubscribeToSystemEvent(HANDLE, EVENT_SIM_START, name.as_ptr()) } != 0 {
        return Err("Error: SimConnect_SubscribeToSystemEvent failed!");
    }

    // Create two private key events to control the throttle
    if unsafe { SimConnect_MapClientEventToSimEvent(HANDLE, EVENT_A, std::ptr::null()) } != 0 {
        return Err("Error: SimConnect_MapClientEventToSimEvent(A) failed!");
    }
    if unsafe { SimConnect_MapClientEventToSimEvent(HANDLE, EVENT_Z, std::ptr::null()) } != 0 {
        return Err("Error: SimConnect_MapClientEventToSimEvent(Z) failed!");
    }

    // Link the events to some keyboard keys
    let key_a = CString::new("A").unwrap();
    let key_z = CString::new("Z").unwrap();
    if unsafe { SimConnect_MapInputEventToClientEvent(HANDLE, 
        INPUT_KEYS, 
        key_a.as_ptr(),
        EVENT_A,
        0, SIMCONNECT_UNUSED, 0, 0
    ) } != 0 {
        return Err("Error: SimConnect_MapInputEventToClientEvent(A) failed!");
    }
    if unsafe { SimConnect_MapInputEventToClientEvent(HANDLE, 
        INPUT_KEYS, 
        key_z.as_ptr(), 
        EVENT_Z,
        0, SIMCONNECT_UNUSED, 0, 0
    ) } != 0 {
        return Err("Error: SimConnect_MapInputEventToClientEvent(Z) failed!");
    }

    // Ensure the input events are off until the sim is up and running
    if unsafe { SimConnect_SetInputGroupState(HANDLE, INPUT_KEYS, SIMCONNECT_STATE_OFF as u32) } != 0 {
        return Err("Error: SimConnect_SetInputGroupState failed!");
    }

    // Sign up for notifications
    if unsafe { SimConnect_AddClientEventToNotificationGroup(HANDLE, GROUP_KEYS, EVENT_A, 0) } != 0 {
        return Err("Error: SimConnect_AddClientEventToNotificationGroup(A) failed!");
    }
    if unsafe { SimConnect_AddClientEventToNotificationGroup(HANDLE, GROUP_KEYS, EVENT_Z, 0) } != 0 {
        return Err("Error: SimConnect_AddClientEventToNotificationGroup(Z) failed!");
    }

    println!("Please launch a flight...");

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