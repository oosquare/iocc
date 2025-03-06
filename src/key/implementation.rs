use std::any;
use std::borrow::Borrow;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

use crate::container::Managed;
use crate::key::{DynKey, TypedKey};

pub struct KeyImpl<T, Q>
where
    T: Managed,
    Q: Copy + Debug + Eq + Hash + Send + Sync + 'static,
{
    qualifier: Q,
    _marker: PhantomData<T>,
}

impl<T, Q> KeyImpl<T, Q>
where
    T: Managed,
    Q: Copy + Debug + Eq + Hash + Send + Sync + 'static,
{
    pub fn new(qualifier: Q) -> Self {
        Self {
            qualifier,
            _marker: PhantomData,
        }
    }
}

impl<T, Q> Clone for KeyImpl<T, Q>
where
    T: Managed,
    Q: Copy + Debug + Eq + Hash + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, Q> Copy for KeyImpl<T, Q>
where
    T: Managed,
    Q: Copy + Debug + Eq + Hash + Send + Sync + 'static,
{
}

// SAFETY: `SimpleKey<T>` doesn't actually contain a `T`.
unsafe impl<T, Q> Sync for KeyImpl<T, Q>
where
    T: Managed,
    Q: Copy + Debug + Eq + Hash + Send + Sync + 'static,
{
}

impl<T, Q> Debug for KeyImpl<T, Q>
where
    T: Managed,
    Q: Copy + Debug + Eq + Hash + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Display::fmt(self, f)
    }
}

impl<T, Q> Display for KeyImpl<T, Q>
where
    T: Managed,
    Q: Copy + Debug + Eq + Hash + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}@{:?}", any::type_name::<T>(), self.qualifier)
    }
}

impl<T, Q> PartialEq for KeyImpl<T, Q>
where
    T: Managed,
    Q: Copy + Debug + Eq + Hash + Send + Sync + 'static,
{
    fn eq(&self, other: &Self) -> bool {
        self.qualifier.eq(&other.qualifier)
    }
}

impl<T, Q> Eq for KeyImpl<T, Q>
where
    T: Managed,
    Q: Copy + Debug + Eq + Hash + Send + Sync + 'static,
{
}

impl<T, Q> Hash for KeyImpl<T, Q>
where
    T: Managed,
    Q: Copy + Debug + Eq + Hash + Send + Sync + 'static,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.qualifier.hash(state);
    }
}

impl<T, Q> Borrow<DynKey> for KeyImpl<T, Q>
where
    T: Managed,
    Q: Copy + Debug + Eq + Hash + Send + Sync + 'static,
{
    fn borrow(&self) -> &DynKey {
        self
    }
}

impl<T, Q> TypedKey for KeyImpl<T, Q>
where
    T: Managed,
    Q: Copy + Debug + Eq + Hash + Send + Sync + 'static,
{
    type Target = T;

    type Qualifier = Q;

    fn qualifier(&self) -> Self::Qualifier
    where
        Self: Sized,
    {
        self.qualifier
    }
}

#[cfg(test)]
mod tests {
    use any::TypeId;

    use crate::key::DynKey;

    use super::*;

    #[test]
    fn key_impl_target_succeeds() {
        let i32_key: Box<DynKey> = Box::new(KeyImpl::<i32, _>::new(()));
        let i32_name1_key: Box<DynKey> = Box::new(KeyImpl::<i32, _>::new("name1"));
        let i32_name2_key: Box<DynKey> = Box::new(KeyImpl::<i32, _>::new("name2"));

        assert_eq!(i32_key.target(), TypeId::of::<i32>());
        assert_eq!(i32_name1_key.target(), TypeId::of::<i32>());
        assert_eq!(i32_name2_key.target(), TypeId::of::<i32>());
    }

    #[test]
    fn key_impl_qualifer_succeeds() {
        let i32_key = KeyImpl::<i32, _>::new(());
        let i32_name1_key = KeyImpl::<i32, _>::new("name1");
        let i32_name2_key = KeyImpl::<i32, _>::new("name2");

        assert_eq!(i32_key.qualifier(), ());
        assert_eq!(i32_name1_key.qualifier(), "name1");
        assert_eq!(i32_name2_key.qualifier(), "name2");
    }

    #[test]
    fn key_impl_eq_succeeds() {
        let i32_key: Box<DynKey> = Box::new(KeyImpl::<i32, _>::new(()));
        let i32_name1_key: Box<DynKey> = Box::new(KeyImpl::<i32, _>::new("name1"));
        let i32_name2_key: Box<DynKey> = Box::new(KeyImpl::<i32, _>::new("name2"));

        assert_ne!(&i32_key, &i32_name1_key);
        assert_ne!(&i32_key, &i32_name2_key);
        assert_ne!(&i32_name1_key, &i32_name2_key);
    }
}
