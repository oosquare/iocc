use std::any::TypeId;
use std::borrow::Borrow;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::mem;

use crate::container::registry::inner::{InnerRegistry, VarProvider};
use crate::container::registry::RegistryError;
use crate::key::Key;
use crate::provider::{Provider, SharedProvider};

#[derive(Debug)]
pub struct TypeSlotRegistry {
    providers: HashMap<TypeId, Slot>,
}

impl TypeSlotRegistry {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    fn register_impl(&mut self, provider: VarProvider) -> Result<(), RegistryError> {
        match self.providers.entry(provider.dyn_key().target()) {
            Entry::Vacant(vaccant) => {
                vaccant.insert(provider.into());
                Ok(())
            }
            Entry::Occupied(mut occupied) => match occupied.get_mut().insert(provider) {
                Some(provider) => Err(RegistryError::KeyDuplicated {
                    key: provider.dyn_key().dyn_clone(),
                }),
                None => Ok(()),
            },
        }
    }
}

impl InnerRegistry for TypeSlotRegistry {
    fn dyn_register(&mut self, provider: Box<dyn Provider>) -> Result<(), RegistryError> {
        self.register_impl(provider.into())
    }

    fn dyn_register_shared(
        &mut self,
        provider: Box<dyn SharedProvider>,
    ) -> Result<(), RegistryError> {
        self.register_impl(provider.into())
    }

    fn get(&self, key: &dyn Key) -> Option<&VarProvider> {
        self.providers
            .get(&key.target())
            .and_then(|slot| slot.get(&key))
    }
}

#[derive(Debug)]
enum Slot {
    Singleton(VarProvider),
    Map(HashMap<Box<dyn Key>, VarProvider>),
}

impl Slot {
    fn insert(&mut self, provider: VarProvider) -> Option<VarProvider> {
        match self {
            Self::Singleton(entry) if entry.dyn_key() == provider.dyn_key() => {
                let original = mem::replace(entry, provider);
                Some(original)
            }
            Self::Singleton(_) => {
                let Self::Singleton(entry) =
                    mem::replace(self, Self::Map(HashMap::with_capacity(2)))
                else {
                    unreachable!("`self` should match Self::Singleton(_)")
                };
                let Self::Map(entries) = self else {
                    unreachable!("`self` should already be assigned to Self::Map(_)")
                };
                entries.insert(entry.dyn_key().dyn_clone(), entry);
                entries.insert(provider.dyn_key().dyn_clone(), provider);
                None
            }
            Self::Map(entries) => entries.insert(provider.dyn_key().dyn_clone(), provider),
        }
    }

    fn get<K>(&self, key: &K) -> Option<&VarProvider>
    where
        K: Borrow<dyn Key>,
    {
        match self {
            Self::Singleton(entry) if entry.dyn_key() != key.borrow() => None,
            Self::Singleton(entry) => Some(entry),
            Self::Map(entries) => entries.get(key.borrow()),
        }
    }
}

impl From<VarProvider> for Slot {
    fn from(provider: VarProvider) -> Self {
        Self::Singleton(provider)
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::sync::Arc;

    use crate::container::injector::{InjectorError, MockInjector, TypedInjector};
    use crate::container::registry::inner::TypedInnerRegistry;
    use crate::key::{self, KeyImpl};
    use crate::provider::{TypedProvider, TypedSharedProvider};
    use crate::util::any::Downcast;

    use super::*;

    #[test]
    fn type_slot_registry_register_succeeds() {
        let mut registry = TypeSlotRegistry::new();
        assert!(registry.register(TestProvider::new(42i32)).is_ok());
        assert!(registry.register(TestProvider::new("str")).is_ok());
    }

    #[test]
    fn type_slot_registry_register_shared_succeeds() {
        let mut registry = TypeSlotRegistry::new();
        assert!(registry
            .register_shared(TestProvider::new(Arc::new(42i32)))
            .is_ok());
        assert!(registry
            .register_shared(TestProvider::new(Arc::new("str")))
            .is_ok());
    }

    #[test]
    fn type_slot_registry_register_fails_when_key_is_duplicated() {
        let mut registry = TypeSlotRegistry::new();
        assert!(registry.register(TestProvider::new(42i32)).is_ok());
        assert!(matches!(
            registry.register(TestProvider::new(43i32)),
            Err(RegistryError::KeyDuplicated { .. })
        ));
    }

    #[test]
    fn type_slot_registry_get_succeeds_when_provider_is_owned() {
        let mut registry = TypeSlotRegistry::new();
        assert!(registry.register(TestProvider::new(42i32)).is_ok());

        let provider = registry.get(&key::of::<i32>()).unwrap();
        assert_eq!(provider.dyn_key(), &key::of::<i32>() as &dyn Key);
        let res = provider
            .as_owned()
            .unwrap()
            .dyn_provide(&mut MockInjector::new())
            .unwrap();
        assert_eq!(*res.downcast::<i32>().unwrap_or(Box::new(0)), 42);
    }

    #[test]
    fn type_slot_registry_get_succeeds_when_provider_is_shared() {
        let mut registry = TypeSlotRegistry::new();
        assert!(registry
            .register_shared(TestProvider::new(Arc::new(42i32)))
            .is_ok());

        let provider = registry.get(&key::of::<Arc<i32>>()).unwrap();
        assert_eq!(provider.dyn_key(), &key::of::<Arc<i32>>() as &dyn Key);
        let res = provider
            .as_shared()
            .unwrap()
            .dyn_provide(&mut MockInjector::new())
            .unwrap();
        assert_eq!(
            **res.downcast::<Arc<i32>>().unwrap_or(Box::new(Arc::new(0))),
            42
        );
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

        fn provide<I>(&self, _injector: &mut I) -> Result<Self::Output, InjectorError>
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
