use std::fmt::{Debug, Formatter, Result as FmtResult};

use crate::container::injector::{Injector, InjectorError, TypedInjector};
use crate::key::TypedKey;
use crate::provider::TypedProvider;

pub struct ClosureProvider<K, C>
where
    K: TypedKey,
    C: FnMut(&mut dyn Injector) -> Result<K::Target, InjectorError> + Send + Sync + 'static,
{
    key: K,
    closure: C,
}

impl<K, C> ClosureProvider<K, C>
where
    K: TypedKey,
    C: FnMut(&mut dyn Injector) -> Result<K::Target, InjectorError> + Send + Sync + 'static,
{
    pub fn new(key: K, closure: C) -> Self {
        Self { key, closure }
    }
}

impl<K, C> Debug for ClosureProvider<K, C>
where
    K: TypedKey,
    C: FnMut(&mut dyn Injector) -> Result<K::Target, InjectorError> + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("ClosureProvider<K, C>")
            .field("key", self.key())
            .finish_non_exhaustive()
    }
}

impl<K, C> TypedProvider for ClosureProvider<K, C>
where
    K: TypedKey,
    C: FnMut(&mut dyn Injector) -> Result<K::Target, InjectorError> + Send + Sync + 'static,
{
    type Key = K;

    type Output = K::Target;

    fn provide<I>(&mut self, injector: &mut I) -> Result<Self::Output, InjectorError>
    where
        I: TypedInjector + ?Sized,
    {
        (self.closure)(injector.upcast_dyn())
    }

    fn key(&self) -> &Self::Key {
        &self.key
    }
}

pub struct OnceClosureProvider<K, C>
where
    K: TypedKey,
    C: FnOnce(&mut dyn Injector) -> Result<K::Target, InjectorError> + Send + Sync + 'static,
{
    key: K,
    closure: Option<C>,
}

impl<K, C> OnceClosureProvider<K, C>
where
    K: TypedKey,
    C: FnOnce(&mut dyn Injector) -> Result<K::Target, InjectorError> + Send + Sync + 'static,
{
    pub fn new(key: K, closure: C) -> Self {
        Self {
            key,
            closure: Some(closure),
        }
    }
}

impl<K, C> Debug for OnceClosureProvider<K, C>
where
    K: TypedKey,
    C: FnOnce(&mut dyn Injector) -> Result<K::Target, InjectorError> + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("OnceClosureProvider<K, C>")
            .field("key", self.key())
            .finish_non_exhaustive()
    }
}

impl<K, C> TypedProvider for OnceClosureProvider<K, C>
where
    K: TypedKey,
    C: FnOnce(&mut dyn Injector) -> Result<K::Target, InjectorError> + Send + Sync + 'static,
{
    type Key = K;

    type Output = K::Target;

    fn provide<I>(&mut self, injector: &mut I) -> Result<Self::Output, InjectorError>
    where
        I: TypedInjector + ?Sized,
    {
        match self.closure.take() {
            Some(closure) => closure(injector.upcast_dyn()),
            None => Err(InjectorError::Consumed {
                key: Box::new(self.key),
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
    use crate::key;

    use super::*;

    #[test]
    fn closure_provider_succeeds() {
        let mut injector = MockInjector::new();
        let mut provider = ClosureProvider::new(key::of(), |_| Ok(42i32));

        let res = provider.provide(&mut injector);
        assert_eq!(res.unwrap(), 42);

        let res = provider.provide(&mut injector);
        assert_eq!(res.unwrap(), 42);
    }

    #[test]
    fn once_closure_provider_succeeds() {
        let mut injector = MockInjector::new();
        let mut provider = OnceClosureProvider::new(key::of(), |_| Ok(42i32));

        let res = provider.provide(&mut injector);
        assert_eq!(res.unwrap(), 42);

        let res = provider.provide(&mut injector);
        assert!(matches!(res.unwrap_err(), InjectorError::Consumed { .. }));
    }
}
