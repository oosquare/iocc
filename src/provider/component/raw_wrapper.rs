use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::marker::PhantomData;
use std::sync::Arc;

use crate::container::injector::{InjectorError, TypedInjector};
use crate::container::SharedManaged;
use crate::provider::component::RawComponent;
use crate::provider::{CallContext, TypedProvider, TypedSharedProvider};

pub struct RawComponentProvider<C>
where
    C: RawComponent,
{
    _marker: PhantomData<C>,
}

impl<C> RawComponentProvider<C>
where
    C: RawComponent,
{
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<C> Debug for RawComponentProvider<C>
where
    C: RawComponent,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("RawComponentProvider<C>")
            .finish_non_exhaustive()
    }
}

impl<C> TypedProvider for RawComponentProvider<C>
where
    C: RawComponent,
{
    type Output = C::RawConstructed;

    fn provide<I>(
        &self,
        injector: &I,
        context: &CallContext<'_>,
    ) -> Result<Self::Output, InjectorError>
    where
        I: TypedInjector + ?Sized,
    {
        match C::construct(injector) {
            Ok(Ok(obj)) => Ok(obj.raw_post_process()),
            Ok(Err(err)) => Err(InjectorError::ObjectConstruction {
                key: context.key().dyn_clone(),
                source: Arc::from(err.into()),
            }),
            Err(err) => Err(err),
        }
    }
}

impl<C> TypedSharedProvider for RawComponentProvider<C> where
    C: RawComponent<RawConstructed: SharedManaged>
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

    impl RawComponent for Impl {
        type RawConstructed = Arc<dyn Abstract>;

        type RawError = Infallible;

        fn construct<I>(_injector: &I) -> Result<Result<Self, Self::RawError>, InjectorError>
        where
            I: TypedInjector + ?Sized,
        {
            Ok(Ok(Impl))
        }

        fn raw_post_process(self) -> Self::RawConstructed {
            Arc::new(self)
        }
    }

    #[test]
    fn component_provider_succeeds() {
        let injector = MockInjector::new();
        let provider = RawComponentProvider::<Impl>::new();
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
