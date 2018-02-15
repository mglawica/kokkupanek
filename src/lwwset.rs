use std::collections::BTreeMap;
use std::default::Default;
use std::iter::FromIterator;

use serde_json::Value as Json;

use timestamp::Timestamp;


#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Item<T> {
    Deleted {
        timestamp: Timestamp,
        deleted: bool
    },
    Value(T),
    BadData(Json),
}

pub trait Timestamped {
    fn timestamp(&self) -> Timestamp;
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Map<K: Ord, V>(BTreeMap<K, V>);

impl<K: Ord, V> Map<K, V> {
    pub fn new() -> Map<K, V> {
        Map(BTreeMap::new())
    }
    pub fn insert(&mut self, k: K, v: V) {
        self.0.insert(k, v);
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<K: Ord, V> Default for Map<K, V> {
    fn default() -> Map<K, V> {
        Map(BTreeMap::default())
    }
}

impl<K: Ord, V> FromIterator<(K, V)> for Map<K, V> {
    fn from_iter<I>(iter: I) -> Self
        where I: IntoIterator<Item=(K, V)>
    {
        Map(iter.into_iter().collect())
    }
}
