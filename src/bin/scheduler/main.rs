extern crate kokkupanek as kk;

extern crate serde_json;
#[macro_use] extern crate log;
#[macro_use] extern crate juniper;
#[macro_use] extern crate serde_derive;

use std::os::raw::{c_void};
use std::cell::RefCell;

use serde_json::{Value};
use kk::logger;
use kk::wrapper;
use kk::input::Schedule as ScheduleTrait;
use kk::input::GenericInput;

mod action;
mod sources;
mod graph;
mod schedule;
mod projects;
mod services;

pub use schedule::Schedule;


fn main() {
    logger::init();
}

type Input = GenericInput<action::Action, Schedule, Value>;

#[no_mangle]
pub extern "C" fn scheduler(ptr: *const u8, len: usize) -> *mut c_void {
    unsafe {
        wrapper::scheduler(ptr, len,
            |input: Input| -> Result<Schedule, String> {
                let schedule = Schedule::from_parents(input.parents);
                let cell = RefCell::new(schedule);
                info!("Scheduler works!");
                action::execute_actions(&cell, &input.actions);
                return Ok(cell.into_inner());
            })
    }
}
