use std::collections::{HashMap, BTreeMap};

use juniper::{GraphQLType, InputValue, RootNode, execute};
use serde_json::{Value as Json, to_value};


#[derive(Deserialize, Clone, Debug)]
pub struct GraphqlAction {
    query: String,
    #[serde(default, rename="operationName")]
    operation_name: Option<String>,
    #[serde(default)]
    variables: HashMap<String, InputValue>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum Action {
    #[allow(non_snake_case)]
    Graphql(GraphqlAction),
    Other(Json),
}


fn execute_action<'a, C, Q, M>(action: &GraphqlAction, context: &C,
    root_node: &RootNode<'a, Q, M>)
    -> Json
    where Q: GraphQLType<Context=C>,
          M: GraphQLType<Context=C>,
{
    let result = execute(&action.query,
        action.operation_name.as_ref().map(|x| &x[..]),
        root_node, &action.variables, context);
    match result {
        Ok((data, errors)) => {
            json!({"data": data, "errors": errors})
        }
        Err(err) => {
            to_value(&err).expect("can serialize juniper's error")
        }
    }
}

pub fn execute_actions<'a, C, Q, M>(actions: BTreeMap<u64, Action>,
    context: &C, root_node: &RootNode<'a, Q, M>)
    -> HashMap<u64, Json>
    where Q: GraphQLType<Context=C>,
          M: GraphQLType<Context=C>,
{
    actions.into_iter().map(|(id, action)| {
        let result = match action {
            Action::Graphql(ref act) => {
                execute_action(act, context, root_node)
            }
            Action::Other(ref data) => {
                warn!("Unknown action {:?}", data);
                json!({"message": "unknown action"})
            }
        };
        (id, result)
    }).collect()
}
