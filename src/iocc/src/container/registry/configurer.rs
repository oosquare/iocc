use std::error::Error;

use crate::container::registry::provider_map::ProviderMap;
use crate::container::registry::{Configurer, ConfigurerPrivate, RegistryError};
use crate::key::Key;
use crate::provider::{Provider, SharedProvider};
use crate::scope::Scope;

pub struct ConfigurerImpl<S: Scope> {
    providers: ProviderMap<S>,
    errors: Vec<RegistryError>,
}

impl<S: Scope> ConfigurerImpl<S> {
    pub fn new() -> Self {
        Self {
            providers: ProviderMap::new(),
            errors: Vec::new(),
        }
    }

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

    #[allow(private_interfaces)]
    fn as_private(&mut self) -> &mut dyn ConfigurerPrivate<Scope = Self::Scope> {
        self
    }

    fn report_module_error(&mut self, module: &'static str, err: Box<dyn Error + Send + Sync>) {
        self.errors.push(RegistryError::ModuleInner {
            module,
            source: err,
        });
    }
}

impl<S: Scope> ConfigurerPrivate for ConfigurerImpl<S> {
    fn dyn_register(&mut self, key: Box<dyn Key>, provider: Box<dyn Provider>) {
        if self.providers.get(key.as_ref()).is_none() {
            self.providers.insert(key, provider);
        } else {
            self.errors.push(RegistryError::KeyDuplicated {
                key: key.dyn_clone(),
            });
        }
    }

    fn dyn_register_shared(
        &mut self,
        key: Box<dyn Key>,
        provider: Box<dyn SharedProvider>,
        scope: S,
    ) {
        if self.providers.get(key.as_ref()).is_none() {
            self.providers.insert_shared(key, provider, scope);
        } else {
            self.errors.push(RegistryError::KeyDuplicated {
                key: key.dyn_clone(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, sync::Arc};

    use crate::container::injector::{InjectorError, TypedInjector};
    use crate::key;
    use crate::provider::{TypedProvider, TypedSharedProvider};
    use crate::provider::context::CallContext;
    use crate::scope::SingletonScope;

    use super::*;

    #[test]
    fn configurer_impl_register_succeeds() {
        let mut configurer = ConfigurerImpl::new();
        configurer.dyn_register(
            Box::new(key::of::<i32>()),
            Box::new(TestProvider::new(42i32)),
        );
        configurer.dyn_register_shared(
            Box::new(key::of::<Arc<&'static str>>()),
            Box::new(TestProvider::new(Arc::new("str"))),
            SingletonScope,
        );

        let map = configurer.finish().unwrap();
        assert!(map.get(&key::of::<i32>()).is_some());
        assert!(map.get(&key::of::<Arc<&str>>()).is_some());
    }

    #[test]
    fn configurer_impl_finish_fails_when_key_is_duplicated() {
        let mut configurer: ConfigurerImpl<SingletonScope> = ConfigurerImpl::new();
        configurer.dyn_register(
            Box::new(key::of::<i32>()),
            Box::new(TestProvider::new(42i32)),
        );
        configurer.dyn_register(
            Box::new(key::of::<i32>()),
            Box::new(TestProvider::new(42i32)),
        );

        let errs = configurer.finish().unwrap_err();
        assert!(matches!(
            errs.first().unwrap(),
            RegistryError::KeyDuplicated { .. }
        ));
    }

    #[test]
    fn configurer_impl_finish_fails_when_other_error_reported() {
        let mut configurer = ConfigurerImpl::new();
        configurer.dyn_register(
            Box::new(key::of::<i32>()),
            Box::new(TestProvider::new(42i32)),
        );
        configurer.dyn_register_shared(
            Box::new(key::of::<Arc<&'static str>>()),
            Box::new(TestProvider::new(Arc::new("str"))),
            SingletonScope,
        );
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
    }

    impl<T> TestProvider<T>
    where
        T: Clone + Debug + Send + Sync + 'static,
    {
        pub fn new(value: T) -> Self {
            Self { value }
        }
    }

    impl<T> TypedProvider for TestProvider<T>
    where
        T: Clone + Debug + Send + Sync + 'static,
    {
        type Output = T;

        fn provide<I>(
            &self,
            _injector: &I,
            _context: &CallContext,
        ) -> Result<Self::Output, InjectorError>
        where
            I: TypedInjector + ?Sized,
        {
            Ok(self.value.clone())
        }
    }

    impl<T> TypedSharedProvider for TestProvider<Arc<T>> where T: Debug + Send + Sync + 'static {}
}
