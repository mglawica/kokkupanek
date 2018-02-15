extern crate kokkupanek as kk;

#[macro_use] extern crate log;
#[macro_use] extern crate serde_json;

use std::os::raw::{c_void};

use serde_json::{Value};
use kk::logger;
use kk::wrapper;
use kk::input::GenericInput;



fn main() {
    logger::init();
}

#[no_mangle]
pub extern "C" fn scheduler(ptr: *const u8, len: usize) -> *mut c_void {
    unsafe {
        wrapper::scheduler(ptr, len,
            |input: GenericInput<(), (), ()>| -> Result<Value, String> {
                info!("Scheduler works!");
                let schedule = json!({});
                return Ok(schedule);
            })
    }
}
