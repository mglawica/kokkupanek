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
use kk::graphql;
use kk::input::Schedule as ScheduleTrait;
use kk::input::GenericInput;

mod sources;
mod graph;
mod schedule;
mod projects;
mod services;

pub use schedule::Schedule;


fn main() {
    logger::init();
}

type Input = GenericInput<graphql::Action, Schedule, Value>;

#[no_mangle]
pub extern "C" fn scheduler(ptr: *const u8, len: usize) -> *mut c_void {
    unsafe {
        wrapper::scheduler(ptr, len, |input: Input| -> Result<_, String> {
            let mut schedule = Schedule::from_parents(input.parents);
            info!("Scheduler works!");
            let actions = graphql::execute_actions(input.actions,
                &graph::Context { schedule: RefCell::new(&mut schedule) },
                &graph::Schema::new(&graph::Query, &graph::Mutation));
            return Ok((schedule, actions));
        })
    }
}
