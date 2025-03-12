use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::marker::PhantomData;

use crate::component::Component;
use crate::container::injector::{InjectorError, TypedInjector};
use crate::container::SharedManaged;
use crate::key::TypedKey;
use crate::provider::{TypedProvider, TypedSharedProvider};

pub struct ComponentProvider<K, C>
where
    K: TypedKey<Target = C::Output>,
    C: Component,
{
    key: K,
    _marker: PhantomData<C>,
}

impl<K, C> ComponentProvider<K, C>
where
    K: TypedKey<Target = C::Output>,
    C: Component,
{
    pub fn new(key: K) -> Self {
        Self {
            key,
            _marker: PhantomData,
        }
    }
}

// SAFETY: The provider neither contains any `C` nor has any access to other `C`.
// Fields other than `PhantomData` are `Sync`, so it should also be `Sync`.
unsafe impl<K, C> Sync for ComponentProvider<K, C>
where
    K: TypedKey<Target = C::Output>,
    C: Component,
{
}

impl<K, C> Debug for ComponentProvider<K, C>
where
    K: TypedKey<Target = C::Output>,
    C: Component,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("ComponentProvider<K, C>")
            .field("key", &self.key)
            .finish_non_exhaustive()
    }
}

impl<K, C> TypedProvider for ComponentProvider<K, C>
where
    K: TypedKey<Target = C::Output>,
    C: Component,
{
    type Key = K;

    type Output = K::Target;

    fn provide<I>(&mut self, injector: &mut I) -> Result<Self::Output, InjectorError>
    where
        I: TypedInjector + ?Sized,
    {
        match C::construct(injector) {
            Ok(Ok(obj)) => Ok(obj.post_process()),
            Ok(Err(err)) => Err(InjectorError::ObjectConstruction {
                key: Box::new(self.key),
                source: err.into(),
            }),
            Err(err) => Err(err),
        }
    }

    fn key(&self) -> &Self::Key {
        &self.key
    }
}

impl<K, C> TypedSharedProvider for ComponentProvider<K, C>
where
    K: TypedKey<Target = C::Output>,
    C: Component<Output: SharedManaged>,
{
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;
    use std::sync::Arc;

    use crate::container::injector::MockInjector;
    use crate::key;
    use crate::provider::SharedProvider;

    use super::*;

    pub trait Abstract: Send + Sync + 'static {}

    pub struct Impl;

    impl Abstract for Impl {}

    impl Component for Impl {
        type Output = Arc<dyn Abstract>;

        type Error = Infallible;

        fn construct<I>(_injector: &mut I) -> Result<Result<Self, Self::Error>, InjectorError>
        where
            I: TypedInjector + ?Sized,
        {
            Ok(Ok(Impl))
        }

        fn post_process(self) -> Self::Output {
            Arc::new(self)
        }
    }

    #[test]
    fn component_provider_succeeds() {
        let mut injector = MockInjector::new();
        let mut provider = ComponentProvider::<_, Impl>::new(key::of::<Arc<dyn Abstract>>());
        assert!(provider.provide(&mut injector).is_ok());

        assert_is_shared_provider(&provider);
    }

    fn assert_is_shared_provider(_: &dyn SharedProvider) {}
}
