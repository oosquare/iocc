use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::sync::Arc;

use crate::container::injector::{InjectorError, TypedInjector};
use crate::container::{Managed, SharedManaged};
use crate::provider::closure::RawClosure;
use crate::provider::{CallContext, TypedProvider, TypedSharedProvider};

pub struct RawClosureProvider<T, C>
where
    T: Managed,
    C: RawClosure<Constructed = T>,
{
    closure: C,
}

impl<T, C> RawClosureProvider<T, C>
where
    T: Managed,
    C: RawClosure<Constructed = T>,
{
    pub fn new(closure: C) -> Self {
        Self { closure }
    }
}

impl<T, C> Debug for RawClosureProvider<T, C>
where
    T: Managed,
    C: RawClosure<Constructed = T>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("RawClosureProvider<T, C>")
            .finish_non_exhaustive()
    }
}

impl<T, C> TypedProvider for RawClosureProvider<T, C>
where
    T: Managed,
    C: RawClosure<Constructed = T>,
{
    type Output = T;

    fn provide<I>(
        &self,
        injector: &I,
        context: &CallContext<'_>,
    ) -> Result<Self::Output, InjectorError>
    where
        I: TypedInjector + ?Sized,
    {
        match (self.closure)(injector.upcast_dyn()) {
            Ok(Ok(obj)) => Ok(obj),
            Ok(Err(err)) => Err(InjectorError::ObjectConstruction {
                key: context.key().dyn_clone(),
                source: Arc::from(err.into()),
            }),
            Err(err) => Err(err),
        }
    }
}

impl<T, C> TypedSharedProvider for RawClosureProvider<T, C>
where
    T: SharedManaged,
    C: RawClosure<Constructed = T>,
{
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;

    use crate::container::injector::MockInjector;
    use crate::key;

    use super::*;

    #[test]
    fn closure_provider_succeeds() {
        let injector = MockInjector::new();
        let provider = RawClosureProvider::new(|_| Ok(Ok::<_, Infallible>(42i32)));

        let res = provider.provide(&injector, &CallContext::new(&key::of::<i32>()));
        assert_eq!(res.unwrap(), 42);

        let res = provider.provide(&injector, &CallContext::new(&key::of::<i32>()));
        assert_eq!(res.unwrap(), 42);
    }
}
