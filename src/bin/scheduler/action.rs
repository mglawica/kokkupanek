use std::cell::RefCell;
use std::collections::BTreeMap;

use graph::GraphqlAction;
use serde_json::{Value as Json};

use graph;
use schedule::Schedule;


#[derive(Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum Action {
    #[allow(non_snake_case)]
    Graphql(GraphqlAction),
    Other(Json),
}


pub fn execute_actions(schedule: &RefCell<Schedule>,
    actions: &BTreeMap<u64, Action>)
{

    for (ref id, ref action) in actions {
        info!("bare action: {}: {:#?}", id, action);
        match *action {
            &Action::Graphql(ref act) => {
                graph::execute_action(act, schedule);
            }
            &Action::Other(ref data) => {
                warn!("Unknown action {:?}", data);
            }
        }
    }
}
