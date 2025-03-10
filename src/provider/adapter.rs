use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::sync::Arc;

use crate::container::injector::{InjectorError, TypedInjector};
use crate::key::{self, KeyImpl, TypedKey};
use crate::provider::{TypedProvider, TypedSharedProvider};

pub struct CachedProviderAdapter<P>
where
    P: TypedProvider<Output: Clone>,
{
    inner: P,
    value: Option<P::Output>,
}

impl<P> CachedProviderAdapter<P>
where
    P: TypedProvider<Output: Clone>,
{
    pub fn new(provider: P) -> Self {
        Self {
            inner: provider,
            value: None,
        }
    }
}

// SAFETY: Mutable access can be only done through its methods which take
// `&mut self` as the receiver. It's guarenteed that mutable reference is
// exclusive and can't be shared across multiple threads, thus making it
// thread-safe.
unsafe impl<P> Sync for CachedProviderAdapter<P> where P: TypedProvider<Output: Clone> {}

impl<P> Debug for CachedProviderAdapter<P>
where
    P: TypedProvider<Output: Clone>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("CachedProviderAdapter<P>")
            .field("inner", &self.inner)
            .finish_non_exhaustive()
    }
}

impl<P> TypedProvider for CachedProviderAdapter<P>
where
    P: TypedProvider<Output: Clone>,
{
    type Key = P::Key;

    type Output = P::Output;

    fn provide<I>(&mut self, injector: &mut I) -> Result<Self::Output, InjectorError>
    where
        I: TypedInjector + ?Sized,
    {
        if let Some(value) = &self.value {
            Ok(value.clone())
        } else {
            let value = self.inner.provide(injector)?;
            self.value = Some(value.clone());
            Ok(value)
        }
    }

    fn key(&self) -> &Self::Key {
        self.inner.key()
    }
}

#[derive(Debug)]
pub struct OnceProviderAdapter<P>
where
    P: TypedProvider,
{
    inner: P,
    consumed: bool,
}

impl<P> OnceProviderAdapter<P>
where
    P: TypedProvider,
{
    fn new(provider: P) -> Self {
        Self {
            inner: provider,
            consumed: false,
        }
    }
}

impl<P> From<P> for OnceProviderAdapter<P>
where
    P: TypedProvider,
{
    fn from(provider: P) -> Self {
        Self::new(provider)
    }
}

impl<P> TypedProvider for OnceProviderAdapter<P>
where
    P: TypedProvider,
{
    type Key = P::Key;

    type Output = P::Output;

    fn provide<I>(&mut self, injector: &mut I) -> Result<Self::Output, InjectorError>
    where
        I: TypedInjector + ?Sized,
    {
        if !self.consumed {
            self.consumed = true;
            self.inner.provide(injector)
        } else {
            Err(InjectorError::Consumed {
                key: Box::new(*self.key()),
            })
        }
    }

    fn key(&self) -> &Self::Key {
        self.inner.key()
    }
}

#[derive(Debug)]
pub struct SharedProviderAdapter<P>
where
    P: TypedProvider<Output: Sync>,
{
    inner: P,
    key: <Self as TypedProvider>::Key,
}

impl<P> SharedProviderAdapter<P>
where
    P: TypedProvider<Output: Sync>,
{
    pub fn new(provider: P) -> Self {
        let key = key::qualified(provider.key().qualifier());
        Self {
            inner: provider,
            key,
        }
    }
}

impl<P> TypedProvider for SharedProviderAdapter<P>
where
    P: TypedProvider<Output: Sync>,
{
    type Key = KeyImpl<Self::Output, <P::Key as TypedKey>::Qualifier>;

    type Output = Arc<P::Output>;

    fn provide<I>(&mut self, injector: &mut I) -> Result<Self::Output, InjectorError>
    where
        I: TypedInjector + ?Sized,
    {
        match self.inner.provide(injector) {
            Ok(obj) => Ok(Arc::new(obj)),
            Err(err) => Err(InjectorError::AdapterInner {
                key: Box::new(self.key),
                source: Box::new(err),
            }),
        }
    }

    fn key(&self) -> &Self::Key {
        &self.key
    }
}

impl<P> TypedSharedProvider for SharedProviderAdapter<P> where P: TypedProvider<Output: Sync> {}

#[cfg(test)]
mod tests {
    use crate::container::injector::MockInjector;
    use crate::key;
    use crate::provider::instance::{CloneableInstanceProvider, OnceInstanceProvider};

    use super::*;

    #[test]
    fn cloneable_provider_adapter_succeeds() {
        let mut injector = MockInjector::new();

        let provider = OnceInstanceProvider::new(key::of(), 42i32);
        let mut provider = CachedProviderAdapter::new(provider);

        let res = provider.provide(&mut injector);
        assert_eq!(res.unwrap(), 42i32);

        let res = provider.provide(&mut injector);
        assert_eq!(res.unwrap(), 42i32);
    }

    #[test]
    fn once_provider_adapter_succeeds() {
        let mut injector = MockInjector::new();

        let provider = CloneableInstanceProvider::new(key::of(), 42i32);
        let mut provider = OnceProviderAdapter::new(provider);

        let res = provider.provide(&mut injector);
        assert_eq!(res.unwrap(), 42i32);

        let res = provider.provide(&mut injector);
        assert!(matches!(res.unwrap_err(), InjectorError::Consumed { .. }));
    }

    #[test]
    fn shared_provider_adapter_succeeds() {
        let mut injector = MockInjector::new();

        let provider = CloneableInstanceProvider::new(key::of(), 42i32);
        let mut provider = SharedProviderAdapter::new(provider);

        let res = provider.provide(&mut injector);
        assert_eq!(res.unwrap(), Arc::new(42i32));
    }
}
