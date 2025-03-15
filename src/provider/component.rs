use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::marker::PhantomData;
use std::sync::Arc;

use crate::component::Component;
use crate::container::injector::{InjectorError, TypedInjector};
use crate::container::SharedManaged;
use crate::provider::{CallContext, TypedProvider, TypedSharedProvider};

pub struct ComponentProvider<C>
where
    C: Component,
{
    _marker: PhantomData<C>,
}

impl<C> ComponentProvider<C>
where
    C: Component,
{
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<C> Debug for ComponentProvider<C>
where
    C: Component,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("ComponentProvider<C>")
            .finish_non_exhaustive()
    }
}

impl<C> TypedProvider for ComponentProvider<C>
where
    C: Component,
{
    type Output = C::Output;

    fn provide<I>(
        &self,
        injector: &I,
        context: &CallContext<'_>,
    ) -> Result<Self::Output, InjectorError>
    where
        I: TypedInjector + ?Sized,
    {
        match C::construct(injector) {
            Ok(Ok(obj)) => Ok(obj.post_process()),
            Ok(Err(err)) => Err(InjectorError::ObjectConstruction {
                key: context.key().dyn_clone(),
                source: Arc::from(err.into()),
            }),
            Err(err) => Err(err),
        }
    }
}

impl<C> TypedSharedProvider for ComponentProvider<C> where C: Component<Output: SharedManaged> {}

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

        fn construct<I>(_injector: &I) -> Result<Result<Self, Self::Error>, InjectorError>
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
        let injector = MockInjector::new();
        let provider = ComponentProvider::<Impl>::new();
        assert!(provider
            .provide(
                &injector,
                &CallContext::new(&key::of::<Arc<dyn Abstract>>())
            )
            .is_ok());

        assert_is_shared_provider(&provider);
    }

    fn assert_is_shared_provider(_: &dyn SharedProvider) {}
}
