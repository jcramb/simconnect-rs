use std::ffi::{CString, c_void};
use std::sync::atomic::{AtomicBool, Ordering};

use simconnect_sys::*;

const GROUP_ZX: u32 = 1;
const INPUT_ZX: u32 = 1;
const EVENT_Z: u32 = 2;
const EVENT_X: u32 = 3;
const EVENT_C: u32 = 4;
const DEFINITION_WAYPOINT: u32 = 1;
const REQUEST_ADD_DA62: u32 = 1;
const REQUEST_ADD_PITTS: u32 = 2;
const REQUEST_ADD_GIRAFFE: u32 = 3;
const REQUEST_ADD_TRUCK: u32 = 4;
const REQUEST_REMOVE_DA62: u32 = 5;
const REQUEST_REMOVE_PITTS: u32 = 6;
const REQUEST_REMOVE_GIRAFFE: u32 = 7;
const REQUEST_REMOVE_TRUCK: u32 = 8;

static mut DA62_ID: u32 = SIMCONNECT_OBJECT_ID_USER;
static mut PITTS_ID: u32 = SIMCONNECT_OBJECT_ID_USER;
static mut GIRAFFE_ID: u32 = SIMCONNECT_OBJECT_ID_USER;
static mut TRUCK_ID: u32 = SIMCONNECT_OBJECT_ID_USER;

static mut HANDLE: *mut c_void = std::ptr::null_mut(); 
static QUIT: AtomicBool = AtomicBool::new(false);

// Set up flags so these operations only happen once
static mut PLANS_SENT: bool = false;
static mut OBJECTS_CREATED: bool = false;

fn setup_sim_objects() {
    
    // Add a parked "Aviat Pitts Special S2S" aircraft, just west of the runway
    let init = SIMCONNECT_DATA_INITPOSITION {
        Altitude: 433.0,				    // Altitude of Sea-tac is 433 feet
        Latitude: 47.0 + (25.96 / 60.0),	// Convert from 47.0 25.96 N
        Longitude: -122.0 - (18.51 / 60.0),	// Convert from 122 18.51 W
        Pitch: 0.0,
        Bank: 0.0,
        Heading: 0.0,
        OnGround: 1,
        Airspeed: 0,
    };
    let title = CString::new("PITTS ASOBO").unwrap();
    if unsafe { SimConnect_AICreateSimulatedObject(HANDLE, title.as_ptr(), init, REQUEST_ADD_PITTS) } != 0 {
        eprintln!("Error: SimConnect_AICreateSimulatedObject failed!");
    }
    
    // initialize DA62 aircraft just in front of user aircraft
    let init = SIMCONNECT_DATA_INITPOSITION {
        Altitude: 433.0,				    // Altitude of Sea-tac is 433 feet
        Latitude: 47.0 + (25.95 / 60.0),	// Convert from 47.0 25.95 N
        Longitude: -122.0 - (18.48 / 60.0),	// Convert from 122 18.48 W
        Pitch: 0.0,
        Bank: 0.0,
        Heading: 360.0,
        OnGround: 1,
        Airspeed: 1,
    };
    let title = CString::new("DA62 ASOBO").unwrap();
    let tail = CString::new("N1001").unwrap();
    if unsafe { SimConnect_AICreateNonATCAircraft(HANDLE, title.as_ptr(), tail.as_ptr(), init, REQUEST_ADD_DA62) } != 0 {
        eprintln!("Error: SimConnect_AICreateSimulatedObject failed!");
    }
    
    // initialize truck in front and to the right of user aircraft
    let init = SIMCONNECT_DATA_INITPOSITION {
        Altitude: 433.0,				    // Altitude of Sea-tac is 433 feet
        Latitude: 47.0 + (25.95 / 60.0),	// Convert from 47.0 25.95 N
        Longitude: -122.0 - (18.47 / 60.0),	// Convert from 122 18.47.0 W
        Pitch: 0.0,
        Bank: 0.0,
        Heading: 360.0,
        OnGround: 1,
        Airspeed: 0,
    };
    let title = CString::new("ASO_Firetruck01").unwrap();
    if unsafe { SimConnect_AICreateSimulatedObject(HANDLE, title.as_ptr(), init, REQUEST_ADD_TRUCK) } != 0 {
        eprintln!("Error: SimConnect_AICreateSimulatedObject failed!");
    }
    
    // Add a giraffe to the left of the user aircraft, next to the trees
    let init = SIMCONNECT_DATA_INITPOSITION {
        Altitude: 433.0,				    // Altitude of Sea-tac is 433 feet
        Latitude: 47.0 + (25.96 / 60.0),	// Convert from 47.0 25.96 N
        Longitude: -122.0 - (18.52 / 60.0),	// Convert from 122 18.52 W
        Pitch: 0.0,
        Bank: 0.0,
        Heading: 0.0,
        OnGround: 1,
        Airspeed: 0,
    };
    let title = CString::new("ReticulatedGiraffe").unwrap();
    if unsafe { SimConnect_AICreateSimulatedObject(HANDLE, title.as_ptr(), init, REQUEST_ADD_GIRAFFE) } != 0 {
        eprintln!("Error: SimConnect_AICreateSimulatedObject failed!");
    }
}

fn send_flight_plans() {

    // DA62 aircraft should fly in circles across the North end of the runway
    let mut waypoint_list_da62 = [
        SIMCONNECT_DATA_WAYPOINT {
            Flags: SIMCONNECT_WAYPOINT_SPEED_REQUESTED,
            Altitude: 800.0,
            Latitude: 47.0 + (27.79 / 60.0),
            Longitude: -122.0 - (18.46 / 60.0),
            ktsSpeed: 100.0,
            percentThrottle: 0.0,    
        },
        SIMCONNECT_DATA_WAYPOINT {
            Flags: SIMCONNECT_WAYPOINT_SPEED_REQUESTED,
            Altitude: 600.0,
            Latitude: 47.0 + (27.79 / 60.0),
            Longitude: -122.0 - (18.37 / 60.0),
            ktsSpeed: 100.0,
            percentThrottle: 0.0,    
        },
        SIMCONNECT_DATA_WAYPOINT {
            Flags: SIMCONNECT_WAYPOINT_WRAP_TO_FIRST | SIMCONNECT_WAYPOINT_SPEED_REQUESTED,
            Altitude: 800.0,
            Latitude: 47.0 + (27.79 / 60.0),
            Longitude: -122.0 - (19.92 / 60.0),
            ktsSpeed: 100.0,
            percentThrottle: 0.0,    
        }
    ];

    // Truck goes down the runway
    let mut waypoint_list_truck = [
        SIMCONNECT_DATA_WAYPOINT {
            Flags: SIMCONNECT_WAYPOINT_SPEED_REQUESTED,
            Altitude: 433.0,
            Latitude: 47.0 + (25.95 / 60.0),
            Longitude: -122.0 - (18.47 / 60.0),
            ktsSpeed: 75.0,
            percentThrottle: 0.0,    
        },
        SIMCONNECT_DATA_WAYPOINT {
            Flags: SIMCONNECT_WAYPOINT_WRAP_TO_FIRST | SIMCONNECT_WAYPOINT_SPEED_REQUESTED,
            Altitude: 433.0,
            Latitude: 47.0 + (26.25 / 60.0),
            Longitude: -122.0 - (18.46 / 60.0),
            ktsSpeed: 55.0,
            percentThrottle: 0.0,    
        },
    ];

    // giraffe walks in circles to the left of the user aircraft
    let mut waypoint_list_giraffe = [
        SIMCONNECT_DATA_WAYPOINT {
            Flags: SIMCONNECT_WAYPOINT_SPEED_REQUESTED,
            Altitude: 433.0,
            Latitude: 47.0 + (25.96 / 60.0),
            Longitude: -122.0 - (18.52 / 60.0),
            ktsSpeed: 5.0,
            percentThrottle: 0.0,    
        },
        SIMCONNECT_DATA_WAYPOINT {
            Flags: SIMCONNECT_WAYPOINT_WRAP_TO_FIRST | SIMCONNECT_WAYPOINT_SPEED_REQUESTED,
            Altitude: 433.0,
            Latitude: 47.0 + (25.98 / 60.0),
            Longitude: -122.0 - (18.55 / 60.0),
            ktsSpeed: 5.0,
            percentThrottle: 0.0,    
        },
    ];
    
    // Send the three waypoints to the DA62
    if unsafe { SimConnect_SetDataOnSimObject(HANDLE, DEFINITION_WAYPOINT, 
        DA62_ID, 
        0, 
        waypoint_list_da62.len() as u32, 
        std::mem::size_of::<SIMCONNECT_DATA_WAYPOINT>() as u32, 
        std::ptr::addr_of_mut!(waypoint_list_da62) as *mut c_void
    ) } != 0 {
        eprintln!("Error: SimConnect_SetDataOnSimObject failed!");
    }
    
    // Send the two waypoints to the truck
    if unsafe { SimConnect_SetDataOnSimObject(HANDLE, DEFINITION_WAYPOINT, 
        TRUCK_ID, 
        0, 
        waypoint_list_truck.len() as u32, 
        std::mem::size_of::<SIMCONNECT_DATA_WAYPOINT>() as u32, 
        std::ptr::addr_of_mut!(waypoint_list_truck) as *mut c_void
    ) } != 0 {
        eprintln!("Error: SimConnect_SetDataOnSimObject failed!");
    }
    
    // Send the two waypoints to the giraffe
    if unsafe { SimConnect_SetDataOnSimObject(HANDLE, DEFINITION_WAYPOINT, 
        GIRAFFE_ID, 
        0, 
        waypoint_list_giraffe.len() as u32, 
        std::mem::size_of::<SIMCONNECT_DATA_WAYPOINT>() as u32, 
        std::ptr::addr_of_mut!(waypoint_list_giraffe) as *mut c_void
    ) } != 0 {
        eprintln!("Error: SimConnect_SetDataOnSimObject failed!");
    }
}

fn remove_sim_objects() {
    unsafe {
        if DA62_ID != SIMCONNECT_OBJECT_ID_USER {
            if SimConnect_AIRemoveObject(HANDLE, DA62_ID, REQUEST_REMOVE_DA62) != 0 {
                eprintln!("Error: SimConnect_AIRemoveObject failed!");
            }
        }
        if PITTS_ID != SIMCONNECT_OBJECT_ID_USER {
            if SimConnect_AIRemoveObject(HANDLE, PITTS_ID, REQUEST_REMOVE_PITTS) != 0 {
                eprintln!("Error: SimConnect_AIRemoveObject failed!");
            }
        }
        if GIRAFFE_ID != SIMCONNECT_OBJECT_ID_USER {
            if SimConnect_AIRemoveObject(HANDLE, GIRAFFE_ID, REQUEST_REMOVE_GIRAFFE) != 0 {
                eprintln!("Error: SimConnect_AIRemoveObject failed!");
            }
        }
        if TRUCK_ID != SIMCONNECT_OBJECT_ID_USER {
            if SimConnect_AIRemoveObject(HANDLE, TRUCK_ID, REQUEST_REMOVE_TRUCK) != 0 {
                eprintln!("Error: SimConnect_AIRemoveObject failed!");
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
        SIMCONNECT_RECV_ID_SIMOBJECT_DATA => {
            let obj_data = data as *const SIMCONNECT_RECV_SIMOBJECT_DATA;
            match (*obj_data).dwRequestID {
                REQUEST_ADD_DA62 => {
                }
                _ => {}
            }
        },
        SIMCONNECT_RECV_ID_EVENT => {
            let event = data as *const SIMCONNECT_RECV_EVENT;
            match (*event).uEventID {
                EVENT_Z => {
                    if !OBJECTS_CREATED {
                        setup_sim_objects();
                        OBJECTS_CREATED = true;
                        println!("Press 'x' to change the flight plans");
                    }
                },
                EVENT_X => {
                    if !PLANS_SENT && OBJECTS_CREATED {
                        send_flight_plans();
                        PLANS_SENT = true;
                    }
                },
                EVENT_C => {
                    if OBJECTS_CREATED {
                        remove_sim_objects();
                        OBJECTS_CREATED = false;
                        PLANS_SENT = false;
                    }
                },
                _ => {
                    let event_id = (*event).uEventID;
                    let event_data = (*event).dwData;
                    println!("SIMCONNECT_RECV_EVENT: {:x} {:x} {:x}", event_id, event_data, cb_data);
                }
            }

        },
        SIMCONNECT_RECV_ID_ASSIGNED_OBJECT_ID => {
            let obj_data = data as *const SIMCONNECT_RECV_ASSIGNED_OBJECT_ID;
            let object_id = (*obj_data).dwObjectID;
            match (*obj_data).dwRequestID {
                REQUEST_ADD_DA62 => {
                    DA62_ID = object_id;
                    println!("Created DA62 id = {}", object_id);
                },
                REQUEST_ADD_PITTS => {
                    PITTS_ID = object_id;
                    println!("Created stationary Aviat Pitts Special S2S id = {}", object_id);
                },
                REQUEST_ADD_GIRAFFE => {
                    GIRAFFE_ID = object_id;
                    println!("Created giraffe id = {}", object_id);
                },
                REQUEST_ADD_TRUCK => {
                    TRUCK_ID = object_id;
                    println!("Created truck id = {}", object_id);
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

    // open simconnect
    let name = CString::new("AI Objects and Waypoints").unwrap();
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
        EVENT_C,
    ] {
        if unsafe { SimConnect_MapClientEventToSimEvent(HANDLE, event, std::ptr::null()) } != 0 {
            return Err("Error: SimConnect_MapClientEventToSimEvent(A) failed!");
        }
    }

    // Link the private events to keyboard keys, and ensure the input events are off
    for (input, name, event) in vec![
        (INPUT_ZX, "Z", EVENT_Z),
        (INPUT_ZX, "X", EVENT_X),
        (INPUT_ZX, "C", EVENT_C),
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
    if unsafe { SimConnect_SetInputGroupState(HANDLE, INPUT_ZX, SIMCONNECT_STATE_ON as u32) } != 0 {
        return Err("Error: SimConnect_SetInputGroupState failed!");
    }

    // Sign up for notifications
    for event in vec![
        EVENT_Z,
        EVENT_X,
        EVENT_C,
    ] {
        if unsafe { SimConnect_AddClientEventToNotificationGroup(HANDLE, GROUP_ZX, event, 0) } != 0 {
            return Err("Error: SimConnect_AddClientEventToNotificationGroup failed!");
        }
    }

    // Set up a definition for a waypoint list
    let name = CString::new("AI Waypoint List").unwrap();
    let unit = CString::new("number").unwrap();
    if unsafe { SimConnect_AddToDataDefinition(HANDLE,
        DEFINITION_WAYPOINT, 
        name.as_ptr(), 
        unit.as_ptr(), 
        SIMCONNECT_DATATYPE_WAYPOINT, 
        0.0, 
        SIMCONNECT_UNUSED
    ) } != 0 {
        return Err("Error: SimConnect_AddToDataDefinition failed!");
    }

    println!("Press 'z' to create new aircraft");

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