static mut NOW: Option<u64> = None;

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp(u64);

pub struct Context;

impl Timestamp {
    pub fn now() -> Timestamp {
        unsafe {
            match NOW {
                Some(x) => Timestamp(x),
                None => panic!("No timestamp set. \
                                Probably not in scheduler context"),
            }
        }
    }
}

pub fn with_timestamp(val: Timestamp) -> Context {
    unsafe {
        NOW = Some(val.0);
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
