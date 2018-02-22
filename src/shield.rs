use std::default::Default;

use serde_json::Value as Json;


#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Shield<T> {
    Valid(T),
    Invalid(Json),
}

impl<T: Default> Default for Shield<T> {
    fn default() -> Shield<T> {
        Shield::Valid(Default::default())
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
