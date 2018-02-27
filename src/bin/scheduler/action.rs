use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::HashMap;

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
    actions: BTreeMap<u64, Action>)
    -> HashMap<u64, Json>
{
    actions.into_iter().map(|(id, action)| {
        let result = match action {
            Action::Graphql(ref act) => {
                graph::execute_action(act, schedule)
            }
            Action::Other(ref data) => {
                warn!("Unknown action {:?}", data);
                json!({"message": "unknown action"})
            }
        };
        (id, result)
    }).collect()
}
