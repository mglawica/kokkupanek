use std::collections::{BTreeMap, HashMap};

use kk::lwwset;
use kk::input;
use kk::shield::Shield;

use sources::Source;
use projects::Project;

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct NodeService {
    pub image: String,
    pub config: String,
    pub instances: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct NodeRole {
    pub template: Option<String>,
    pub version: String,
    #[serde(default, skip_serializing_if="HashMap::is_empty")]
    pub services: HashMap<String, Shield<NodeService>>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct Node {
    pub roles: HashMap<String, Shield<NodeRole>>
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct Schedule {
    #[serde(default, skip_serializing_if="lwwset::Map::is_empty")]
    pub sources: lwwset::Map<String, Source>,
    #[serde(default, skip_serializing_if="lwwset::Map::is_empty")]
    pub projects: lwwset::Map<String, Project>,
    // Compatibility things
    #[serde(default, skip_serializing_if="BTreeMap::is_empty")]
    pub nodes: BTreeMap<String, Shield<Node>>,
}

impl input::Schedule for Schedule {
    fn new() -> Self {
        Schedule {
            sources: lwwset::Map::new(),
            projects: lwwset::Map::new(),
            nodes: BTreeMap::new(),
        }
    }
    fn merge(&mut self, other: Self) {
        let Schedule { sources, projects,
            // don't merge nodes: nodes are generated stuff
            nodes: _
        } = other;
        self.sources.merge(sources);
        self.projects.merge(projects);
    }
}
