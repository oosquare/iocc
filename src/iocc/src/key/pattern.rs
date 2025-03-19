use std::any::TypeId;
use std::marker::PhantomData;

use crate::container::Managed;
use crate::key::{Key, TypedQualifier};

use super::Qualifier;

pub trait Pattern {
    type Target: Managed;

    type Qualifier;

    fn matches(&self, key: &dyn Key) -> bool;
}

pub struct AnyPattern<T>
where
    T: Managed,
{
    _marker: PhantomData<T>,
}

impl<T> AnyPattern<T>
where
    T: Managed,
{
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T> Pattern for AnyPattern<T>
where
    T: Managed,
{
    type Target = T;

    type Qualifier = Box<dyn Qualifier>;

    fn matches(&self, key: &dyn Key) -> bool {
        key.target_type() == TypeId::of::<T>()
    }
}

pub struct KeyTypePattern<T, Q>
where
    T: Managed,
    Q: TypedQualifier,
{
    _marker: PhantomData<(T, Q)>,
}

impl<T, Q> KeyTypePattern<T, Q>
where
    T: Managed,
    Q: TypedQualifier,
{
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T, Q> Pattern for KeyTypePattern<T, Q>
where
    T: Managed,
    Q: TypedQualifier,
{
    type Target = T;

    type Qualifier = Q;

    fn matches(&self, key: &dyn Key) -> bool {
        key.target_type() == TypeId::of::<T>() && key.qualifier_type() == TypeId::of::<Q>()
    }
}

#[cfg(test)]
mod tests {
    use crate::key;

    use super::*;

    #[test]
    fn any_pattern_matches_succeeds() {
        let pattern: AnyPattern<i32> = AnyPattern::new();
        assert!(pattern.matches(&key::of::<i32>()));
        assert!(pattern.matches(&key::named::<i32>("named")));
        assert!(!pattern.matches(&key::of::<i64>()));
        assert!(!pattern.matches(&key::named::<i64>("named")));
    }

    #[test]
    fn key_type_pattern_matches_succeeds() {
        let pattern: KeyTypePattern<i32, ()> = KeyTypePattern::new();
        assert!(pattern.matches(&key::of::<i32>()));
        assert!(!pattern.matches(&key::named::<i32>("named")));
        assert!(!pattern.matches(&key::of::<i64>()));
        assert!(!pattern.matches(&key::named::<i64>("named")));
    }
}
