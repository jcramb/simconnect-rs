use std::ffi::{CString, c_void};
use std::sync::atomic::{AtomicBool, Ordering};

use simconnect_sys::*;

const EVENT_SIM_START: u32 = 1;
const DEFINITION_1: u32 = 1;
const REQUEST_1: u32 = 1;

static mut HANDLE: *mut c_void = std::ptr::null_mut(); 
static QUIT: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct Struct1 {
    title: [u8; 256],
    kohlsmann: f64,
    altitude: f64,
    latitude: f64,
    longitude: f64,
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
                    println!("Sim Started!");
                    
                    // Now the sim is running, request information on the user aircraft
                    if unsafe { SimConnect_RequestDataOnSimObjectType(HANDLE,
                        REQUEST_1, 
                        DEFINITION_1,
                        0,
                        SIMCONNECT_SIMOBJECT_TYPE_USER
                    ) } != 0 {
                        eprintln!("Error: SimConnect_RequestDataOnSimObjectType failed!");
                    }
                },
                _ => {}
            }
        },
        SIMCONNECT_RECV_ID_SIMOBJECT_DATA_BYTYPE => {
            let obj_data = data as *const SIMCONNECT_RECV_SIMOBJECT_DATA_BYTYPE;
            match (*obj_data)._base.dwRequestID {
                REQUEST_1 => {
                    let object_id = (*obj_data)._base.dwObjectID;
                    let struct1 = *(std::ptr::addr_of!((*obj_data)._base.dwData) as *const Struct1);
                    let title = std::str::from_utf8_unchecked(std::ffi::CStr::from_ptr(
                        &struct1.title as *const u8 as *const i8).to_bytes()
                    ).to_string();
                    println!("ObjectID={}  Title='{}'", object_id, title);
                    println!("Lat={}  Lon={}  Alt={}  Kohlsman={:.2}", 
                        struct1.latitude, struct1.longitude, struct1.altitude, struct1.kohlsmann);
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
        _ => {
            println!("RECEIVED: data={:p} cb_data={:x}", data, cb_data);
        }
    }
}

fn main() -> Result<(), &'static str> {

    // open simconnect
    let name = CString::new("Request Data").unwrap();
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

    // Set up a data definition, but do not do anything with it
    let name = CString::new("Title").unwrap();
    if unsafe { SimConnect_AddToDataDefinition(HANDLE, 
        DEFINITION_1, 
        name.as_ptr(), 
        std::ptr::null(), 
        SIMCONNECT_DATATYPE_STRING256, 
        0.0, SIMCONNECT_UNUSED
    ) } != 0 {
        return Err("Error: SimConnect_AddToDataDefinition failed!");
    }
    let name = CString::new("Kohlsman setting hg").unwrap();
    let unit = CString::new("inHg").unwrap();
    if unsafe { SimConnect_AddToDataDefinition(HANDLE, 
        DEFINITION_1, 
        name.as_ptr(), 
        unit.as_ptr(), 
        SIMCONNECT_DATATYPE_FLOAT64, 
        0.0, SIMCONNECT_UNUSED
    ) } != 0 {
        return Err("Error: SimConnect_AddToDataDefinition failed!");
    }
    let name = CString::new("Plane Altitude").unwrap();
    let unit = CString::new("feet").unwrap();
    if unsafe { SimConnect_AddToDataDefinition(HANDLE, 
        DEFINITION_1, 
        name.as_ptr(), 
        unit.as_ptr(), 
        SIMCONNECT_DATATYPE_FLOAT64, 
        0.0, SIMCONNECT_UNUSED
    ) } != 0 {
        return Err("Error: SimConnect_AddToDataDefinition failed!");
    }
    let name = CString::new("Plane Latitude").unwrap();
    let unit = CString::new("degrees").unwrap();
    if unsafe { SimConnect_AddToDataDefinition(HANDLE, 
        DEFINITION_1, 
        name.as_ptr(), 
        unit.as_ptr(), 
        SIMCONNECT_DATATYPE_FLOAT64, 
        0.0, SIMCONNECT_UNUSED
    ) } != 0 {
        return Err("Error: SimConnect_AddToDataDefinition failed!");
    }
    let name = CString::new("Plane Longitude").unwrap();
    let unit = CString::new("degrees").unwrap();
    if unsafe { SimConnect_AddToDataDefinition(HANDLE, 
        DEFINITION_1, 
        name.as_ptr(), 
        unit.as_ptr(), 
        SIMCONNECT_DATATYPE_FLOAT64, 
        0.0, SIMCONNECT_UNUSED
    ) } != 0 {
        return Err("Error: SimConnect_AddToDataDefinition failed!");
    }

    // Request an event when the simulation starts
    let name = CString::new("SimStart").unwrap();
    if unsafe { SimConnect_SubscribeToSystemEvent(HANDLE, EVENT_SIM_START, name.as_ptr()) } != 0 {
        return Err("Error: SimConnect_SubscribeToSystemEvent failed!");
    }

    println!("Launch a flight...");

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