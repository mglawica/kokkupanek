use std::os::raw::{c_void};
use std::fmt::{self, Write};
use std::mem;
use std::slice;

use serde::{Serialize};
use serde::de::{DeserializeOwned};
use serde_json::Value as Json;
use serde_json::{from_slice, to_vec};

use logger;
use input;
use timestamp;


pub unsafe fn scheduler<F, I, R, E>(ptr: *const u8, len: usize, f: F) -> *mut c_void
    where F: FnOnce(I) -> Result<R, E>,
          E: fmt::Display,
          I: DeserializeOwned + input::Input,
          R: Serialize,
{
    let input = slice::from_raw_parts(ptr, len);
    let mut out = serde_wrapper(input, f);
    let out_ptr = out.as_mut_ptr();
    mem::forget(out);
    return out_ptr as *mut c_void;
}

fn serde_wrapper<'x, F, I, R, E>(data: &'x [u8], f: F) -> Vec<u8>
    where F: FnOnce(I) -> Result<R, E>,
          E: fmt::Display,
          I: DeserializeOwned + input::Input,
          R: Serialize,
{
    let input = match from_slice(data) {
        Ok(inp) => inp,
        Err(e) => {
            return to_vec(
                &(Json::Null, format!("Error deserialing input: {}", e))
            ).expect("can serialize error")
        }
    };
    let (res, mut log) = logging_wrapper(input, f);
    let sres = match res {
        Ok(x) => to_vec(&(x, &log)),
        Err(()) => to_vec(&(Json::Null, &log)),
    };
    match sres {
        Ok(result) => result,
        Err(e) => {
            writeln!(&mut log, "\nError serializing output: {}", e).ok();
            return to_vec(&(Json::Null, log)).expect("can serialize error")
        }
    }
}

fn logging_wrapper<F, I, R, E>(input: I, f: F) -> (Result<R, ()>, String)
    where F: FnOnce(I) -> Result<R, E>,
          E: fmt::Display,
          I: input::Input,
{
    let logger = logger::SchedulerLogger::context();
    let _timestamp = timestamp::with_timestamp(input.now());
    match f(input) {
        Ok(schedule) => {
            let mut out = logger.into_inner();
            return (Ok(schedule), out);
        }
        Err(e) => {
            error!("Error running scheduler: {}", e);
            let mut out = logger.into_inner();
            return (Err(()), out);
        }
    }
}

// In order to work with the memory we expose (de)allocation methods
#[no_mangle]
pub extern "C" fn alloc(size: usize) -> *mut c_void {
    let mut buf = Vec::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    mem::forget(buf);
    return ptr as *mut c_void;
}

#[no_mangle]
pub extern "C" fn dealloc(ptr: *mut c_void) {
    unsafe  {
        let _buf = Vec::from_raw_parts(ptr, 0, 1);
    }
}
