use std::cell::RefCell;
use std::collections::HashMap;

use juniper::{RootNode, InputValue, FieldError, execute};

use schedule::Schedule;
use sources;

#[derive(Debug)]
pub struct Query;

#[derive(Debug)]
pub struct Mutation;

pub type Schema<'a> = RootNode<'a, &'a Query, &'a Mutation>;

#[derive(Debug)]
pub struct Context<'a> {
    pub schedule: &'a RefCell<Schedule>,
}

#[derive(GraphQLObject, Debug)]
#[graphql(description="A generic successful response")]
pub struct Okay {
    pub ok: bool,
}

#[derive(Deserialize, Clone, Debug)]
pub struct GraphqlAction {
    query: String,
    #[serde(default, rename="operationName")]
    operation_name: Option<String>,
    #[serde(default)]
    variables: HashMap<String, InputValue>,
}

pub fn execute_action(action: &GraphqlAction, schedule: &RefCell<Schedule>) {
    let r = execute(&action.query,
        action.operation_name.as_ref().map(|x| &x[..]),
        &Schema::new(&Query, &Mutation),
        &action.variables,
        &Context { schedule },
    );
    info!("Result {:#?}", r);
}


graphql_object!(<'a> &'a Query: Context<'a> as "Query" |&self| {
});

graphql_object!(<'a> &'a Mutation: Context<'a> as "Mutation" |&self| {
    field create_source(&executor, slug: String, keys: Vec<sources::Key>)
        -> Result<Okay, FieldError>
    {
        sources::create_source(executor, slug, keys)
    }
    field add_deployment(&executor, slug: String,
        deploy: sources::NewDeployment)
        -> Result<Okay, FieldError>
    {
        sources::add_deployment(executor, slug, deploy)
    }
});
