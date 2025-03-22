use std::fmt::{Debug, Formatter, Result as FmtResult};

use crate::container::injector::{InjectorError, TypedInjector};
use crate::container::{Managed, SharedManaged};
use crate::provider::{TypedProvider, TypedSharedProvider};
use crate::provider::context::CallContext;

pub struct InstanceProvider<T>
where
    T: Managed + Clone,
{
    instance: T,
}

impl<T> InstanceProvider<T>
where
    T: Managed + Clone,
{
    pub fn new(instance: T) -> Self {
        Self { instance }
    }
}

impl<T> Debug for InstanceProvider<T>
where
    T: Managed + Clone,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("InstanceProvider<T>")
            .finish_non_exhaustive()
    }
}

impl<T> TypedProvider for InstanceProvider<T>
where
    T: Managed + Clone,
{
    type Output = T;

    fn provide<I>(
        &self,
        _injector: &I,
        _context: &CallContext<'_>,
    ) -> Result<Self::Output, InjectorError>
    where
        I: TypedInjector + ?Sized,
    {
        Ok(self.instance.clone())
    }
}

impl<T> TypedSharedProvider for InstanceProvider<T> where T: SharedManaged + Clone {}

#[cfg(test)]
mod tests {
    use crate::container::injector::MockInjector;
    use crate::key;

    use super::*;

    #[test]
    fn instance_provider_succeeds() {
        let provider = InstanceProvider::new(42);
        let injector = MockInjector::new();

        let res = provider.provide(&injector, &CallContext::new(&key::of::<i32>()));
        assert_eq!(res.unwrap(), 42);

        let res = provider.provide(&injector, &CallContext::new(&key::of::<i32>()));
        assert_eq!(res.unwrap(), 42);
    }
}
