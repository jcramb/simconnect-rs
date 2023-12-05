use std::ffi::{CString, c_void};
use std::sync::atomic::{AtomicBool, Ordering};

use simconnect_sys::*;

static mut HANDLE: *mut c_void = std::ptr::null_mut(); 
static QUIT: AtomicBool = AtomicBool::new(false);

const EVENT_SIM_START: u32 = 1;
const DEFINITION_PDR: u32 = 1;
const REQUEST_PDR: u32 = 1;
const DATA_VERTICAL_SPEED: u32 = 1;
const DATA_PITOT_HEAT: u32 = 2;

// Sample requests 'vertical speed' and 'pitot heat'
const MAX_RETURNED_ITEMS: usize = 2;

// a basic structure for a single item of returned data
#[repr(C)]
struct StructOneDatum {
    id: u32,
    value: f32
}

#[repr(C)]
struct StructDatum {
    datum: [StructOneDatum; MAX_RETURNED_ITEMS]
}


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
                    
                    // Make the call for data every second, but only when it changes
                    // and only that data that has changed
                    println!("SIM STARTED, requesting data.");
                    if unsafe { SimConnect_RequestDataOnSimObject(HANDLE, 
                        REQUEST_PDR, 
                        DEFINITION_PDR,
                        SIMCONNECT_OBJECT_ID_USER, 
                        SIMCONNECT_PERIOD_SECOND, 
                        SIMCONNECT_DATA_REQUEST_FLAG_CHANGED | SIMCONNECT_DATA_REQUEST_FLAG_TAGGED, 
                        0, 0, 0)
                    } != 0 {
                        eprintln!("Error: SimConnect_RequestDataOnSimObject failed!");
                    }
                },
                _ => {}
            }
        },
        SIMCONNECT_RECV_ID_SIMOBJECT_DATA => {
            let obj_data = data as *const SIMCONNECT_RECV_SIMOBJECT_DATA;
            match (*obj_data).dwRequestID {
                REQUEST_PDR => {
                    let mut count: usize = 0;
                    let s = std::ptr::addr_of!((*obj_data).dwData) as *const StructDatum;

                    // There can be a minimum of 1 and a maximum of MAX_RETURNED_ITEMS
                    // in the StructDatum structure. The actual number returned will
                    // be held in the dwDefineCount parameter.
                    let define_count = (*obj_data).dwDefineCount as usize;
                    while count < define_count {

                        let id = (*s).datum[count].id;
                        let value = (*s).datum[count].value;
                        match id {
                            DATA_VERTICAL_SPEED => {
                                println!("Vertical speed = {:.2}", value)
                            },
                            DATA_PITOT_HEAT => {
                                println!("Pitot heat = {:.2}", value)
                            },
                            _ => {
                                println!("Unknown datum ID ({}): {} (value {:.2})", count, id, value);
                            }
                        }
                        count += 1;
                    }
                }
                _ => {}
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
    let name = CString::new("Tagged Data").unwrap();
    let hr = unsafe { SimConnect_Open(
        &mut HANDLE, name.as_ptr(), 
        std::ptr::null_mut(), 
        0, 
        std::ptr::null_mut(), 
        0
    ) };
    if hr != 0 {
        return Err("Error: SimConnect_Open failed!");
    }
    println!("Connected to Flight Simulator!");

    // Set up the data definition, ensuring that all the elements are in Float32 units,
    // to match the StructDatum structure. The number of entries in the DEFINITION_PDR
    // definition should be equal to the maxReturnedItems define

    let name = CString::new("Vertical Speed").unwrap();
    let unit = CString::new("Feet per second").unwrap();
    if unsafe { SimConnect_AddToDataDefinition(HANDLE, 
        DEFINITION_PDR, 
        name.as_ptr(), 
        unit.as_ptr(), 
        SIMCONNECT_DATATYPE_FLOAT32, 
        0.0, 
        DATA_VERTICAL_SPEED)
    } != 0 {
        return Err("Error: SimConnect_AddToDataDefinition failed!");
    }

    let name = CString::new("Pitot Heat").unwrap();
    let unit = CString::new("Bool").unwrap();
    if unsafe { SimConnect_AddToDataDefinition(HANDLE, 
        DEFINITION_PDR, 
        name.as_ptr(), 
        unit.as_ptr(), 
        SIMCONNECT_DATATYPE_FLOAT32, 
        0.0, 
        DATA_PITOT_HEAT)
    } != 0 {
        return Err("Error: SimConnect_AddToDataDefinition failed!");
    }

    let name = CString::new("SimStart").unwrap();
    if unsafe { SimConnect_SubscribeToSystemEvent(HANDLE, EVENT_SIM_START, name.as_ptr()) } != 0 {
        return Err("Error: SimConnect_SubscribeToSystemEvent failed!");
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
    let hr = unsafe { SimConnect_Close(HANDLE) };
    if hr != 0 {
        return Err("Error: SimConnect_Close failed!");
    }
    println!("Disconnected from Flight Simulator");
    Ok(())
}