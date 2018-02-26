use rand::{XorShiftRng, SeedableRng};
use timestamp::Timestamp;
pub use rand::{Rng};

pub struct GlobalRng;

pub struct Context;



static mut RANDOM: Option<XorShiftRng> = None;


fn get<F: FnOnce(&mut XorShiftRng) -> R, R>(f: F) -> R {
    let mut gen = unsafe {
        RANDOM.as_mut().expect("random generator is initialized")
    };
    return f(&mut gen);
}

impl Rng for GlobalRng {
    fn next_u32(&mut self) -> u32 {
        get(|r| r.next_u32())
    }
}

pub fn with_generator(time: Timestamp) -> Context {
    unsafe {
        RANDOM = Some(XorShiftRng::from_seed([
            (time.0 >> 32) as u32,
            (time.0 & 0xFFFFFFFF) as u32,
            0,
            0,
        ]));
    }
    return Context;
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            RANDOM = None;
        }
    }
}
