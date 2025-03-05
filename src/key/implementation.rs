use std::any::{self, TypeId};
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

use crate::key::Key;

#[derive(Clone, Copy)]
pub struct KeyImpl<T, Q>
where
    T: Send + 'static,
    Q: Copy + Debug + Eq + Hash + Send + Sync + 'static,
{
    qualifier: Q,
    _marker: PhantomData<T>,
}

impl<T, Q> KeyImpl<T, Q>
where
    T: Send + 'static,
    Q: Copy + Debug + Eq + Hash + Send + Sync + 'static,
{
    pub fn new(qualifier: Q) -> Self {
        Self {
            qualifier,
            _marker: PhantomData,
        }
    }
}

// SAFETY: `SimpleKey<T>` doesn't actually contain a `T`.
unsafe impl<T, Q> Sync for KeyImpl<T, Q>
where
    T: Send + 'static,
    Q: Copy + Debug + Eq + Hash + Send + Sync + 'static,
{
}

impl<T, Q> Debug for KeyImpl<T, Q>
where
    T: Send + 'static,
    Q: Copy + Debug + Eq + Hash + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Display::fmt(self, f)
    }
}

impl<T, Q> Display for KeyImpl<T, Q>
where
    T: Send + 'static,
    Q: Copy + Debug + Eq + Hash + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}@{:?}", any::type_name::<T>(), self.qualifier)
    }
}

impl<T, Q> PartialEq for KeyImpl<T, Q>
where
    T: Send + 'static,
    Q: Copy + Debug + Eq + Hash + Send + Sync + 'static,
{
    fn eq(&self, other: &Self) -> bool {
        self.qualifier.eq(&other.qualifier)
    }
}

impl<T, Q> Eq for KeyImpl<T, Q>
where
    T: Send + 'static,
    Q: Copy + Debug + Eq + Hash + Send + Sync + 'static,
{
}

impl<T, Q> Hash for KeyImpl<T, Q>
where
    T: Send + 'static,
    Q: Copy + Debug + Eq + Hash + Send + Sync + 'static,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.qualifier.hash(state);
    }
}

impl<T, Q> Key for KeyImpl<T, Q>
where
    T: Send + 'static,
    Q: Copy + Debug + Eq + Hash + Send + Sync + 'static,
{
    type Target
        = T
    where
        Self: Sized;

    type Qualifier
        = Q
    where
        Self: Sized;

    fn target(&self) -> TypeId {
        TypeId::of::<T>()
    }

    fn qualifier(&self) -> Self::Qualifier
    where
        Self: Sized,
    {
        self.qualifier
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_impl_target_succeeds() {
        let i32_key: Box<dyn Key> = Box::new(KeyImpl::<i32, _>::new(()));
        let i32_name1_key: Box<dyn Key> = Box::new(KeyImpl::<i32, _>::new("name1"));
        let i32_name2_key: Box<dyn Key> = Box::new(KeyImpl::<i32, _>::new("name2"));

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
        let i32_key: Box<dyn Key> = Box::new(KeyImpl::<i32, _>::new(()));
        let i32_name1_key: Box<dyn Key> = Box::new(KeyImpl::<i32, _>::new("name1"));
        let i32_name2_key: Box<dyn Key> = Box::new(KeyImpl::<i32, _>::new("name2"));

        assert_ne!(&i32_key, &i32_name1_key);
        assert_ne!(&i32_key, &i32_name2_key);
        assert_ne!(&i32_name1_key, &i32_name2_key);
    }
}
