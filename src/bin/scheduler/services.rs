use std::collections::{BTreeMap};
use std::time::SystemTime;

use kk::lwwset::Mergeable;
use kk::timestamp;
use serde_json::{Value as Json};

use juniper::{Executor, FieldError};

use graph::{Okay, Context};


#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Service {
    #[serde(with="::serde_millis")]
    pub timestamp: SystemTime,
    pub source: String,
    pub config: String,
    pub version: String,
    pub branch: Option<String>,
    pub instances: u32,
    #[serde(default, skip_serializing_if="BTreeMap::is_empty")]
    pub variables: BTreeMap<String, Json>,
}

#[derive(GraphQLInputObject, Debug)]
#[graphql(description="New service structure")]
pub struct NewService {
    slug: String,
    source: String,
    config: String,
    version: String,
    branch: Option<String>,
    instances: i32,
    variables: Option<Vec<Variable>>,
}

#[derive(GraphQLInputObject, Debug)]
#[graphql(description="A variable value for a service")]
pub struct Variable {
    name: String,
    tcp_port: Option<i32>,
    choice: Option<String>,
}

impl Mergeable for Service {
    fn timestamp(&self) -> SystemTime {
        self.timestamp
    }
    fn merge(&mut self, other: Service) {
        if other.timestamp > self.timestamp {
            *self = other;
        }
    }
}

pub fn create_service(executor: &Executor<Context>,
    project: String, group: String, service: NewService)
    -> Result<Okay, FieldError>
{
    let mut schedule = executor.context().schedule.borrow_mut();
    info!("Create service {:?} in {:?}:{:?}", service.slug, project, group);
    let proj = match schedule.projects.get_mut(&project) {
        Some(project) => project,
        None => {
            error!("No project named {:?} found", project);
            return Err(FieldError::new("no project found", project.into()));
        }
    };
    let group = match proj.groups.get_mut(&group) {
        Some(project) => project,
        None => {
            error!("No group named {:?} in project {:?} found",
                group, project);
            return Err(FieldError::new("no group found", group.into()));
        }
    };
    if service.slug == "" {
        return Err(FieldError::new(
            "slug must not be empty", service.slug.into()));
    }
    group.services.insert(service.slug, Service {
        timestamp: timestamp::now(),
        source: service.source,
        config: service.config,
        version: service.version,
        branch: service.branch,
        instances: service.instances as u32,
        variables: BTreeMap::new(), // TODO(tailhook) variables
    });
    return Ok(Okay { ok: true })
}
