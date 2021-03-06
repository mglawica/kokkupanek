use std::cell::RefCell;

use juniper::{RootNode, FieldError};

use schedule::Schedule;
use sources;
use projects;
use services;


#[derive(Debug)]
pub struct Query;

#[derive(Debug)]
pub struct Mutation;

pub type Schema<'a> = RootNode<'a, &'a Query, &'a Mutation>;

#[derive(Debug)]
pub struct Context<'a> {
    pub schedule: RefCell<&'a mut Schedule>,
}

#[derive(GraphQLObject, Debug)]
#[graphql(description="A generic successful response")]
pub struct Okay {
    pub ok: bool,
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

    field create_project(&executor, slug: String, title: String)
        -> Result<Okay, FieldError>
    {
        projects::create_project(executor, slug, title)
    }
    field create_group(&executor, project: String, slug: String, title: String)
        -> Result<Okay, FieldError>
    {
        projects::create_group(executor, project, slug, title)
    }
    field create_service(&executor, project: String, group: String,
        service: services::NewService)
        -> Result<Okay, FieldError>
    {
        services::create_service(executor, project, group, service)
    }


});
