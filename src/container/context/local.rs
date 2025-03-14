use std::collections::HashSet;
use std::sync::Arc;

use parking_lot::Mutex;

use crate::container::context::SharedContext;
use crate::container::injector::{Injector, InjectorError};
use crate::container::registry::provider_map::ProviderEntry;
use crate::container::Managed;
use crate::key::Key;
use crate::provider::Provider;
use crate::scope::Scope;

pub struct LocalContext<S: Scope> {
    shared: Arc<SharedContext<S>>,
    managed: Mutex<LocalManagedObjectData>,
}

impl<S: Scope> LocalContext<S> {
    #[cfg_attr(not(test), expect(dead_code))]
    pub fn new(shared: Arc<SharedContext<S>>) -> Self {
        Self {
            shared,
            managed: Mutex::new(LocalManagedObjectData::new()),
        }
    }

    fn get_object(&self, key: &dyn Key) -> Result<Box<dyn Managed>, InjectorError> {
        match self.try_get_provider_by_key(key)? {
            ProviderEntry::Shared { .. } => self.shared.dyn_get(key),
            ProviderEntry::Owned { provider } => {
                self.get_object_from_current_context(provider.as_ref())
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
    ) -> Result<Box<dyn Managed>, InjectorError> {
        let key = provider.dyn_key();
        let mut managed = self.managed.lock();

        if managed.constructing.contains(key) {
            Err(InjectorError::CyclicDependency {
                key: key.dyn_clone(),
            })
        } else {
            managed.constructing.insert(key.dyn_clone());
            drop(managed);

            let res = provider.dyn_provide(self);

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

    use crate::component::Component;
    use crate::container::injector::TypedInjector;
    use crate::container::registry::provider_map::ProviderMap;
    use crate::key;
    use crate::provider::component::ComponentProvider;
    use crate::provider::instance::InstanceProvider;
    use crate::scope::SingletonScope;

    use super::*;

    struct TestObject {
        a: i32,
        b: &'static str,
    }

    impl Component for TestObject {
        type Output = Self;

        type Error = Infallible;

        fn construct<I>(injector: &I) -> Result<Result<Self, Self::Error>, InjectorError>
        where
            I: TypedInjector + ?Sized,
        {
            Ok(Ok(Self {
                a: injector.get(&key::of())?,
                b: injector.get(&key::of())?,
            }))
        }

        fn post_process(self) -> Self::Output {
            self
        }
    }

    struct RecursiveObject {
        _recursive: Box<RecursiveObject>,
    }

    impl Component for RecursiveObject {
        type Output = Box<Self>;

        type Error = Infallible;

        fn construct<I>(injector: &I) -> Result<Result<Self, Self::Error>, InjectorError>
        where
            I: TypedInjector + ?Sized,
        {
            Ok(Ok(Self {
                _recursive: injector.get(&key::of())?,
            }))
        }

        fn post_process(self) -> Self::Output {
            Box::new(self)
        }
    }

    #[test]
    fn local_context_get_succeeds() {
        let mut providers: ProviderMap<SingletonScope> = ProviderMap::new();
        providers.insert(Box::new(ComponentProvider::<_, TestObject>::new(key::of())));
        providers.insert(Box::new(InstanceProvider::new(key::of(), 42i32)));
        providers.insert(Box::new(InstanceProvider::new(key::of(), "str")));

        let root_context = SharedContext::new_root(Arc::new(providers));
        let local_context = LocalContext::new(Arc::new(root_context));

        let object: TestObject = local_context.get(&key::of()).unwrap();
        assert_eq!(object.a, 42i32);
        assert_eq!(object.b, "str");
    }

    #[test]
    fn local_context_get_succeeds_when_needing_non_transient_object() {
        let mut providers: ProviderMap<SingletonScope> = ProviderMap::new();
        providers.insert_shared(
            Box::new(InstanceProvider::new(key::of(), Arc::new(42i32))),
            SingletonScope,
        );

        let root_context = Arc::new(SharedContext::new_root(Arc::new(providers)));
        let local_context = LocalContext::new(Arc::clone(&root_context));

        let val: Arc<i32> = local_context.get(&key::of()).unwrap();
        assert_eq!(*val, 42i32);
    }

    #[test]
    fn local_context_get_fails_when_there_exists_cyclic_dependency() {
        let mut providers: ProviderMap<SingletonScope> = ProviderMap::new();
        providers.insert(Box::new(ComponentProvider::<_, RecursiveObject>::new(
            key::of(),
        )));

        let root_context = SharedContext::new_root(Arc::new(providers));
        let local_context = LocalContext::new(Arc::new(root_context));

        assert!(matches!(
            local_context.get(&key::of::<Box<RecursiveObject>>()),
            Err(InjectorError::CyclicDependency { .. })
        ));
    }

    #[test]
    fn local_context_get_fails_when_key_not_found() {
        let providers: ProviderMap<SingletonScope> = ProviderMap::new();
        let root_context = SharedContext::new_root(Arc::new(providers));
        let local_context = LocalContext::new(Arc::new(root_context));

        assert!(matches!(
            local_context.get(&key::of::<i32>()),
            Err(InjectorError::NotFound { .. })
        ));
    }
}
