extern crate kokkupanek as kk;

extern crate serde_json;
#[macro_use] extern crate log;
#[macro_use] extern crate juniper;
#[macro_use] extern crate serde_derive;

use std::os::raw::{c_void};
use std::cell::RefCell;
use std::collections::HashSet;

use serde_json::{Value};
use kk::logger;
use kk::wrapper;
use kk::graphql;
use kk::input::Schedule as ScheduleTrait;
use kk::input::GenericInput;

mod ciruela;
mod distribute;
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
            let cur_nodes = input.hosts()
                .map(|x| x.to_string()).collect::<HashSet<_>>();
            let mut schedule = Schedule::from_parents(input.parents);

            info!("Scheduler works!");

            let actions = graphql::execute_actions(input.actions,
                &graph::Context { schedule: RefCell::new(&mut schedule) },
                &graph::Schema::new(&graph::Query, &graph::Mutation));

            let new_nodes = distribute::distribute(&mut schedule, &cur_nodes);
            schedule.nodes = new_nodes;
            ciruela::populate(&mut schedule, &cur_nodes);

            return Ok((schedule, actions));
        })
    }
}
