mod implementation;
mod pattern;

use std::any::TypeId;
use std::fmt::{Debug, Display};
use std::hash::{Hash, Hasher};

use crate::container::Managed;
use crate::util::any::AsAny;
use crate::util::hash::DynHash;

pub(crate) use crate::key::implementation::KeyImpl;
pub use crate::key::pattern::{AnyPattern, KeyTypePattern, Pattern};

pub trait Key
where
    Self: Debug + Display + AsAny + DynHash + Send + Sync + 'static,
{
    fn target_type(&self) -> TypeId;

    fn qualifier_type(&self) -> TypeId;

    fn dyn_qualifier(&self) -> &dyn Qualifier;

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
    fn target_type(&self) -> TypeId {
        TypeId::of::<T::Target>()
    }

    fn qualifier_type(&self) -> TypeId {
        TypeId::of::<T::Qualifier>()
    }

    fn dyn_qualifier(&self) -> &dyn Qualifier {
        self.qualifier_ref()
    }

    fn dyn_clone(&self) -> Box<dyn Key> {
        Box::new(*self)
    }
}

pub trait TypedKey: Key + Copy + Eq + Hash {
    type Target: Managed;

    type Qualifier: TypedQualifier;

    fn qualifier(&self) -> Self::Qualifier;

    fn qualifier_ref(&self) -> &Self::Qualifier;
}

pub trait Qualifier
where
    Self: Debug + AsAny + DynHash + Send + Sync + 'static,
{
    fn dyn_clone(&self) -> Box<dyn Qualifier>;
}

impl PartialEq for dyn Qualifier {
    fn eq(&self, other: &Self) -> bool {
        self.dyn_eq(other.as_any())
    }
}

impl Eq for dyn Qualifier {}

impl Hash for dyn Qualifier {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.dyn_hash(state);
    }
}

pub trait TypedQualifier: Qualifier + Copy + Eq + Hash {
    /// Upcasts `self` to [`dyn Qualifier`].
    fn upcast_dyn(&self) -> &dyn Qualifier;
}

impl<T> TypedQualifier for T where T: Debug + Copy + Eq + Hash + Send + Sync + 'static {
    fn upcast_dyn(&self) -> &dyn Qualifier {
        self
    }
}

impl<T: TypedQualifier> Qualifier for T {
    fn dyn_clone(&self) -> Box<dyn Qualifier> {
        Box::new(*self)
    }
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

pub fn qualified<T>(qualifier: impl TypedQualifier) -> impl TypedKey<Target = T>
where
    T: Managed,
{
    KeyImpl::new(qualifier)
}
