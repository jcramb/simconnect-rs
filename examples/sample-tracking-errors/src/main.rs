use std::ffi::{CString, c_void};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use simconnect_sys::*;

const GROUP: u32 = 1;
const EVENT_BRAKES: u32 = 1000;
const EVENT_BAD: u32 = 2000;

// Holds send ids and identification strings
struct Record {
    call: String,
    send_id: u32
}

static SEND_RECORD: Mutex<Vec<Record>> = Mutex::new(Vec::new());
static QUIT: AtomicBool = AtomicBool::new(false);

// Record the ID along with identification string in SEND_RECORDs
fn add_send_record(handle: *mut c_void, s: impl Into<String>) {
    let mut id: u32 = 0;
    let hr = unsafe { SimConnect_GetLastSentPacketID(handle, &mut id) };
    if hr != 0 {
        eprintln!("Error: SimConnect_GetLastSentPacketID failed!");
        return;
    }
    SEND_RECORD.lock().unwrap().push(Record {
        call: s.into(),
        send_id: id,
    });
}

fn find_send_record(id: u32) -> String {
    match SEND_RECORD.lock().unwrap().iter().find(|v| v.send_id == id) {
        Some(record) => record.call.clone(),
        None => "Send Record not found".into()
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
                EVENT_BRAKES => {
                    let event_data = (*event).dwData;
                    println!("Event brakes: {}", event_data)
                },
                _ => {}
            }
        },
        SIMCONNECT_RECV_ID_EXCEPTION => {

            // display exception
            let e = data as *const SIMCONNECT_RECV_EXCEPTION;
            let exception = (*e).dwException;
            let send_id = (*e).dwSendID;
            let index = (*e).dwIndex;
            eprintln!("\n*** EXCEPTION={} SendID={} Index={} cbData={}", 
                exception, send_id, index, cb_data);

            // locate the bad call and display it
            let s = find_send_record(send_id);
            eprintln!("--> {}\n", s);

            // let the program exit
            QUIT.store(true, Ordering::SeqCst);
        },
        SIMCONNECT_RECV_ID_QUIT => {
            QUIT.store(true, Ordering::Relaxed);
        },
        _ => {}
    }
}

fn main() {
    unsafe {

        // open simconnect
        let mut handle = std::ptr::null_mut();
        let name = CString::new("Tracking Errors").unwrap();
        let hr = SimConnect_Open(
            &mut handle, name.as_ptr(), 
            std::ptr::null_mut(), 
            0, 
            std::ptr::null_mut(), 
            0
        );
        if hr != 0 {
            eprintln!("Error: SimConnect_Open failed!");
            return;
        }
        println!("Connected to Flight Simulator!");

        //
        let name = CString::new("brakes").unwrap();
        let hr = SimConnect_MapClientEventToSimEvent(handle, EVENT_BRAKES, name.as_ptr());
        if hr != 0 {
            eprintln!("Error: SimConnect_MapClientEventToSimEvent failed!");
            return;
        }
        add_send_record(handle, "SimConnect_MapClientEventToSimEvent(handle, EventBrakes, \"brakes\");");
        
        // force an error by using the wrong event
        if SimConnect_AddClientEventToNotificationGroup(handle, GROUP, EVENT_BAD, 0) != 0 {
            eprintln!("Error: SimConnect_AddClientEventToNotificationGroup failed!");
            return;
        }
        add_send_record(handle, "SimConnect_AddClientEventToNotificationGroup(handle, GROUP, EVENT_BAD, 0);");

        //
        if SimConnect_SetNotificationGroupPriority(handle, GROUP, SIMCONNECT_GROUP_PRIORITY_HIGHEST,) != 0 {
            eprintln!("Error: SimConnect_SetNotificationGroupPriority failed!");
            return;
        }
        add_send_record(handle, "SimConnect_AddClientEventToNotificationGroup(handle, GROUP, EVENT_BAD);");

        // run dispatch loop
        while QUIT.load(Ordering::SeqCst) == false {
            let hr = SimConnect_CallDispatch(handle, Some(my_dispatch_proc), std::ptr::null_mut());
            if hr != 0 {
                eprintln!("Error: SimConnect_CallDispatch failed!");
                break;
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
        }

        // disconnect from simconnect
        let hr = SimConnect_Close(handle);
        if hr != 0 {
            eprintln!("Error: SimConnect_Close failed!");
            return;
        }
        println!("Disconnected from Flight Simulator");
    }
}
