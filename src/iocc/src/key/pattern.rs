use std::any::TypeId;
use std::marker::PhantomData;

use crate::container::Managed;
use crate::key::{Key, Qualifier, TypedQualifier};

/// A pattern used to match against all kinds of keys.
pub trait Pattern {
    /// The target type that all matched keys should have.
    type Target: Managed;

    /// The qualifier type that all matched keys should have.
    type Qualifier;

    /// Tests whether the key matches the pattern.
    fn matches(&self, key: &dyn Key) -> bool;
}

/// A [`Pattern`] which matches all keys of target type `T` and any qualifier
/// type.
///
/// # Examples
///
/// ```rust
/// # use iocc::key::{self, Pattern, AnyPattern};
/// let pattern = AnyPattern::<i32>::new();
/// assert!(pattern.matches(&key::of::<i32>()));
/// assert!(pattern.matches(&key::named::<i32>("named")));
/// assert!(!pattern.matches(&key::of::<i64>()));
/// assert!(!pattern.matches(&key::named::<i64>("named")));
/// ```
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
    /// Creates a new [`AnyPattern`].
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

/// A [`Pattern`] which matches all keys of target type `T` and qualifier type
/// `Q`.
///
/// # Examples
///
/// ```rust
/// # use iocc::key::{self, Pattern, KeyTypePattern};
/// let pattern = KeyTypePattern::<i32, ()>::new();
/// assert!(pattern.matches(&key::of::<i32>()));
/// assert!(!pattern.matches(&key::named::<i32>("named")));
/// assert!(!pattern.matches(&key::of::<i64>()));
/// assert!(!pattern.matches(&key::named::<i64>("named")));
/// ```
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
