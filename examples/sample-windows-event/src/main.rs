use std::ffi::{CString, c_void};
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use windows_sys::{
    Win32::Foundation::*, Win32::System::Threading::*,
};

use simconnect_sys::*;

pub enum EventId {
    Unknown,
    FourSeconds
}

impl From<u32> for EventId {
    fn from(v: u32) -> Self {
        match v {
            x if x == EventId::FourSeconds as u32 => EventId::FourSeconds,
            _ => EventId::Unknown,
        }
    }
}

static QUIT: AtomicBool = AtomicBool::new(false);
static TICK: AtomicI32 = AtomicI32::new(0);

unsafe extern "C" fn my_dispatch_proc(
    data: *mut SIMCONNECT_RECV,
    cb_data: DWORD,
    _context: *mut c_void,
) {
    match (*data).dwID as i32 {
        SIMCONNECT_RECV_ID_EVENT => {
            let event = data as *const SIMCONNECT_RECV_EVENT;
            match (*event).uEventID.into() {
                EventId::FourSeconds => {
                    println!("4 second timer: {}", TICK.fetch_add(1, Ordering::SeqCst));
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
            println!("UNKNOWN DATA RECEIVED: data={:p} cb_data={:x}", data, cb_data);
        }
    }
}

fn main() {
    unsafe {

        // register windows event handle
        let event = CreateEventA(std::ptr::null(), 0, 0, std::ptr::null());
        if event == 0 {
            eprintln!("Error: CreateEventA failed!");
            return;
        }

        // open simconnect
        let mut handle = std::ptr::null_mut();
        let name = CString::new("Windows Event").unwrap();
        let hr = SimConnect_Open(
            &mut handle, name.as_ptr(), 
            std::ptr::null_mut(), 
            0, 
            event as *mut c_void, 
            0
        );
        if hr != 0 {
            eprintln!("Error: SimConnect_Open failed!");
            return;
        }

        // subscribe to four second timer
        let name = CString::new("4sec").unwrap();
        let hr = SimConnect_SubscribeToSystemEvent(handle, EventId::FourSeconds as u32, name.as_ptr());
        if hr != 0 {
            eprintln!("Error: SimConnect_SubscribeToSystemEvent failed!");
            return;
        }

        // check for messages only when a windows event has been received
        while QUIT.load(Ordering::SeqCst) == false && WaitForSingleObject(event, INFINITE) == WAIT_OBJECT_0 {
            let hr = SimConnect_CallDispatch(handle, Some(my_dispatch_proc), std::ptr::null_mut());
            if hr != 0 {
                eprintln!("Error: SimConnect_CallDispatch failed!");
                break;
            }
        }

        // close windows event handle
        if CloseHandle(event) == 0 {
            eprintln!("Error: CloseHandle failed!");
        }

        // disconnect from simconnect
        let hr = SimConnect_Close(handle);
        if hr != 0 {
            eprintln!("Error: SimConnect_Close failed!");
            return;
        }
        println!("SimConnect_Close");
    }
}