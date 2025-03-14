use std::error::Error;

use crate::container::registry::provider_map::ProviderMap;
use crate::container::registry::{Configurer, RegistryError};
use crate::provider::{Provider, SharedProvider};
use crate::scope::Scope;

pub struct ConfigurerImpl<S: Scope> {
    providers: ProviderMap<S>,
    errors: Vec<RegistryError>,
}

impl<S: Scope> ConfigurerImpl<S> {
    #[cfg_attr(not(test), expect(dead_code))]
    pub fn new() -> Self {
        Self {
            providers: ProviderMap::new(),
            errors: Vec::new(),
        }
    }

    #[cfg_attr(not(test), expect(dead_code))]
    pub fn finish(self) -> Result<ProviderMap<S>, Vec<RegistryError>> {
        if self.errors.is_empty() {
            Ok(self.providers)
        } else {
            Err(self.errors)
        }
    }
}

impl<S: Scope> Configurer for ConfigurerImpl<S> {
    type Scope = S;

    fn register(&mut self, provider: Box<dyn Provider>) {
        if self.providers.get(provider.dyn_key()).is_none() {
            self.providers.insert(provider);
        } else {
            self.errors.push(RegistryError::KeyDuplicated {
                key: provider.dyn_key().dyn_clone(),
            });
        }
    }

    fn register_shared(&mut self, provider: Box<dyn SharedProvider>, scope: S) {
        if self.providers.get(provider.dyn_key()).is_none() {
            self.providers.insert_shared(provider, scope);
        } else {
            self.errors.push(RegistryError::KeyDuplicated {
                key: provider.dyn_key().dyn_clone(),
            });
        }
    }

    fn report_module_error(&mut self, module: &'static str, err: Box<dyn Error + Send + Sync>) {
        self.errors.push(RegistryError::ModuleInner {
            module,
            source: err,
        });
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, sync::Arc};

    use crate::container::injector::{InjectorError, TypedInjector};
    use crate::key::{self, KeyImpl};
    use crate::provider::{TypedProvider, TypedSharedProvider};
    use crate::scope::SingletonScope;

    use super::*;

    #[test]
    fn configurer_impl_register_succeeds() {
        let mut configurer = ConfigurerImpl::new();
        configurer.register(Box::new(TestProvider::new(42i32)));
        configurer.register_shared(Box::new(TestProvider::new(Arc::new("str"))), SingletonScope);

        let map = configurer.finish().unwrap();
        assert!(map.get(&key::of::<i32>()).is_some());
        assert!(map.get(&key::of::<Arc<&str>>()).is_some());
    }

    #[test]
    fn configurer_impl_finish_fails_when_key_is_duplicated() {
        let mut configurer: ConfigurerImpl<SingletonScope> = ConfigurerImpl::new();
        configurer.register(Box::new(TestProvider::new(42i32)));
        configurer.register(Box::new(TestProvider::new(42i32)));

        let errs = configurer.finish().unwrap_err();
        assert!(matches!(
            errs.first().unwrap(),
            RegistryError::KeyDuplicated { .. }
        ));
    }

    #[test]
    fn configurer_impl_finish_fails_when_other_error_reported() {
        let mut configurer = ConfigurerImpl::new();
        configurer.register(Box::new(TestProvider::new(42i32)));
        configurer.register_shared(Box::new(TestProvider::new(Arc::new("str"))), SingletonScope);
        configurer.report_module_error("test", "whatever".into());

        let errs = configurer.finish().unwrap_err();
        assert!(matches!(
            errs.first().unwrap(),
            RegistryError::ModuleInner { .. }
        ));
    }

    #[derive(Debug)]
    struct TestProvider<T>
    where
        T: Clone + Debug + Send + Sync + 'static,
    {
        value: T,
        key: KeyImpl<T, ()>,
    }

    impl<T> TestProvider<T>
    where
        T: Clone + Debug + Send + Sync + 'static,
    {
        pub fn new(value: T) -> Self {
            Self {
                value,
                key: key::of::<T>(),
            }
        }
    }

    impl<T> TypedProvider for TestProvider<T>
    where
        T: Clone + Debug + Send + Sync + 'static,
    {
        type Key = KeyImpl<T, ()>;

        type Output = T;

        fn provide<I>(&self, _injector: &I) -> Result<Self::Output, InjectorError>
        where
            I: TypedInjector + ?Sized,
        {
            Ok(self.value.clone())
        }

        fn key(&self) -> &Self::Key {
            &self.key
        }
    }

    impl<T> TypedSharedProvider for TestProvider<Arc<T>> where T: Debug + Send + Sync + 'static {}
}
