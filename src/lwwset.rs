use std::borrow::Borrow;
use std::cmp::{max, Ord};
use std::collections::{BTreeMap, btree_map};
use std::default::Default;
use std::fmt;
use std::iter::FromIterator;
use std::mem;
use std::time::SystemTime;

use timestamp;

use serde_json::Value as Json;


#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Item<T> {
    Deleted {
        #[serde(with="::serde_millis")]
        timestamp: SystemTime,
        deleted: bool
    },
    Value(T),
    BadData(Json),
}

pub trait Mergeable {
    fn timestamp(&self) -> SystemTime;
    fn merge(&mut self, other: Self);
}

pub struct Iter<'a, K:'a , V:'a>(btree_map::Iter<'a, K, Item<V>>);

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
    pub fn get<Q>(&self, k: &Q) -> Option<&V>
        where K: Borrow<Q>,
              Q: Ord + ?Sized,
    {
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
    pub fn remove(&mut self, k: &K) -> Option<V> {
        if let Some(e) = self.0.get_mut(k) {
            let v = mem::replace(e, Item::Deleted {
                timestamp: timestamp::now(),
                deleted: true,
            });
            match v {
                Item::Value(x) => Some(x),
                _ => None,
            }
        } else {
            None
        }
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
    pub fn iter(&self) -> Iter<K, V> {
        self.into_iter()
    }
}

impl<K: Ord, V> Default for Map<K, V> {
    fn default() -> Map<K, V> {
        Map(BTreeMap::default())
    }
}

impl<'a, K: Ord +'a, V: 'a> IntoIterator for &'a Map<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;
    fn into_iter(self) -> Iter<'a, K, V> {
        Iter(self.0.iter())
    }
}

impl<'a, K: Ord + 'a, V: 'a> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<(&'a K, &'a V)> {
        loop {
            match self.0.next() {
                None => break None,
                Some((k, &Item::Value(ref v))) => break Some((k, v)),
                Some((_, &Item::Deleted {..})) => continue,
                Some((_, &Item::BadData(..))) => continue,
            }
        }
    }
}

impl<K: Ord, V> FromIterator<(K, V)> for Map<K, V> {
    fn from_iter<I>(iter: I) -> Self
        where I: IntoIterator<Item=(K, V)>
    {
        Map(iter.into_iter().map(|(k, v)| (k, Item::Value(v))).collect())
    }
}
