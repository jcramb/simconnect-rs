use std::ffi::{CString, c_void};
use std::sync::atomic::{AtomicBool, Ordering};

use simconnect_sys::*;

const GROUP_0: u32 = 1;
const INPUT_Z: u32 = 1;
const INPUT_SLIDER: u32 = 2;
const INPUT_XAXIS: u32 = 3;
const INPUT_YAXIS: u32 = 4;
const INPUT_RZAXIS: u32 = 5;
const INPUT_HAT: u32 = 6;
const EVENT_Z: u32 = 1;
const EVENT_SLIDER: u32 = 2;
const EVENT_XAXIS: u32 = 3;
const EVENT_YAXIS: u32 = 4;
const EVENT_RZAXIS: u32 = 5;
const EVENT_HAT: u32 = 6;

static mut HANDLE: *mut c_void = std::ptr::null_mut(); 
static mut CURRENT: i64 = 0;
static QUIT: AtomicBool = AtomicBool::new(false);

unsafe extern "C" fn my_dispatch_proc(
    data: *mut SIMCONNECT_RECV,
    cb_data: DWORD,
    _context: *mut c_void,
) {
    match (*data).dwID as i32 {
        SIMCONNECT_RECV_ID_EVENT => {
            let event = data as *const SIMCONNECT_RECV_EVENT;
            let data = (*event).dwData;
            match (*event).uEventID {
                EVENT_SLIDER => println!("Slider value: {}", data),
                EVENT_XAXIS => println!("X Axis value: {}", data),
                EVENT_YAXIS => println!("Y Axis value: {}", data),
                EVENT_RZAXIS => println!("Rotate Z Axis value: {}", data),
                EVENT_HAT => println!("Hat value: {}", data),
                EVENT_Z => {
                    CURRENT += 1;
                    if CURRENT == 6 {
                        CURRENT = 1;
                    }
                    match CURRENT {
                        1 => {
                            println!("SLIDER is active");
                            let _ = unsafe { SimConnect_SetInputGroupState(HANDLE, INPUT_SLIDER, SIMCONNECT_STATE_ON as u32) };
                            let _ = unsafe { SimConnect_SetInputGroupState(HANDLE, INPUT_XAXIS, SIMCONNECT_STATE_OFF as u32) };
                            let _ = unsafe { SimConnect_SetInputGroupState(HANDLE, INPUT_YAXIS, SIMCONNECT_STATE_OFF as u32) };
                            let _ = unsafe { SimConnect_SetInputGroupState(HANDLE, INPUT_RZAXIS, SIMCONNECT_STATE_OFF as u32) };
                            let _ = unsafe { SimConnect_SetInputGroupState(HANDLE, INPUT_HAT, SIMCONNECT_STATE_OFF as u32) };
                        },
                        2 => {
                            println!("X AXIS is active");
                            let _ = unsafe { SimConnect_SetInputGroupState(HANDLE, INPUT_SLIDER, SIMCONNECT_STATE_OFF as u32) };
                            let _ = unsafe { SimConnect_SetInputGroupState(HANDLE, INPUT_XAXIS, SIMCONNECT_STATE_ON as u32) };
                            let _ = unsafe { SimConnect_SetInputGroupState(HANDLE, INPUT_YAXIS, SIMCONNECT_STATE_OFF as u32) };
                            let _ = unsafe { SimConnect_SetInputGroupState(HANDLE, INPUT_RZAXIS, SIMCONNECT_STATE_OFF as u32) };
                            let _ = unsafe { SimConnect_SetInputGroupState(HANDLE, INPUT_HAT, SIMCONNECT_STATE_OFF as u32) };
                        },
                        3 => {
                            println!("Y AXIS is active");
                            let _ = unsafe { SimConnect_SetInputGroupState(HANDLE, INPUT_SLIDER, SIMCONNECT_STATE_OFF as u32) };
                            let _ = unsafe { SimConnect_SetInputGroupState(HANDLE, INPUT_XAXIS, SIMCONNECT_STATE_OFF as u32) };
                            let _ = unsafe { SimConnect_SetInputGroupState(HANDLE, INPUT_YAXIS, SIMCONNECT_STATE_ON as u32) };
                            let _ = unsafe { SimConnect_SetInputGroupState(HANDLE, INPUT_RZAXIS, SIMCONNECT_STATE_OFF as u32) };
                            let _ = unsafe { SimConnect_SetInputGroupState(HANDLE, INPUT_HAT, SIMCONNECT_STATE_OFF as u32) };
                        },
                        4 => {
                            println!("Z ROTATION is active");
                            let _ = unsafe { SimConnect_SetInputGroupState(HANDLE, INPUT_SLIDER, SIMCONNECT_STATE_OFF as u32) };
                            let _ = unsafe { SimConnect_SetInputGroupState(HANDLE, INPUT_XAXIS, SIMCONNECT_STATE_OFF as u32) };
                            let _ = unsafe { SimConnect_SetInputGroupState(HANDLE, INPUT_YAXIS, SIMCONNECT_STATE_OFF as u32) };
                            let _ = unsafe { SimConnect_SetInputGroupState(HANDLE, INPUT_RZAXIS, SIMCONNECT_STATE_ON as u32) };
                            let _ = unsafe { SimConnect_SetInputGroupState(HANDLE, INPUT_HAT, SIMCONNECT_STATE_OFF as u32) };
                        },
                        5 => {
                            println!("HAT is active");
                            let _ = unsafe { SimConnect_SetInputGroupState(HANDLE, INPUT_SLIDER, SIMCONNECT_STATE_OFF as u32) };
                            let _ = unsafe { SimConnect_SetInputGroupState(HANDLE, INPUT_XAXIS, SIMCONNECT_STATE_OFF as u32) };
                            let _ = unsafe { SimConnect_SetInputGroupState(HANDLE, INPUT_YAXIS, SIMCONNECT_STATE_OFF as u32) };
                            let _ = unsafe { SimConnect_SetInputGroupState(HANDLE, INPUT_RZAXIS, SIMCONNECT_STATE_OFF as u32) };
                            let _ = unsafe { SimConnect_SetInputGroupState(HANDLE, INPUT_HAT, SIMCONNECT_STATE_ON as u32) };
                        },
                        _ => {}
                    }
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
        _ => {}
    }
}

fn main() -> Result<(), &'static str> {

    // open simconnect
    let name = CString::new("Joystick Input").unwrap();
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

    // Set up private events and add them to a notification group
    for event in vec![
        EVENT_Z,
        EVENT_SLIDER,
        EVENT_XAXIS,
        EVENT_YAXIS,
        EVENT_RZAXIS,
        EVENT_HAT
    ] {
        if unsafe { SimConnect_MapClientEventToSimEvent(HANDLE, event, std::ptr::null()) } != 0 {
            return Err("Error: SimConnect_MapClientEventToSimEvent failed!");
        }
        if unsafe { SimConnect_AddClientEventToNotificationGroup(HANDLE, GROUP_0, event, 0) } != 0 {
            return Err("Error: SimConnect_MapClientEventToSimEvent failed!");
        }
    }

    // Set a high priority for the group
    if unsafe { SimConnect_SetNotificationGroupPriority(HANDLE, GROUP_0, SIMCONNECT_GROUP_PRIORITY_HIGHEST) } != 0 {
        return Err("Error: SimConnect_SetNotificationGroupPriority failed!");
    }

    // Map input events to the private client events
    for (input, name, event) in vec![
        (INPUT_Z, "z", EVENT_Z),
        (INPUT_SLIDER, "joystick:0:slider", EVENT_SLIDER),
        (INPUT_XAXIS, "joystick:0:XAxis", EVENT_XAXIS),
        (INPUT_YAXIS, "joystick:0:YAxis", EVENT_YAXIS),
        (INPUT_RZAXIS, "joystick:0:RzAxis", EVENT_RZAXIS),
        (INPUT_HAT, "joystick:0:POV", EVENT_HAT),
    ] {
        let name = CString::new(name).unwrap();
        if unsafe { SimConnect_MapInputEventToClientEvent(HANDLE, 
            input, 
            name.as_ptr(), 
            event,
            0, SIMCONNECT_UNUSED, 0, 0
        ) } != 0 {
            return Err("Error: SimConnect_MapInputEventToClientEvent(Z) failed!");
        }
    }

    // Turn on the Z key
    if unsafe { SimConnect_SetInputGroupState(HANDLE, INPUT_Z, SIMCONNECT_STATE_ON as u32) } != 0 {
        return Err("Error: SimConnect_SetInputGroupState failed!");
    }
    if unsafe { SimConnect_SetInputGroupPriority(HANDLE, INPUT_Z, SIMCONNECT_GROUP_PRIORITY_HIGHEST) } != 0 {
        return Err("Error: SimConnect_SetInputGroupPriority failed!");
    }

    // Turn all the joystick events off
    for input in vec![
        INPUT_Z,
        INPUT_SLIDER,
        INPUT_XAXIS,
        INPUT_YAXIS,
        INPUT_RZAXIS,
        INPUT_HAT,
    ] {
        if unsafe { SimConnect_SetInputGroupState(HANDLE, input, SIMCONNECT_STATE_OFF as u32) } != 0 {
            return Err("Error: SimConnect_SetInputGroupState(Z) failed!");
        }
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