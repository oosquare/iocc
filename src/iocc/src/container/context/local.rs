use std::any::TypeId;
use std::collections::HashSet;
use std::sync::Arc;

use parking_lot::Mutex;

use crate::container::context::SharedContext;
use crate::container::injector::{Injector, InjectorError};
use crate::container::registry::ProviderEntry;
use crate::container::Managed;
use crate::key::Key;
use crate::provider::Provider;
use crate::provider::context::CallContext;
use crate::scope::Scope;

pub struct LocalContext<S: Scope> {
    shared: Arc<SharedContext<S>>,
    managed: Mutex<LocalManagedObjectData>,
}

impl<S: Scope> LocalContext<S> {
    pub fn new(shared: Arc<SharedContext<S>>) -> Self {
        Self {
            shared,
            managed: Mutex::new(LocalManagedObjectData::new()),
        }
    }

    pub fn shared(&self) -> Arc<SharedContext<S>> {
        Arc::clone(&self.shared)
    }

    pub fn shared_ref(&self) -> &SharedContext<S> {
        &self.shared
    }

    pub fn current_scope(&self) -> S {
        self.shared.current_scope()
    }

    fn get_object(&self, key: &dyn Key) -> Result<Box<dyn Managed>, InjectorError> {
        match self.try_get_provider_by_key(key)? {
            ProviderEntry::Shared { .. } => self.shared.dyn_get(key),
            ProviderEntry::Owned { provider, .. } => {
                self.get_object_from_current_context(provider.as_ref(), key)
            }
        }
    }

    fn try_get_provider_by_key(&self, key: &dyn Key) -> Result<&ProviderEntry<S>, InjectorError> {
        if let Some(provider) = self.shared.providers().get(key) {
            Ok(provider)
        } else {
            Err(InjectorError::NotFound {
                key: key.dyn_clone(),
            })
        }
    }

    fn get_object_from_current_context(
        &self,
        provider: &dyn Provider,
        key: &dyn Key,
    ) -> Result<Box<dyn Managed>, InjectorError> {
        let mut managed = self.managed.lock();

        if managed.constructing.contains(key) {
            Err(InjectorError::CyclicDependency {
                key: key.dyn_clone(),
            })
        } else {
            managed.constructing.insert(key.dyn_clone());
            drop(managed);

            let context = CallContext::new(key);
            let res = provider.dyn_provide(self, &context);

            let mut managed = self.managed.lock();
            managed.constructing.remove(key);

            res
        }
    }
}

impl<S: Scope> Injector for LocalContext<S> {
    fn dyn_get(&self, key: &dyn Key) -> Result<Box<dyn Managed>, InjectorError> {
        self.get_object(key)
    }

    fn keys(&self, type_id: TypeId) -> Vec<Box<dyn Key>> {
        self.shared.keys(type_id)
    }
}

struct LocalManagedObjectData {
    constructing: HashSet<Box<dyn Key>>,
}

impl LocalManagedObjectData {
    fn new() -> Self {
        Self {
            constructing: HashSet::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;

    use crate::container::injector::TypedInjector;
    use crate::container::registry::ProviderMap;
    use crate::key;
    use crate::provider::component::{Component, ComponentProvider};
    use crate::provider::instance::InstanceProvider;
    use crate::scope::SingletonScope;

    use super::*;

    struct TestObject {
        a: i32,
        b: &'static str,
    }

    impl Component for TestObject {
        type Constructed = Self;

        type Error = Infallible;

        fn construct<I>(injector: &I) -> Result<Result<Self, Self::Error>, InjectorError>
        where
            I: TypedInjector + ?Sized,
        {
            Ok(Ok(Self {
                a: injector.get(key::of())?,
                b: injector.get(key::of())?,
            }))
        }

        fn post_process(self) -> Self::Constructed {
            self
        }
    }

    struct RecursiveObject {
        _recursive: Box<RecursiveObject>,
    }

    impl Component for RecursiveObject {
        type Constructed = Box<Self>;

        type Error = Infallible;

        fn construct<I>(injector: &I) -> Result<Result<Self, Self::Error>, InjectorError>
        where
            I: TypedInjector + ?Sized,
        {
            Ok(Ok(Self {
                _recursive: injector.get(key::of())?,
            }))
        }

        fn post_process(self) -> Self::Constructed {
            Box::new(self)
        }
    }

    #[test]
    fn local_context_get_succeeds() {
        let mut providers: ProviderMap<SingletonScope> = ProviderMap::new();
        providers.insert(
            Box::new(key::of::<TestObject>()),
            Box::new(ComponentProvider::<TestObject>::new()),
        );
        providers.insert(
            Box::new(key::of::<i32>()),
            Box::new(InstanceProvider::new(42i32)),
        );
        providers.insert(
            Box::new(key::of::<&'static str>()),
            Box::new(InstanceProvider::new("str")),
        );

        let root_context = SharedContext::new_root(Arc::new(providers));
        let local_context = LocalContext::new(Arc::new(root_context));

        let object: TestObject = local_context.get(key::of()).unwrap();
        assert_eq!(object.a, 42i32);
        assert_eq!(object.b, "str");
    }

    #[test]
    fn local_context_get_succeeds_when_needing_non_transient_object() {
        let mut providers: ProviderMap<SingletonScope> = ProviderMap::new();
        providers.insert_shared(
            Box::new(key::of::<Arc<i32>>()),
            Box::new(InstanceProvider::new(Arc::new(42i32))),
            SingletonScope,
        );

        let root_context = Arc::new(SharedContext::new_root(Arc::new(providers)));
        let local_context = LocalContext::new(Arc::clone(&root_context));

        let val: Arc<i32> = local_context.get(key::of()).unwrap();
        assert_eq!(*val, 42i32);
    }

    #[test]
    fn local_context_get_fails_when_there_exists_cyclic_dependency() {
        let mut providers: ProviderMap<SingletonScope> = ProviderMap::new();
        providers.insert(
            Box::new(key::of::<Box<RecursiveObject>>()),
            Box::new(ComponentProvider::<RecursiveObject>::new()),
        );

        let root_context = SharedContext::new_root(Arc::new(providers));
        let local_context = LocalContext::new(Arc::new(root_context));

        assert!(matches!(
            local_context.get(key::of::<Box<RecursiveObject>>()),
            Err(InjectorError::CyclicDependency { .. })
        ));
    }

    #[test]
    fn local_context_get_fails_when_key_not_found() {
        let providers: ProviderMap<SingletonScope> = ProviderMap::new();
        let root_context = SharedContext::new_root(Arc::new(providers));
        let local_context = LocalContext::new(Arc::new(root_context));

        assert!(matches!(
            local_context.get(key::of::<i32>()),
            Err(InjectorError::NotFound { .. })
        ));
    }
}
