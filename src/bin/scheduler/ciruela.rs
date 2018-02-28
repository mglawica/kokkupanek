use std::collections::HashSet;
use kk::shield::ShieldExt;

use schedule::{Schedule};


pub fn populate(schedule: &mut Schedule, hosts: &HashSet<String>) {
    for name in hosts {
        let role = schedule.nodes.entry(name.to_string())
            .ensure_valid()
            .roles
            .entry("ciruela".into())
            .ensure_valid();
        role.template = Some("ciruela".into());
    }
}
