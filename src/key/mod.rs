mod hash;
mod implementation;

use std::any::TypeId;
use std::fmt::{Debug, Display};
use std::hash::{Hash, Hasher};

use as_any::AsAny;

use crate::key::hash::DynHash;

pub use crate::key::implementation::KeyImpl;

pub trait Key
where
    Self: Debug + Display + AsAny + DynHash + Send + Sync + 'static,
{
    type Target: Send + 'static
    where
        Self: Sized;

    type Qualifier: Copy + Debug + Eq + Hash + Send + Sync + 'static
    where
        Self: Sized;

    fn target(&self) -> TypeId;

    fn qualifier(&self) -> Self::Qualifier
    where
        Self: Sized;
}

impl PartialEq for dyn Key {
    fn eq(&self, other: &Self) -> bool {
        self.dyn_eq(other.as_any())
    }
}

impl Eq for dyn Key {}

impl Hash for dyn Key {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.dyn_hash(state);
    }
}

impl PartialEq for dyn Key + Send {
    fn eq(&self, other: &Self) -> bool {
        <dyn Key>::eq(self, other)
    }
}

impl Eq for dyn Key + Send {}

impl Hash for dyn Key + Send {
    fn hash<H: Hasher>(&self, state: &mut H) {
        <dyn Key>::hash(self, state);
    }
}

impl PartialEq for dyn Key + Send + Sync {
    fn eq(&self, other: &Self) -> bool {
        <dyn Key>::eq(self, other)
    }
}

impl Eq for dyn Key + Send + Sync {}

impl Hash for dyn Key + Send + Sync {
    fn hash<H: Hasher>(&self, state: &mut H) {
        <dyn Key>::hash(self, state);
    }
}

pub fn of<T: Send + 'static>() -> KeyImpl<T, ()> {
    KeyImpl::new(())
}

pub fn named<T: Send + 'static>(name: &'static str) -> KeyImpl<T, &'static str> {
    KeyImpl::new(name)
}

pub fn qualified<T, Q>(qualifier: Q) -> KeyImpl<T, Q>
where
    T: Send + 'static,
    Q: Copy + Debug + Eq + Hash + Send + Sync + 'static,
{
    KeyImpl::new(qualifier)
}
