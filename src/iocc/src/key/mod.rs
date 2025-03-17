mod implementation;

use std::any::TypeId;
use std::fmt::{Debug, Display};
use std::hash::{Hash, Hasher};

use crate::container::Managed;
use crate::util::any::AsAny;
use crate::util::hash::DynHash;

pub(crate) use crate::key::implementation::KeyImpl;

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

pub fn of<T>() -> impl TypedKey<Target = T, Qualifier = ()>
where
    T: Managed,
{
    KeyImpl::new(())
}

pub fn named<T>(name: &'static str) -> impl TypedKey<Target = T, Qualifier = &'static str>
where
    T: Managed,
{
    KeyImpl::new(name)
}

pub fn qualified<T, Q>(qualifier: Q) -> impl TypedKey<Target = T, Qualifier = Q>
where
    T: Managed,
    Q: Copy + Debug + Eq + Hash + Send + Sync + 'static,
{
    KeyImpl::new(qualifier)
}
