use std::ffi::{CString, c_void};
use std::time::Duration;
use parking_lot::Mutex;
use tokio::sync::mpsc::{Sender, channel};

use simconnect_sys::*;

// Global channel to return data from dispatch callback
static DISPATCH_TX: Mutex<Option<Sender<ExampleData>>> = Mutex::new(None);

// Struct in the format of the data definition
#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct ExampleData {
    pub heading: f64,
    title: [u8; 128]
}

impl ExampleData {
    unsafe fn from_simobject_data(data: *const SIMCONNECT_RECV_SIMOBJECT_DATA) 
        -> Result<Self, &'static str> {
        if (*data).dwDefineCount != 2 {
            return Err("from_simobject_data: define count mismatch")
        }
        let p = std::ptr::addr_of!((*data).dwData) as *const ExampleData;
        Ok(*p)
    }
    unsafe fn title(&self) -> String {
        std::str::from_utf8_unchecked(std::ffi::CStr::from_ptr(
            &self.title as *const u8 as *const i8).to_bytes()
        ).to_string()
    }
}

#[tokio::main]
async fn main() -> Result<(), &'static str> {

    // Create dispatch channel
    let (tx, mut rx) = channel(128);
    *DISPATCH_TX.lock() = Some(tx);

    // Open handle to SimConnect
    let mut handle = std::ptr::null_mut();
    let name = CString::new("Example").unwrap();
    let hr = unsafe { SimConnect_Open(
        &mut handle,
        name.as_ptr(),
        std::ptr::null_mut(),
        0,
        std::ptr::null_mut(),
        0,
    ) };
    if hr != 0 || handle.is_null() {
        return Err("SimConnect_Open failed");
    }
    println!("SimConnect_Open");

    // Add float64 to data definition
    let define_id = 1;
    let datum_name = CString::new("PLANE HEADING DEGREES TRUE").unwrap();
    let datum_unit = CString::new("Degrees").unwrap();
    let hr = unsafe { SimConnect_AddToDataDefinition(
        handle,
        define_id,                      // define_id
        datum_name.as_ptr(),           // datum_name
        datum_unit.as_ptr(),           // datum_unit 
        SIMCONNECT_DATATYPE_FLOAT64,   // data_type
        0.0,                            // f_epsilon
        SIMCONNECT_UNUSED                // datum_id
    ) };
    if hr != 0 {
        return Err("Failed to add PLANE HEADING DEGREES TRUE");
    }
    println!("SimConnect_AddToDataDefinition - PLANE HEADING DEGREES TRUE");

    // Add string to data definition
    // NOTE: datum_unit must be NULL for String/Struct
    let datum_name = CString::new("TITLE").unwrap();
    let hr = unsafe { SimConnect_AddToDataDefinition(
        handle,
        define_id,                       // define_id
        datum_name.as_ptr(),            // datum_name
        std::ptr::null(),               // datum_unit 
        SIMCONNECT_DATATYPE_STRING128,  // data_type
        0.0,                             // f_epsilon
        SIMCONNECT_UNUSED                 // datum_id
    ) };
    if hr != 0 {
        return Err("Failed to add TITLE (aircraft name)");
    }
    println!("SimConnect_AddToDataDefinition - TITLE");

    // submit request for data 
    let request_id = 1000;
    let hr = unsafe { SimConnect_RequestDataOnSimObject(
        handle,
        request_id,
        define_id,
        SIMCONNECT_OBJECT_ID_USER,
        SIMCONNECT_PERIOD_SIM_FRAME,
        SIMCONNECT_DATA_REQUEST_FLAG_CHANGED,
        0,    // origin
        0,  // interval
        0      // limit
    ) };
    if hr != 0 {
        return Err("Failed to request data");
    }
    println!("SimConnect_RequestDataOnSimObject - RequestID {request_id}");

    // Run dispatch loop to drive callbacks
    // We're using tokio, so we want this on its own dedicated thread
    let dispatch_handle = handle as usize;
    std::thread::spawn(move || {
        let handle = dispatch_handle as *mut c_void;
        loop {
            let hr = unsafe { SimConnect_CallDispatch(
                handle,                     
                Some(dispatch_proc),    // cbCallback
                std::ptr::null_mut(),       // pContext
            ) };
            if hr != 0 {
                eprintln!("SimConnect_CallDispatch failed");
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    });

    loop {
        tokio::select! {

            // Exit on Ctrl+C
            _ = tokio::signal::ctrl_c() => {
                println!("Ctrl+C");
                break
            }

            // Process next ExampleData returned from callback
            msg = rx.recv() => {
                match msg {
                    Some(data) =>  
                        println!("RECV_DATA - Title: '{}', Heading: {:.0}°", 
                            unsafe { data.title() }, data.heading),
                    None => break
                }
            }
        }
    }

    // Cleanup SimConnect handle and all definitions
    let hr = unsafe { SimConnect_Close(handle) };
    if hr != 0 {
        return Err("SimConnect_Close failed");
    }
    println!("SimConnect_Close");

    Ok(())
}

// SimConnect Callback Function 
unsafe extern "C" fn dispatch_proc(
    data: *mut SIMCONNECT_RECV,
    _cb_data: DWORD,
    _context: *mut c_void,
) {
    let lock = DISPATCH_TX.lock();
    let tx = lock.as_ref().unwrap().clone();
    drop(lock);
    match (*data).dwID as i32 {
        SIMCONNECT_RECV_ID_NULL => {
            println!("RECV_NULL"); 
            // No messages, shouldn't happen when using CallDispatch
        },
        SIMCONNECT_RECV_ID_OPEN => {
            println!("RECV_OPEN"); 
        },
        SIMCONNECT_RECV_ID_QUIT => {
            println!("RECV_QUIT"); 
        },
        SIMCONNECT_RECV_ID_EXCEPTION => {
            let e = data as *mut SIMCONNECT_RECV_EXCEPTION;
            eprintln!("{}", exception_str((*e).dwException));
        },
        SIMCONNECT_RECV_ID_SIMOBJECT_DATA => {
            match ExampleData::from_simobject_data(
                data as *const SIMCONNECT_RECV_SIMOBJECT_DATA
            ) {
                Ok(data) => if let Err(e) = tx.blocking_send(data) {
                    eprintln!("blocking_send: {e:?}");
                },
                Err(e) => eprintln!("{e:?}")
            }
        },
        SIMCONNECT_RECV_ID_EVENT => {
            println!("RECV_EVENT");
        },
        SIMCONNECT_RECV_ID_EVENT_FRAME => {
            println!("RECV_EVENT_FRAME");
        },
        _ => {
            println!("RECV_UNKNOWN");
        }
    }
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