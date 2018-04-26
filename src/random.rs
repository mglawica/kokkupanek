use std::time::{SystemTime, UNIX_EPOCH};

use rand::{XorShiftRng, SeedableRng};
pub use rand::{Rng, Rand, seq};

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

pub fn with_generator(time: SystemTime) -> Context {
    let seed = time.duration_since(UNIX_EPOCH).expect("valid time");
    unsafe {
        RANDOM = Some(XorShiftRng::from_seed([
            (seed.as_secs() >> 32) as u32,
            (seed.as_secs() & 0xFFFFFFFF) as u32,
            seed.subsec_nanos(),
            0,
        ]));
    }
    return Context;
}

/// Returns current global random number generator
pub fn global_rng() -> GlobalRng {
    GlobalRng
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            RANDOM = None;
        }
    }
}
