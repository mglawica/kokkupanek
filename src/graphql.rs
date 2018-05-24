use std::collections::{HashMap, BTreeMap};

use juniper::{GraphQLType, InputValue, RootNode, execute};
use serde_json::{Value as Json, to_value, from_value};


#[derive(Deserialize, Clone, Debug)]
pub struct GraphqlAction {
    query: String,
    #[serde(default, rename="operationName")]
    operation_name: Option<String>,
    #[serde(default)]
    variables: HashMap<String, InputValue>,
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

pub fn execute_actions<'a, C, Q, M>(actions: BTreeMap<u64, Json>,
    context: &C, root_node: &RootNode<'a, Q, M>)
    -> HashMap<u64, Json>
    where Q: GraphQLType<Context=C>,
          M: GraphQLType<Context=C>,
{
    actions.into_iter().map(|(id, action)| {
        let result = match from_value(action) {
            Ok(act) => {
                execute_action(&act, context, root_node)
            }
            Err(ref error) => {
                warn!("Unknown action {}: {}", id, error);
                json!({"message": format!("bad action: {}", error)})
            }
        };
        (id, result)
    }).collect()
}
