use std::collections::{BTreeMap};

use kk::lwwset;
use kk::input;

use sources::Source;

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct Node {
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct Schedule {
    #[serde(default, skip_serializing_if="lwwset::Map::is_empty")]
    pub sources: lwwset::Map<String, Source>,
    // Compatibility things
    #[serde(default, skip_serializing_if="BTreeMap::is_empty")]
    pub nodes: BTreeMap<String, Node>,
}

impl input::Schedule for Schedule {
    fn new() -> Self {
        Schedule {
            sources: lwwset::Map::new(),
            nodes: BTreeMap::new(),
        }
    }
    fn merge(&mut self, other: Self) {
        self.sources.merge(other.sources);
        // don't merge nodes: nodes are generated stuff
    }
}
