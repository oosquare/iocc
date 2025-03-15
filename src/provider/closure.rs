use std::fmt::{Debug, Formatter, Result as FmtResult};

use crate::container::injector::{Injector, InjectorError, TypedInjector};
use crate::container::{Managed, SharedManaged};
use crate::provider::{CallContext, TypedProvider, TypedSharedProvider};

pub struct ClosureProvider<T, C>
where
    T: Managed,
    C: Fn(&dyn Injector) -> Result<T, InjectorError> + Send + Sync + 'static,
{
    closure: C,
}

impl<T, C> ClosureProvider<T, C>
where
    T: Managed,
    C: Fn(&dyn Injector) -> Result<T, InjectorError> + Send + Sync + 'static,
{
    pub fn new(closure: C) -> Self {
        Self { closure }
    }
}

impl<T, C> Debug for ClosureProvider<T, C>
where
    T: Managed,
    C: Fn(&dyn Injector) -> Result<T, InjectorError> + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("ClosureProvider<T, C>")
            .finish_non_exhaustive()
    }
}

impl<T, C> TypedProvider for ClosureProvider<T, C>
where
    T: Managed,
    C: Fn(&dyn Injector) -> Result<T, InjectorError> + Send + Sync + 'static,
{
    type Output = T;

    fn provide<I>(
        &self,
        injector: &I,
        _context: &CallContext<'_>,
    ) -> Result<Self::Output, InjectorError>
    where
        I: TypedInjector + ?Sized,
    {
        (self.closure)(injector.upcast_dyn())
    }
}

impl<T, C> TypedSharedProvider for ClosureProvider<T, C>
where
    T: SharedManaged,
    C: Fn(&dyn Injector) -> Result<T, InjectorError> + Send + Sync + 'static,
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
        let provider = ClosureProvider::new(|_| Ok(42i32));

        let res = provider.provide(&injector, &CallContext::new(&key::of::<i32>()));
        assert_eq!(res.unwrap(), 42);

        let res = provider.provide(&injector, &CallContext::new(&key::of::<i32>()));
        assert_eq!(res.unwrap(), 42);
    }
}
