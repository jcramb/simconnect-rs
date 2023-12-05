use std::ffi::{CString, c_void};
use std::sync::atomic::{AtomicBool, Ordering};

use simconnect_sys::*;

const GROUP_ZX: u32 = 1;
const INPUT_ZX: u32 = 1;
const EVENT_Z: u32 = 1;
const EVENT_X: u32 = 2;
const EVENT_ADDED_AIRCRAFT: u32 = 3;
const EVENT_REMOVED_AIRCRAFT: u32 = 4;
const REQUEST_VL3: u32 = 1;
const REQUEST_VL3_PARKED: u32 = 2;
const REQUEST_747_PARKED: u32 = 3;
const REQUEST_LV3_PARKED_FLIGHTPLAN: u32 = 4;
const REQUEST_747_PARKED_FLIGHTPLAN: u32 = 5;

static mut FLIGHTPLAN: Option<String> = None;

static mut PARKED_VL3_ID: u32 = SIMCONNECT_OBJECT_ID_USER;
static mut PARKED_747_ID: u32 = SIMCONNECT_OBJECT_ID_USER;

static mut HANDLE: *mut c_void = std::ptr::null_mut(); 
static QUIT: AtomicBool = AtomicBool::new(false);

// Set up flags so these operations only happen once
static mut PLANS_SENT: bool = false;
static mut AIRCRAFT_CREATED: bool = false;

fn setup_ai_aircraft() {
    
    let title = CString::new("VL3 Asobo").unwrap();
    let tail = CString::new("N200").unwrap();
    let route = CString::new(unsafe { FLIGHTPLAN.clone().unwrap() }).unwrap();
    if unsafe { SimConnect_AICreateEnrouteATCAircraft(HANDLE, title.as_ptr(), tail.as_ptr(), 200, route.as_ptr(), 0.0, 0, REQUEST_VL3) } != 0 {
        eprintln!("Error: SimConnect_AICreateEnrouteATCAircraft failed!");
    }

    // park a few aircraft
    let tail = CString::new("N102").unwrap();
    let icao = CString::new("KYKM").unwrap();
    if unsafe { SimConnect_AICreateParkedATCAircraft(HANDLE, title.as_ptr(), tail.as_ptr(), icao.as_ptr(), REQUEST_VL3_PARKED) } != 0 {
        eprintln!("Error: SimConnect_AICreateParkedATCAircraft failed!");
    }

    // will fail, as the aircraft is too big
    let title = CString::new("Boeing 747-8f Asobo").unwrap();
    let tail = CString::new("N202").unwrap();
    if unsafe { SimConnect_AICreateParkedATCAircraft(HANDLE, title.as_ptr(), tail.as_ptr(), icao.as_ptr(), REQUEST_747_PARKED) } != 0 {
        eprintln!("FAILURE - As expected failed to create parked 747.");
    }
}

fn send_flight_plans() {
    unsafe {
        let path = CString::new(FLIGHTPLAN.clone().unwrap()).unwrap();
        println!("Flightplan file: '{}.pln'", path.as_c_str().to_str().unwrap());
        if PARKED_VL3_ID != SIMCONNECT_OBJECT_ID_USER {
            println!("Sending flight plan to VL3. (ObjectID {})", PARKED_VL3_ID);
            if SimConnect_AISetAircraftFlightPlan(HANDLE, PARKED_VL3_ID, path.as_ptr(), REQUEST_LV3_PARKED_FLIGHTPLAN) != 0 {
                eprintln!("Error: SimConnect_AISetAircraftFlightPlan failed!");
            }
        }
        // this will fail, as the 747 will not be created
        if PARKED_747_ID != SIMCONNECT_OBJECT_ID_USER {
            if SimConnect_AISetAircraftFlightPlan(HANDLE, PARKED_747_ID, path.as_ptr(), REQUEST_747_PARKED_FLIGHTPLAN) != 0 {
                eprintln!("FAILURE - As expected couldn't set flight plan for Parked 747.");
            }
        }
    }
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
                EVENT_Z => {
                    if !AIRCRAFT_CREATED {
                        AIRCRAFT_CREATED = true;
                        setup_ai_aircraft();
                        println!("Press 'x' to change the flight plans");
                    }
                },
                EVENT_X => {
                    if !PLANS_SENT && AIRCRAFT_CREATED {
                        PLANS_SENT = true;
                        send_flight_plans();
                    }
                },
                _ => {
                    let event_id = (*event).uEventID;
                    let event_data = (*event).dwData;
                    println!("SIMCONNECT_RECV_EVENT: {:x} {:x} {:x}", event_id, event_data, cb_data);
                }
            }

        },
        SIMCONNECT_RECV_ID_EVENT_OBJECT_ADDREMOVE => {
            let event = data as *const SIMCONNECT_RECV_EVENT_OBJECT_ADDREMOVE;
            // let obj_type = (*event).eObjType;
            // let obj_data = (*event)._base.dwData;
            match (*event)._base.uEventID {
                EVENT_ADDED_AIRCRAFT => {
                    // this event is quite noisy so it's commented out by default
                    // println!("AI object added: Type={}, ObjectID={}", obj_type, obj_data);
                },
                EVENT_REMOVED_AIRCRAFT => {
                    // this event is quite noisy so it's commented out by default
                    // println!("AI object removed: Type={}, ObjectID={}", obj_type, obj_data);
                },
                _ => {}
            }
        },
        SIMCONNECT_RECV_ID_ASSIGNED_OBJECT_ID => {
            let obj_data = data as *const SIMCONNECT_RECV_ASSIGNED_OBJECT_ID;
            let object_id = (*obj_data).dwObjectID;
            match (*obj_data).dwRequestID {
                REQUEST_VL3 => {
                    println!("Created Asobo VL3 id = {}", object_id);
                },
                // record the object id so the flight plan can be sent out later
                REQUEST_VL3_PARKED => {
                    PARKED_VL3_ID = object_id;
                    println!("Created parked Asobo VL3 id = {}", object_id);
                },
                // this wil not happen, as the 747 is too big
                REQUEST_747_PARKED => {
                    PARKED_747_ID = object_id;
                    println!("Created parked 747 400 id = {}", object_id);
                },
                _ => {
                    println!("Unknown creation id = {}", object_id);
                }
            }
        },
        SIMCONNECT_RECV_ID_OPEN => {
            println!("SIMCONNECT_RECV_OPEN: data={:p} cb_data={:x}", data, cb_data);
        },
        SIMCONNECT_RECV_ID_QUIT => {
            QUIT.store(true, Ordering::Relaxed);
        },
        SIMCONNECT_RECV_ID_EXCEPTION => {
            let e = data as *mut SIMCONNECT_RECV_EXCEPTION;
            eprintln!("{}", exception_str((*e).dwException));
        },
        _ => {
            println!("DATA RECEIVED: data={:p} cb_data={:x}", data, cb_data);
        }
    }
}

fn main() -> Result<(), &'static str> {

    // set flightplan filepath
    let dir = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    unsafe {
        FLIGHTPLAN = Some(dir.join("IFR Yakima Air Term Mcallister to Spokane Intl").to_str().unwrap().to_string());
    }

    // open simconnect
    let name = CString::new("AI Traffic").unwrap();
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

    // Create some private events
    for event in vec![
        EVENT_Z,
        EVENT_X,
    ] {
        if unsafe { SimConnect_MapClientEventToSimEvent(HANDLE, event, std::ptr::null()) } != 0 {
            return Err("Error: SimConnect_MapClientEventToSimEvent(A) failed!");
        }
    }

    // Link the private events to keyboard keys, and ensure the input events are off
    for (input, name, event) in vec![
        (INPUT_ZX, "Z", EVENT_Z),
        (INPUT_ZX, "X", EVENT_X),
    ] {
        let name = CString::new(name).unwrap();
        if unsafe { SimConnect_MapInputEventToClientEvent(HANDLE, 
            input, 
            name.as_ptr(),
            event,
            0, SIMCONNECT_UNUSED, 0, 0
        ) } != 0 {
            return Err("Error: SimConnect_MapInputEventToClientEvent failed!");
        }
    }
    if unsafe { SimConnect_SetInputGroupState(HANDLE, INPUT_ZX, SIMCONNECT_STATE_OFF as u32) } != 0 {
        return Err("Error: SimConnect_SetInputGroupState failed!");
    }

    // Sign up for notifications
    for event in vec![
        EVENT_Z,
        EVENT_X,
    ] {
        if unsafe { SimConnect_AddClientEventToNotificationGroup(HANDLE, GROUP_ZX, event, 0) } != 0 {
            return Err("Error: SimConnect_AddClientEventToNotificationGroup failed!");
        }
    }

    // Subscribe to system events notifying the client that objects have been added or removed
    let name = CString::new("ObjectAdded").unwrap();
    if unsafe { SimConnect_SubscribeToSystemEvent(HANDLE, EVENT_ADDED_AIRCRAFT, name.as_ptr()) } != 0 {
        return Err("Error: SimConnect_SubscribeToSystemEvent failed!");
    }
    let name = CString::new("ObjectRemoved").unwrap();
    if unsafe { SimConnect_SubscribeToSystemEvent(HANDLE, EVENT_REMOVED_AIRCRAFT, name.as_ptr()) } != 0 {
        return Err("Error: SimConnect_SubscribeToSystemEvent failed!");
    }

    println!("Press 'z' to create new aircraft");
    if unsafe { SimConnect_SetInputGroupState(HANDLE, INPUT_ZX, SIMCONNECT_STATE_ON as u32) } != 0 {
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


fn exception_str(code: u32) -> &'static str {
    match code as i32 {
        SIMCONNECT_EXCEPTION_NONE => "SIMCONNECT_EXCEPTION_NONE",
        SIMCONNECT_EXCEPTION_ERROR => "SIMCONNECT_EXCEPTION_ERROR",
        SIMCONNECT_EXCEPTION_SIZE_MISMATCH => "SIMCONNECT_EXCEPTION_SIZE_MISMATCH",
        SIMCONNECT_EXCEPTION_UNRECOGNIZED_ID => "SIMCONNECT_EXCEPTION_UNRECOGNIZED_ID",
        SIMCONNECT_EXCEPTION_UNOPENED => "SIMCONNECT_EXCEPTION_UNOPENED",
        SIMCONNECT_EXCEPTION_VERSION_MISMATCH => "SIMCONNECT_EXCEPTION_VERSION_MISMATCH",
        SIMCONNECT_EXCEPTION_TOO_MANY_GROUPS => "SIMCONNECT_EXCEPTION_TOO_MANY_GROUPS",
        SIMCONNECT_EXCEPTION_NAME_UNRECOGNIZED => "SIMCONNECT_EXCEPTION_NAME_UNRECOGNIZED",
        SIMCONNECT_EXCEPTION_TOO_MANY_EVENT_NAMES => "SIMCONNECT_EXCEPTION_TOO_MANY_EVENT_NAMES",
        SIMCONNECT_EXCEPTION_EVENT_ID_DUPLICATE => "SIMCONNECT_EXCEPTION_EVENT_ID_DUPLICATE",
        SIMCONNECT_EXCEPTION_TOO_MANY_MAPS => "SIMCONNECT_EXCEPTION_TOO_MANY_MAPS",
        SIMCONNECT_EXCEPTION_TOO_MANY_OBJECTS => "SIMCONNECT_EXCEPTION_TOO_MANY_OBJECTS",
        SIMCONNECT_EXCEPTION_TOO_MANY_REQUESTS => "SIMCONNECT_EXCEPTION_TOO_MANY_REQUESTS",
        SIMCONNECT_EXCEPTION_WEATHER_INVALID_PORT => "SIMCONNECT_EXCEPTION_WEATHER_INVALID_PORT",
        SIMCONNECT_EXCEPTION_WEATHER_INVALID_METAR => "SIMCONNECT_EXCEPTION_WEATHER_INVALID_METAR",
        SIMCONNECT_EXCEPTION_WEATHER_UNABLE_TO_GET_OBSERVATION => "SIMCONNECT_EXCEPTION_WEATHER_UNABLE_TO_GET_OBSERVATION",
        SIMCONNECT_EXCEPTION_WEATHER_UNABLE_TO_CREATE_STATION => "SIMCONNECT_EXCEPTION_WEATHER_UNABLE_TO_CREATE_STATION",
        SIMCONNECT_EXCEPTION_WEATHER_UNABLE_TO_REMOVE_STATION => "SIMCONNECT_EXCEPTION_WEATHER_UNABLE_TO_REMOVE_STATION",
        SIMCONNECT_EXCEPTION_INVALID_DATA_TYPE => "SIMCONNECT_EXCEPTION_INVALID_DATA_TYPE",
        SIMCONNECT_EXCEPTION_INVALID_DATA_SIZE => "SIMCONNECT_EXCEPTION_INVALID_DATA_SIZE",
        SIMCONNECT_EXCEPTION_DATA_ERROR => "SIMCONNECT_EXCEPTION_DATA_ERROR",
        SIMCONNECT_EXCEPTION_INVALID_ARRAY => "SIMCONNECT_EXCEPTION_INVALID_ARRAY",
        SIMCONNECT_EXCEPTION_CREATE_OBJECT_FAILED => "SIMCONNECT_EXCEPTION_CREATE_OBJECT_FAILED",
        SIMCONNECT_EXCEPTION_LOAD_FLIGHTPLAN_FAILED => "SIMCONNECT_EXCEPTION_LOAD_FLIGHTPLAN_FAILED",
        SIMCONNECT_EXCEPTION_OPERATION_INVALID_FOR_OBJECT_TYPE => "SIMCONNECT_EXCEPTION_OPERATION_INVALID_FOR_OBJECT_TYPE",
        SIMCONNECT_EXCEPTION_ILLEGAL_OPERATION => "SIMCONNECT_EXCEPTION_ILLEGAL_OPERATION",
        SIMCONNECT_EXCEPTION_ALREADY_SUBSCRIBED => "SIMCONNECT_EXCEPTION_ALREADY_SUBSCRIBED",
        SIMCONNECT_EXCEPTION_INVALID_ENUM => "SIMCONNECT_EXCEPTION_INVALID_ENUM",
        SIMCONNECT_EXCEPTION_DEFINITION_ERROR => "SIMCONNECT_EXCEPTION_DEFINITION_ERROR",
        SIMCONNECT_EXCEPTION_DUPLICATE_ID => "SIMCONNECT_EXCEPTION_DUPLICATE_ID",
        SIMCONNECT_EXCEPTION_DATUM_ID => "SIMCONNECT_EXCEPTION_DATUM_ID",
        SIMCONNECT_EXCEPTION_OUT_OF_BOUNDS => "SIMCONNECT_EXCEPTION_OUT_OF_BOUNDS",
        SIMCONNECT_EXCEPTION_ALREADY_CREATED => "SIMCONNECT_EXCEPTION_ALREADY_CREATED",
        SIMCONNECT_EXCEPTION_OBJECT_OUTSIDE_REALITY_BUBBLE => "SIMCONNECT_EXCEPTION_OBJECT_OUTSIDE_REALITY_BUBBLE",
        SIMCONNECT_EXCEPTION_OBJECT_CONTAINER => "SIMCONNECT_EXCEPTION_OBJECT_CONTAINER",
        SIMCONNECT_EXCEPTION_OBJECT_AI => "SIMCONNECT_EXCEPTION_OBJECT_AI",
        SIMCONNECT_EXCEPTION_OBJECT_ATC => "SIMCONNECT_EXCEPTION_OBJECT_ATC",
        SIMCONNECT_EXCEPTION_OBJECT_SCHEDULE => "SIMCONNECT_EXCEPTION_OBJECT_SCHEDULE",
        SIMCONNECT_EXCEPTION_JETWAY_DATA => "SIMCONNECT_EXCEPTION_JETWAY_DATA",
        SIMCONNECT_EXCEPTION_ACTION_NOT_FOUND => "SIMCONNECT_EXCEPTION_ACTION_NOT_FOUND",
        SIMCONNECT_EXCEPTION_NOT_AN_ACTION => "SIMCONNECT_EXCEPTION_NOT_AN_ACTION",
        SIMCONNECT_EXCEPTION_INCORRECT_ACTION_PARAMS => "SIMCONNECT_EXCEPTION_INCORRECT_ACTION_PARAMS",
        SIMCONNECT_EXCEPTION_GET_INPUT_EVENT_FAILED => "SIMCONNECT_EXCEPTION_GET_INPUT_EVENT_FAILED",
        SIMCONNECT_EXCEPTION_SET_INPUT_EVENT_FAILED => "SIMCONNECT_EXCEPTION_SET_INPUT_EVENT_FAILED",
        _ => "UNKNOWN EXCEPTION"
    }
}