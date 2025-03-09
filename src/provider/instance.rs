use std::fmt::{Debug, Formatter, Result as FmtResult};

use crate::container::injector::{InjectorError, TypedInjector};
use crate::key::TypedKey;
use crate::provider::TypedProvider;

pub struct CloneableInstanceProvider<K>
where
    K: TypedKey<Target: Clone>,
{
    key: K,
    instance: K::Target,
}

impl<K> CloneableInstanceProvider<K>
where
    K: TypedKey<Target: Clone>,
{
    pub fn new(key: K, instance: K::Target) -> Self {
        Self { key, instance }
    }
}

// SAFETY: Mutable access can be only done through its methods which take
// `&mut self` as the receiver. It's guarenteed that mutable reference is
// exclusive and can't be shared across multiple threads, thus making it
// thread-safe.
unsafe impl<K> Sync for CloneableInstanceProvider<K> where K: TypedKey<Target: Clone> {}

impl<K> Debug for CloneableInstanceProvider<K>
where
    K: TypedKey<Target: Clone>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("CloneableInstanceProvider<K>")
            .field("key", &self.key)
            .finish_non_exhaustive()
    }
}

impl<K> TypedProvider for CloneableInstanceProvider<K>
where
    K: TypedKey<Target: Clone>,
{
    type Key = K;

    type Output = K::Target;

    fn provide<I>(&mut self, _injector: &mut I) -> Result<Self::Output, InjectorError>
    where
        I: TypedInjector + ?Sized,
    {
        Ok(self.instance.clone())
    }

    fn key(&self) -> &Self::Key {
        &self.key
    }
}

pub struct OnceInstanceProvider<K>
where
    K: TypedKey,
{
    key: K,
    instance: Option<K::Target>,
}

impl<K> OnceInstanceProvider<K>
where
    K: TypedKey,
{
    pub fn new(key: K, instance: K::Target) -> Self {
        Self {
            key,
            instance: Some(instance),
        }
    }
}

// SAFETY: Mutable access can be only done through its methods which take
// `&mut self` as the receiver. It's guarenteed that mutable reference is
// exclusive and can't be shared across multiple threads, thus making it
// thread-safe.
unsafe impl<K> Sync for OnceInstanceProvider<K> where K: TypedKey {}

impl<K> Debug for OnceInstanceProvider<K>
where
    K: TypedKey,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("OnceInstanceProvider<K>")
            .field("key", &self.key)
            .finish_non_exhaustive()
    }
}

impl<K> TypedProvider for OnceInstanceProvider<K>
where
    K: TypedKey,
{
    type Key = K;

    type Output = K::Target;

    fn provide<I>(&mut self, _injector: &mut I) -> Result<Self::Output, InjectorError>
    where
        I: TypedInjector + ?Sized,
    {
        match self.instance.take() {
            Some(instance) => Ok(instance),
            None => Err(InjectorError::Consumed {
                key: Box::new(self.key().clone()),
            }),
        }
    }

    fn key(&self) -> &Self::Key {
        &self.key
    }
}

#[cfg(test)]
mod tests {
    use crate::container::injector::MockInjector;
    use crate::key::{self, Key};
    use crate::provider::Provider;

    use super::*;

    #[test]
    fn cloneable_instance_provider_succeeds() {
        let mut provider = CloneableInstanceProvider::new(key::of::<i32>(), 42);
        let mut injector = MockInjector::new();

        assert_eq!(provider.dyn_key(), &key::of::<i32>() as &dyn Key);

        let res = provider.provide(&mut injector);
        assert_eq!(res.unwrap(), 42);

        let res = provider.provide(&mut injector);
        assert_eq!(res.unwrap(), 42);
    }

    #[test]
    fn once_instance_provider_succeeds() {
        let mut provider = OnceInstanceProvider::new(key::of::<i32>(), 42);
        let mut injector = MockInjector::new();

        assert_eq!(provider.dyn_key(), &key::of::<i32>() as &dyn Key);

        let res = provider.provide(&mut injector);
        assert_eq!(res.unwrap(), 42);

        let res = provider.provide(&mut injector);
        assert!(matches!(res.unwrap_err(), InjectorError::Consumed { .. }));
    }
}
