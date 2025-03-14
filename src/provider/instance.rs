use std::fmt::{Debug, Formatter, Result as FmtResult};

use crate::container::injector::{InjectorError, TypedInjector};
use crate::container::SharedManaged;
use crate::key::TypedKey;
use crate::provider::{TypedProvider, TypedSharedProvider};

pub struct InstanceProvider<K>
where
    K: TypedKey<Target: Clone>,
{
    key: K,
    instance: K::Target,
}

impl<K> InstanceProvider<K>
where
    K: TypedKey<Target: Clone>,
{
    pub fn new(key: K, instance: K::Target) -> Self {
        Self { key, instance }
    }
}

impl<K> Debug for InstanceProvider<K>
where
    K: TypedKey<Target: Clone>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("InstanceProvider<K>")
            .field("key", &self.key)
            .finish_non_exhaustive()
    }
}

impl<K> TypedProvider for InstanceProvider<K>
where
    K: TypedKey<Target: Clone>,
{
    type Key = K;

    type Output = K::Target;

    fn provide<I>(&self, _injector: &I) -> Result<Self::Output, InjectorError>
    where
        I: TypedInjector + ?Sized,
    {
        Ok(self.instance.clone())
    }

    fn key(&self) -> &Self::Key {
        &self.key
    }
}

impl<K> TypedSharedProvider for InstanceProvider<K> where K: TypedKey<Target: Clone + SharedManaged> {}

#[cfg(test)]
mod tests {
    use crate::container::injector::MockInjector;
    use crate::key::{self, Key};
    use crate::provider::Provider;

    use super::*;

    #[test]
    fn instance_provider_succeeds() {
        let provider = InstanceProvider::new(key::of::<i32>(), 42);
        let injector = MockInjector::new();

        assert_eq!(provider.dyn_key(), &key::of::<i32>() as &dyn Key);

        let res = provider.provide(&injector);
        assert_eq!(res.unwrap(), 42);

        let res = provider.provide(&injector);
        assert_eq!(res.unwrap(), 42);
    }
}
