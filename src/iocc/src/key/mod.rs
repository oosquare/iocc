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

/// An abstract identifier for each object managed by a container.
///
/// A [`Key`] consists of a target type and a qualifier. The target refers to
/// the container-managed object identified by this key, while the qualifier
/// helps distinguish different objects of the same type. Containers map keys
/// to object definitions and managed objects.
///
/// To ensure type-safety, you are not allowed to implement your own [`Key`].
/// Use functions provided by the [`key`] module to create keys instead.
///
/// [`key`]: crate::key
pub trait Key
where
    Self: Debug + Display + AsAny + DynHash + Send + Sync + 'static,
{
    /// Returns a [`TypeId`] of the target.
    fn target_type(&self) -> TypeId;

    /// Returns a [`TypeId`] of the qualifier.
    fn qualifier_type(&self) -> TypeId;

    /// Gets a type-erased qualifier.
    fn dyn_qualifier(&self) -> &dyn Qualifier;

    /// Clones a new [`Key`] from `self`.
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

/// A static variant of [`Key`], which specifies its target type and qualifier
/// type.
///
/// To ensure type-safety, you are not allowed to implement your own
/// [`TypedKey`]. Use functions provided by the [`key`] module to create keys instead.
///
/// [`key`]: crate::key
pub trait TypedKey: Key + Copy + Eq + Hash {
    type Target: Managed;

    type Qualifier: TypedQualifier;

    /// Gets the qualifier by value.
    fn qualifier(&self) -> Self::Qualifier;

    /// Gets the qualifier by reference.
    fn qualifier_ref(&self) -> &Self::Qualifier;
}

/// An abstract value helps distinguish multiple managed objects of the same
/// type.
pub trait Qualifier
where
    Self: Debug + AsAny + DynHash + Send + Sync + 'static,
{
    /// Clones a new [`Qualifier`] from `self`.
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

/// A static variant of [`Qualifier`].
pub trait TypedQualifier: Qualifier + Copy + Eq + Hash {
    /// Upcasts `self` to [`dyn Qualifier`].
    fn upcast_dyn(&self) -> &dyn Qualifier;
}

impl<T> TypedQualifier for T
where
    T: Debug + Copy + Eq + Hash + Send + Sync + 'static,
{
    fn upcast_dyn(&self) -> &dyn Qualifier {
        self
    }
}

impl<T: TypedQualifier> Qualifier for T {
    fn dyn_clone(&self) -> Box<dyn Qualifier> {
        Box::new(*self)
    }
}

/// Creates a key of target type `T` without a qualifier.
pub fn of<T>() -> impl TypedKey<Target = T, Qualifier = ()>
where
    T: Managed,
{
    KeyImpl::new(())
}

/// Creates a key of target type `T`, using a name as its qualifier.
pub fn named<T>(name: &'static str) -> impl TypedKey<Target = T, Qualifier = &'static str>
where
    T: Managed,
{
    KeyImpl::new(name)
}

/// Creates a key of target type `T` with any capable qualifier.
pub fn qualified<T>(qualifier: impl TypedQualifier) -> impl TypedKey<Target = T>
where
    T: Managed,
{
    KeyImpl::new(qualifier)
}
