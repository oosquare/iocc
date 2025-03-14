use std::fmt::{Debug, Formatter, Result as FmtResult};

use crate::container::injector::{Injector, InjectorError, TypedInjector};
use crate::container::SharedManaged;
use crate::key::TypedKey;
use crate::provider::{TypedProvider, TypedSharedProvider};

pub struct ClosureProvider<K, C>
where
    K: TypedKey,
    C: Fn(&dyn Injector) -> Result<K::Target, InjectorError> + Send + Sync + 'static,
{
    key: K,
    closure: C,
}

impl<K, C> ClosureProvider<K, C>
where
    K: TypedKey,
    C: Fn(&dyn Injector) -> Result<K::Target, InjectorError> + Send + Sync + 'static,
{
    pub fn new(key: K, closure: C) -> Self {
        Self { key, closure }
    }
}

impl<K, C> Debug for ClosureProvider<K, C>
where
    K: TypedKey,
    C: Fn(&dyn Injector) -> Result<K::Target, InjectorError> + Send + Sync + 'static,
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
    C: Fn(&dyn Injector) -> Result<K::Target, InjectorError> + Send + Sync + 'static,
{
    type Key = K;

    type Output = K::Target;

    fn provide<I>(&self, injector: &I) -> Result<Self::Output, InjectorError>
    where
        I: TypedInjector + ?Sized,
    {
        (self.closure)(injector.upcast_dyn())
    }

    fn key(&self) -> &Self::Key {
        &self.key
    }
}

impl<K, C> TypedSharedProvider for ClosureProvider<K, C>
where
    K: TypedKey<Target: SharedManaged>,
    C: Fn(&dyn Injector) -> Result<K::Target, InjectorError> + Send + Sync + 'static,
{
}

#[cfg(test)]
mod tests {
    use crate::container::injector::MockInjector;
    use crate::key;

    use super::*;

    #[test]
    fn closure_provider_succeeds() {
        let injector = MockInjector::new();
        let provider = ClosureProvider::new(key::of(), |_| Ok(42i32));

        let res = provider.provide(&injector);
        assert_eq!(res.unwrap(), 42);

        let res = provider.provide(&injector);
        assert_eq!(res.unwrap(), 42);
    }
}
