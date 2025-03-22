use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::marker::PhantomData;
use std::sync::Arc;

use crate::container::{Managed, SharedManaged};
use crate::prelude::{InjectorError, TypedInjector};
use crate::provider::closure::Closure;
use crate::provider::{TypedProvider, TypedSharedProvider};
use crate::provider::context::CallContext;

pub struct ClosureProvider<T, C, D>
where
    T: Managed,
    C: Closure<D, Constructed = T>,
    D: Send + Sync + 'static,
{
    closure: C,
    _marker: PhantomData<(T, D)>,
}

impl<T, C, D> ClosureProvider<T, C, D>
where
    T: Managed,
    C: Closure<D, Constructed = T>,
    D: Send + Sync + 'static,
{
    pub fn new(closure: C) -> Self {
        Self {
            closure,
            _marker: PhantomData,
        }
    }
}

impl<T, C, D> Debug for ClosureProvider<T, C, D>
where
    T: Managed,
    C: Closure<D, Constructed = T>,
    D: Send + Sync + 'static,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("ClosureProvider<T, C, D>")
            .finish_non_exhaustive()
    }
}

impl<T, C, D> TypedProvider for ClosureProvider<T, C, D>
where
    T: Managed,
    C: Closure<D, Constructed = T>,
    D: Send + Sync + 'static,
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
        match self.closure.run(injector.upcast_dyn()) {
            Ok(Ok(obj)) => Ok(obj),
            Ok(Err(err)) => Err(InjectorError::ObjectConstruction {
                key: context.key().dyn_clone(),
                source: Arc::from(err.into()),
            }),
            Err(err) => Err(err),
        }
    }
}

impl<T, C, D> TypedSharedProvider for ClosureProvider<T, C, D>
where
    T: SharedManaged,
    C: Closure<D, Constructed = T>,
    D: Send + Sync + 'static,
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
        let mut injector = MockInjector::new();
        injector.expect_dyn_get().returning(|_| Ok(Box::new(42i32)));

        let provider = ClosureProvider::new(|v: i32| Ok::<_, Infallible>(v));

        let res = provider.provide(&injector, &CallContext::new(&key::of::<i32>()));
        assert_eq!(res.unwrap(), 42);

        let res = provider.provide(&injector, &CallContext::new(&key::of::<i32>()));
        assert_eq!(res.unwrap(), 42);
    }
}
