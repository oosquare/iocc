use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::marker::PhantomData;
use std::sync::Arc;

use crate::container::injector::{CallContext, ContextForwardingInjectorProxy};
use crate::container::{Managed, SharedManaged};
use crate::prelude::{InjectorError, TypedInjector};
use crate::provider::closure::Closure;
use crate::provider::{TypedProvider, TypedSharedProvider};

/// A [`Provider`] which supplies objects from a [`Closure`].
///
/// Note that each argument of the closure is fetched without specifying a
/// qualifier.
///
/// # Examples
///
/// ```rust
/// # use std::convert::Infallible;
/// # use iocc::provider::closure::ClosureProvider;
/// let closure = |a: i32, b: f64| Ok::<_, Infallible>((a, b));
/// let provider = ClosureProvider::new(closure);
/// ```
///
/// [`Provider`]: crate::provider::Provider
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
    /// Creates a new [`ClosureProvider`] from a [`Closure`].
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
        let injector = ContextForwardingInjectorProxy::new(injector, context);
        match self.closure.run(&injector) {
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
        injector
            .expect_dyn_get_dependency()
            .returning(|_, _| Ok(Box::new(42i32)));

        let provider = ClosureProvider::new(|v: i32| Ok::<_, Infallible>(v));

        let res = provider.provide(&injector, &CallContext::new(&key::of::<i32>()));
        assert_eq!(res.unwrap(), 42);

        let res = provider.provide(&injector, &CallContext::new(&key::of::<i32>()));
        assert_eq!(res.unwrap(), 42);
    }
}
