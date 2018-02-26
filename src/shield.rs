use std::cmp::Ord;
use std::collections::{hash_map, btree_map};
use std::default::Default;

use serde_json::Value as Json;


#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Shield<T> {
    Valid(T),
    Invalid(Json),
}

pub trait ShieldExt<'a, T> {
    fn ensure_valid(self) -> &'a mut T;
}

impl<T: Default> Default for Shield<T> {
    fn default() -> Shield<T> {
        Shield::Valid(Default::default())
    }
}
impl<T> From<T> for Shield<T> {
    fn from(val: T) -> Shield<T> {
        Shield::Valid(val)
    }
}

impl<T: Default> Shield<T> {
    /// Replaces invalid reference with Default::default
    pub fn ensure_valid(&mut self) -> &mut T {
        match *self {
            Shield::Valid(ref mut x) => return x,
            Shield::Invalid(_) => {}
        }
        *self = Shield::Valid(T::default());
        match *self {
            Shield::Valid(ref mut x) => return x,
            Shield::Invalid(_) => unreachable!(),
        }
    }
    /// Returns reference to T if value is valid
    ///
    /// This is useful to use with `map()` and other `Option<T>` combinators
    pub fn ok(&self) -> Option<&T> {
        match *self {
            Shield::Valid(ref x) => Some(x),
            Shield::Invalid(_) => None,
        }
    }
}

impl<'a, K: 'a, V: 'a> ShieldExt<'a, V> for hash_map::Entry<'a, K, Shield<V>>
    where V: Default
{
    fn ensure_valid(self) -> &'a mut V {
        self.or_insert_with(Default::default).ensure_valid()
    }
}

impl<'a, K: 'a, V: 'a> ShieldExt<'a, V> for btree_map::Entry<'a, K, Shield<V>>
    where K: Ord,
          V: Default,
{
    fn ensure_valid(self) -> &'a mut V {
        self.or_insert_with(Default::default).ensure_valid()
    }
}
