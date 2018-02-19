use std::cmp::max;
use std::fmt;
use std::collections::BTreeMap;
use std::default::Default;
use std::iter::FromIterator;

use serde_json::Value as Json;

use timestamp::Timestamp;


#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Item<T> {
    Deleted {
        timestamp: Timestamp,
        deleted: bool
    },
    Value(T),
    BadData(Json),
}

pub trait Mergeable {
    fn timestamp(&self) -> Timestamp;
    fn merge(&mut self, other: Self);
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Map<K: Ord, V>(BTreeMap<K, Item<V>>);

impl<K: Ord, V> Map<K, V>
    where K: fmt::Debug,
          V: Mergeable,
{
    pub fn new() -> Map<K, V> {
        Map(BTreeMap::new())
    }
    pub fn insert(&mut self, k: K, v: V) {
        self.0.insert(k, Item::Value(v));
    }
    pub fn get(&mut self, k: &K) -> Option<&V> {
        self.0.get(k).and_then(|x| match x {
            &Item::Value(ref x) => Some(x),
            _ => None,
        })
    }
    pub fn get_mut(&mut self, k: &K) -> Option<&mut V> {
        self.0.get_mut(k).and_then(|x| match x {
            &mut Item::Value(ref mut x) => Some(x),
            _ => None,
        })
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn merge(&mut self, other: Self) {
        use self::Item::*;
        for (key, item) in other.0 {
            match item {
                Deleted { timestamp, deleted } => {
                    let val = match self.0.get(&key) {
                        Some(&Deleted { timestamp: old_ts, ..}) => {
                            Some(Deleted {
                                timestamp: max(timestamp, old_ts),
                                deleted: true,
                            })
                        }
                        Some(&Value(ref val))
                        if val.timestamp() > timestamp
                        => None,
                        Some(&Value(_)) | Some(&BadData(..)) | None
                        => Some(Deleted { timestamp, deleted }),
                    };
                    match val {
                        Some(e) => {
                            self.0.insert(key, e);
                        }
                        None => {}
                    }
                }
                Value(val) => {
                    let val = match self.0.get_mut(&key) {
                        Some(&mut Deleted { timestamp, ..})
                        if timestamp > val.timestamp()
                        => {
                            Some(Deleted { timestamp, deleted: true })
                        }
                        Some(&mut Deleted {..}) => None,
                        Some(&mut Value(ref mut old_val)) => {
                            old_val.merge(val);
                            None
                        }
                        Some(&mut BadData(ref json)) => {
                            warn!("Bad data found for key {:?}: {:?}. \
                                Dropping...", key, json);
                            Some(Value(val))
                        },
                        None => {
                            Some(Value(val))
                        }
                    };
                    match val {
                        Some(e) => {
                            self.0.insert(key, e);
                        }
                        None => {}
                    }
                }
                BadData(data) => {
                    warn!("Bad data found for key {:?}: {:?}. Dropping...",
                        key, data);
                }
            }
        }
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
        Map(iter.into_iter().map(|(k, v)| (k, Item::Value(v))).collect())
    }
}
