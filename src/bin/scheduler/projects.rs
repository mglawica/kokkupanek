use std::time::SystemTime;

use juniper::{Executor, FieldError};

use kk::lwwset::{self, Mergeable};
use kk::timestamp;

use graph::{Okay, Context};
use services::{Service};


#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Project {
    #[serde(with="::serde_millis")]
    pub timestamp: SystemTime,
    pub slug: String,
    pub title: String,
    #[serde(default, skip_serializing_if="lwwset::Map::is_empty")]
    pub groups: lwwset::Map<String, Group>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Group {
    #[serde(with="::serde_millis")]
    pub timestamp: SystemTime,
    pub slug: String,
    pub title: String,
    #[serde(default, skip_serializing_if="lwwset::Map::is_empty")]
    pub services: lwwset::Map<String, Service>,
}

pub fn create_project(executor: &Executor<Context>, slug: String, title: String)
    -> Result<Okay, FieldError>
{
    let mut schedule = executor.context().schedule.borrow_mut();
    info!("Create project {:?} slug {:?}", title, slug);
    schedule.projects.insert(slug.clone(), Project {
        timestamp: timestamp::now(),
        groups: lwwset::Map::new(),
        slug, title,
    });
    return Ok(Okay { ok: true })
}

impl Mergeable for Project {
    fn timestamp(&self) -> SystemTime {
        self.timestamp
    }
    fn merge(&mut self, other: Project) {
        if other.timestamp > self.timestamp {
            let Project { timestamp, slug, title, groups } = other;
            self.timestamp = timestamp;
            self.slug = slug;
            self.title = title;
            self.groups.merge(groups);
        } else {
            self.groups.merge(other.groups);
        }
    }
}

impl Mergeable for Group {
    fn timestamp(&self) -> SystemTime {
        self.timestamp
    }
    fn merge(&mut self, other: Group) {
        if other.timestamp > self.timestamp {
            let Group { timestamp, slug, title, services } = other;
            self.timestamp = timestamp;
            self.slug = slug;
            self.title = title;
            self.services.merge(services);
        } else {
            self.services.merge(other.services);
        }
    }
}

pub fn create_group(executor: &Executor<Context>,
    project: String, slug: String, title: String)
    -> Result<Okay, FieldError>
{
    let mut schedule = executor.context().schedule.borrow_mut();
    info!("Create group {:?} slug {:?} in {:?}", title, slug, project);
    let proj = match schedule.projects.get_mut(&project) {
        Some(project) => project,
        None => {
            error!("No project named {:?} found", slug);
            return Err(FieldError::new("no project found", slug.into()));
        }
    };
    proj.groups.insert(slug.clone(), Group {
        timestamp: timestamp::now(),
        services: lwwset::Map::new(),
        slug, title,
    });
    return Ok(Okay { ok: true })
}
