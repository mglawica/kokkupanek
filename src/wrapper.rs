use std::os::raw::{c_void};
use std::fmt::{self, Write};
use std::mem;
use std::slice;
use std::collections::HashMap;

use failure::Error;
use serde::{Serialize, Deserialize};
use serde::de::{DeserializeOwned};
use serde_json::{from_slice, to_vec};

use logger;
use input;
use timestamp;
use random;

#[derive(Serialize, Debug)]
struct SchedulerResult<'a, S: Serialize, A: Serialize> {
    schedule: S,
    log: &'a String,
    #[serde(default, skip_serializing_if="HashMap::is_empty")]
    actions: HashMap<u64, A>,
}

#[derive(Serialize, Debug)]
struct ErrorResult<'a> {
    schedule: (),
    log: &'a str,
}

#[derive(Debug, Serialize)]
enum ErrorKind {
    Serialize,
    Deserialize,
    Internal,
    #[doc(hidden)]
    __Nonexhaustive,
}

#[derive(Debug, Serialize)]
struct QueryError {
    kind: ErrorKind,
    message: String,
    causes: Option<Vec<String>>,
    backtrace: Option<String>,
}

pub unsafe fn scheduler<F, I, R, E, A>(ptr: *const u8, len: usize, f: F)
    -> *mut c_void
    where F: FnOnce(I) -> Result<(R, HashMap<u64, A>), E>,
          I: DeserializeOwned + input::Input,
          R: Serialize,
          E: fmt::Display,
          A: Serialize,
{
    let input = slice::from_raw_parts(ptr, len);
    let mut out = serde_wrapper(input, f);
    let out_ptr = out.as_mut_ptr();
    mem::forget(out);
    return out_ptr as *mut c_void;
}

pub unsafe fn json_call<'x, F, I, R>(ptr: *const u8, len: usize, f: F)
    -> *mut c_void
    where F: FnOnce(I) -> Result<R, Error>,
          I: Deserialize<'x>,
          R: Serialize,
{
    let input = slice::from_raw_parts(ptr, len);
    let mut out = json_serde_wrapper(input, f);
    let out_ptr = out.as_mut_ptr();
    mem::forget(out);
    return out_ptr as *mut c_void;
}

fn json_serde_wrapper<'x, F, I, R>(data: &'x [u8], f: F) -> Vec<u8>
    where F: FnOnce(I) -> Result<R, Error>,
          I: Deserialize<'x>,
          R: Serialize,
{
    let input = match from_slice(data) {
        Ok(inp) => inp,
        Err(e) => {
            return to_vec(
                &Err::<(), _>(QueryError {
                    kind: ErrorKind::Deserialize,
                    message: e.to_string(),
                    causes: None,
                    backtrace: None,
                })
            ).expect("should serialize standard json");
        }
    };
    let result = match f(input) {
        Ok(v) => Ok(v),
        Err(e) => {
            let mut causes = Vec::new();
            for c in e.causes() {
                causes.push(c.to_string());
            }
            Err(QueryError {
                kind: ErrorKind::Internal,
                message: e.to_string(),
                causes: Some(causes),
                backtrace: Some(format!("{}", e.backtrace())),
            })
        }
    };
    match to_vec(&result) {
        Ok(result) => result,
        Err(e) => {
            return to_vec(
                &Err::<(), _>(QueryError {
                    kind: ErrorKind::Serialize,
                    message: e.to_string(),
                    causes: None,
                    backtrace: None,
                })
            ).expect("should serialize standard json");
        }
    }
}

fn serde_wrapper<'x, F, I, R, E, A>(data: &'x [u8], f: F) -> Vec<u8>
    where F: FnOnce(I) -> Result<(R, HashMap<u64, A>), E>,
          I: DeserializeOwned + input::Input,
          R: Serialize,
          E: fmt::Display,
          A: Serialize,
{
    let input = match from_slice(data) {
        Ok(inp) => inp,
        Err(e) => {
            return to_vec(&ErrorResult {
                schedule: (),
                log: &format!("Error deserialing input: {}", e),
            }).expect("can serialize error")
        }
    };
    let (res, mut log) = logging_wrapper(input, f);
    let sres = match res {
        Ok((schedule, actions)) => to_vec(&SchedulerResult {
            schedule,
            actions,
            log: &log
        }),
        Err(()) => to_vec(&ErrorResult {
            schedule: (),
            log: &log,
        }),
    };
    match sres {
        Ok(result) => result,
        Err(e) => {
            writeln!(&mut log, "\nError serializing output: {}", e).ok();
            return to_vec(&ErrorResult {
                schedule: (),
                log: &log,
            }).expect("can serialize error")
        }
    }
}

fn logging_wrapper<F, I, R, E, A>(input: I, f: F)
    -> (Result<(R, HashMap<u64, A>), ()>, String)
    where F: FnOnce(I) -> Result<(R, HashMap<u64, A>), E>,
          E: fmt::Display,
          I: input::Input,
          A: Serialize,
{
    let logger = logger::SchedulerLogger::context();
    let _timestamp = timestamp::with_timestamp(input.now());
    let _generator = random::with_generator(input.now());
    match f(input) {
        Ok((schedule, actions)) => {
            let mut out = logger.into_inner();
            return (Ok((schedule, actions)), out);
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
