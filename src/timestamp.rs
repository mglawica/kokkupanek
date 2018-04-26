use std::time::SystemTime;

static mut NOW: Option<SystemTime> = None;

pub struct Context;

pub fn now() -> SystemTime {
    unsafe {
        match NOW {
            Some(x) => x,
            None => panic!("No timestamp set. \
                            Probably not in scheduler context"),
        }
    }
}

pub fn with_timestamp(val: SystemTime) -> Context {
    unsafe {
        NOW = Some(val);
    }
    return Context;
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            NOW = None;
        }
    }
}
