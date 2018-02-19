use std::collections::{BTreeMap, BTreeSet};
use std::cmp::max;

use juniper::{Executor, FieldError};

use kk::lwwset::{self, Mergeable};
use kk::timestamp::Timestamp;

use graph::{Context, Okay};

type Version = String;


#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Source {
    pub timestamp: Timestamp,
    pub keys: lwwset::Map<String, KeyMeta>,
    #[serde(default, skip_serializing_if="BTreeMap::is_empty")]
    pub deployments: BTreeMap<Version, Deployment>,
    #[serde(default, skip_serializing_if="BTreeMap::is_empty")]
    pub images: BTreeMap<String, Container>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct KeyMeta {
    timestamp: Timestamp,
    comment: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Container {
    daemons: BTreeMap<String, Daemon>,
    commands: BTreeMap<String, Command>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Daemon {
    config: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Command {
    config: String,
}

#[derive(GraphQLInputObject, Debug)]
#[graphql(description="A daemon in new deployment")]
pub struct NewDaemon {
    name: String,
    config: String,
    image: String,
    variables: Option<Vec<NewVariable>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Deployment {
    timestamp: Timestamp,
    branch: Option<String>,
    containers: BTreeSet<String>,
}

#[derive(GraphQLEnum, Debug)]
#[graphql(description="A variable type")]
pub enum VariableType {
    #[graphql(name="TcpPort")]
    TcpPort,
}

#[derive(GraphQLInputObject, Debug)]
#[graphql(description="A process variable")]
pub struct NewVariable {
    name: String,
    #[graphql(name="type")]
    kind: VariableType,
}

#[derive(GraphQLInputObject, Debug)]
#[graphql(description="A daemon in new deployment")]
pub struct NewCommand {
    name: String,
    config: String,
    image: String,
}

#[derive(GraphQLInputObject, Debug)]
#[graphql(description="An ack for new deployment")]
pub struct NewDeployment {
    version: String,
    branch: Option<String>,
    daemons: Vec<NewDaemon>,
    commands: Vec<NewCommand>,
}

#[derive(GraphQLInputObject, Debug)]
#[graphql(description="A ciruela public key")]
pub struct Key {
    key: String,
    comment: String,
}

pub fn create_source(executor: &Executor<Context>,
    slug: String, keys: Vec<Key>)
    -> Result<Okay, FieldError>
{
    let mut schedule = executor.context().schedule.borrow_mut();
    info!("Create source {:?}, keys {:?}", slug, keys);
    schedule.sources.insert(slug, Source {
        timestamp: Timestamp::now(),
        keys: keys.into_iter()
            .map(|key| {
                (key.key, KeyMeta {
                    timestamp: Timestamp::now(),
                    comment: key.comment,
                })
            }).collect(),
        deployments: BTreeMap::new(),
        images: BTreeMap::new(),
    });
    return Ok(Okay { ok: true })
}

pub fn add_deployment(executor: &Executor<Context>,
    slug: String, config: NewDeployment)
    -> Result<Okay, FieldError>
{
    let mut schedule = executor.context().schedule.borrow_mut();
    info!("Add deployment for {:?}: {:?}", slug, config);
    let source = match schedule.sources.get_mut(&slug) {
        Some(source) => source,
        None => {
            error!("No source named {:?} found", slug);
            return Err(FieldError::new("no source found", slug.into()));
        }
    };
    source.deployments.insert(config.version, Deployment {
        timestamp: Timestamp::now(),
        branch: config.branch,
        containers: config.commands.iter().map(|x| x.image.clone())
            .chain(config.daemons.iter().map(|x| x.image.clone()))
            .collect(),
    });
    for cmd in config.commands {
        source.images.entry(cmd.image)
            .or_insert_with(|| Container {
                daemons: BTreeMap::new(),
                commands: BTreeMap::new(),
            })
            .commands.insert(cmd.name, Command {
                config: cmd.config,
            });
    }
    for cmd in config.daemons {
        source.images.entry(cmd.image)
            .or_insert_with(|| Container {
                daemons: BTreeMap::new(),
                commands: BTreeMap::new(),
            })
            .daemons.insert(cmd.name, Daemon {
                config: cmd.config,
            });
    }
    return Ok(Okay { ok: true })
}


impl Mergeable for Source {
    fn timestamp(&self) -> Timestamp {
        self.timestamp
    }
    fn merge(&mut self, other: Source) {
        self.timestamp = max(self.timestamp, other.timestamp);
        self.keys.merge(other.keys);
        self.deployments.extend(other.deployments.into_iter());
        self.images.extend(other.images.into_iter());
    }
}

impl Mergeable for KeyMeta {
    fn timestamp(&self) -> Timestamp {
        self.timestamp
    }
    fn merge(&mut self, other: KeyMeta) {
        if other.timestamp > self.timestamp {
            *self = other;
        }
    }
}
