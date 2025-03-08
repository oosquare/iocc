mod hash;
mod implementation;

use std::any::TypeId;
use std::fmt::{Debug, Display};
use std::hash::{Hash, Hasher};

use crate::container::Managed;
use crate::key::hash::DynHash;
use crate::util::any::AsAny;

pub use crate::key::implementation::KeyImpl;

pub trait Key
where
    Self: Debug + Display + AsAny + DynHash + Send + Sync + 'static,
{
    fn target(&self) -> TypeId;

    fn dyn_clone(&self) -> Box<dyn Key>;
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

impl<T: TypedKey> Key for T {
    fn target(&self) -> TypeId {
        TypeId::of::<T::Target>()
    }

    fn dyn_clone(&self) -> Box<dyn Key> {
        Box::new(*self)
    }
}

pub trait TypedKey: Key + Copy + Eq + Hash {
    type Target: Managed;

    type Qualifier: Copy + Debug + Eq + Hash + Send + Sync + 'static;

    fn qualifier(&self) -> Self::Qualifier;
}

pub fn of<T: Managed>() -> KeyImpl<T, ()> {
    KeyImpl::new(())
}

pub fn named<T: Managed>(name: &'static str) -> KeyImpl<T, &'static str> {
    KeyImpl::new(name)
}

pub fn qualified<T, Q>(qualifier: Q) -> KeyImpl<T, Q>
where
    T: Managed,
    Q: Copy + Debug + Eq + Hash + Send + Sync + 'static,
{
    KeyImpl::new(qualifier)
}
