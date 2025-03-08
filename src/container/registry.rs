use std::any::TypeId;
use std::borrow::Borrow;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::mem;

use snafu::prelude::*;

use crate::key::Key;
use crate::provider::{Provider, TypedProvider};

pub trait Registry: Send + Sync + 'static {
    fn dyn_register(&mut self, provider: Box<dyn Provider>) -> Result<(), RegistryError>;
}

pub trait TypedRegistry: Registry {
    fn register<P>(&mut self, provider: P) -> Result<(), RegistryError>
    where
        P: TypedProvider,
    {
        self.dyn_register(Box::new(provider))
    }
}

impl<T> TypedRegistry for T where T: Registry + ?Sized {}

#[derive(Debug, Snafu)]
#[non_exhaustive]
pub enum RegistryError {
    #[snafu(display("the key {key} already exists in the registry"))]
    #[non_exhaustive]
    KeyDuplicated { key: Box<dyn Key> },
}

impl Clone for RegistryError {
    fn clone(&self) -> Self {
        match self {
            Self::KeyDuplicated { key } => Self::KeyDuplicated {
                key: key.dyn_clone(),
            },
        }
    }
}

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

    pub fn get<K>(&self, key: &K) -> Option<&dyn Provider>
    where
        K: Borrow<dyn Key> + ?Sized,
    {
        let key: &dyn Key = key.borrow();
        self.providers
            .get(&key.target())
            .and_then(|slot| slot.get(&key))
    }
}

impl Registry for TypeSlotRegistry {
    fn dyn_register(&mut self, provider: Box<dyn Provider>) -> Result<(), RegistryError> {
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

#[derive(Debug)]
enum Slot {
    Singleton(Box<dyn Provider>),
    Map(HashMap<Box<dyn Key>, Box<dyn Provider>>),
}

impl Slot {
    fn insert(&mut self, provider: Box<dyn Provider>) -> Option<Box<dyn Provider>> {
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

    fn get<K>(&self, key: &K) -> Option<&dyn Provider>
    where
        K: Borrow<dyn Key>,
    {
        match self {
            Self::Singleton(entry) if entry.dyn_key() != key.borrow() => None,
            Self::Singleton(entry) => Some(&**entry),
            Self::Map(entries) => entries.get(key.borrow()).map(AsRef::as_ref),
        }
    }
}

impl From<Box<dyn Provider>> for Slot {
    fn from(provider: Box<dyn Provider>) -> Self {
        Self::Singleton(provider)
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use crate::container::injector::{InjectorError, MockInjector, TypedInjector};
    use crate::key::{self, KeyImpl};
    use crate::util::any::Downcast;

    use super::*;

    #[test]
    fn type_slot_registry_register_succeeds() {
        let mut registry = TypeSlotRegistry::new();
        assert!(registry.register(TestProvider::new(42i32)).is_ok());
        assert!(registry.register(TestProvider::new("str")).is_ok());
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
    fn type_slot_registry_get_succeeds() {
        let mut registry = TypeSlotRegistry::new();
        assert!(registry.register(TestProvider::new(42i32)).is_ok());

        let provider = registry.get(&key::of::<i32>()).unwrap();
        assert_eq!(provider.dyn_key(), &key::of::<i32>() as &dyn Key);
        let res = provider.dyn_provide(&mut MockInjector::new()).unwrap();
        assert_eq!(*res.downcast::<i32>().unwrap_or(Box::new(0)), 42);

        assert!(registry.get(&key::of::<&str>()).is_none());
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
}
