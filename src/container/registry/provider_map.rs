use std::any::TypeId;
use std::collections::HashMap;
use std::mem;

use crate::key::Key;
use crate::provider::{Provider, SharedProvider};

#[derive(Debug)]
pub struct ProviderMap {
    providers: HashMap<TypeId, ProviderSlot>,
}

impl ProviderMap {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    pub fn insert(&mut self, provider: Box<dyn Provider>) -> Option<ProviderEntry> {
        self.insert_impl(provider.into())
    }

    pub fn insert_shared(&mut self, provider: Box<dyn SharedProvider>) -> Option<ProviderEntry> {
        self.insert_impl(provider.into())
    }

    pub fn move_out(&mut self, key: &dyn Key) -> Option<ProviderEntry> {
        self.providers
            .get_mut(&key.target())
            .and_then(|slot| slot.get(key))
            .map(|entry| mem::replace(entry, ProviderEntry::TemporaryMoved(key.dyn_clone())))
    }

    pub fn get(&mut self, key: &dyn Key) -> Option<&mut ProviderEntry> {
        self.providers
            .get_mut(&key.target())
            .and_then(|slot| slot.get(key))
    }

    fn insert_impl(&mut self, provider: ProviderEntry) -> Option<ProviderEntry> {
        let target = provider.dyn_key().target();
        if let Some(slot) = self.providers.get_mut(&target) {
            slot.insert(provider)
        } else {
            self.providers.insert(target, provider.into());
            None
        }
    }
}

#[derive(Debug)]
enum ProviderSlot {
    Singleton(ProviderEntry),
    Map(HashMap<Box<dyn Key>, ProviderEntry>),
}

impl ProviderSlot {
    fn insert(&mut self, provider: ProviderEntry) -> Option<ProviderEntry> {
        match self {
            Self::Singleton(entry) if entry.dyn_key() == provider.dyn_key() => {
                let original = mem::replace(entry, provider);
                Some(original)
            }
            Self::Singleton(_) => {
                let Self::Singleton(entry) =
                    mem::replace(self, Self::Map(HashMap::with_capacity(2)))
                else {
                    unreachable!("`self` should match `Self::Singleton(_)``")
                };
                let Self::Map(entries) = self else {
                    unreachable!("`self` should already be assigned to `Self::Map(_)``")
                };
                entries.insert(entry.dyn_key().dyn_clone(), entry);
                entries.insert(provider.dyn_key().dyn_clone(), provider);
                None
            }
            Self::Map(entries) => entries.insert(provider.dyn_key().dyn_clone(), provider),
        }
    }

    fn get(&mut self, key: &dyn Key) -> Option<&mut ProviderEntry> {
        match self {
            Self::Singleton(entry) if entry.dyn_key() != key => None,
            Self::Singleton(entry) => Some(entry),
            Self::Map(entries) => entries.get_mut(key),
        }
    }
}

impl From<ProviderEntry> for ProviderSlot {
    fn from(provider: ProviderEntry) -> Self {
        Self::Singleton(provider)
    }
}

#[derive(Debug)]
pub enum ProviderEntry {
    Shared(Box<dyn SharedProvider>),
    Owned(Box<dyn Provider>),
    TemporaryMoved(Box<dyn Key>),
}

impl ProviderEntry {
    pub fn dyn_key(&self) -> &dyn Key {
        match self {
            Self::TemporaryMoved(k) => k.as_ref(),
            Self::Shared(s) => s.dyn_key(),
            Self::Owned(s) => s.dyn_key(),
        }
    }

    #[cfg(test)]
    pub fn as_shared(&mut self) -> Option<&mut dyn SharedProvider> {
        if let Self::Shared(v) = self {
            Some(v.as_mut())
        } else {
            None
        }
    }
}

impl From<Box<dyn Provider>> for ProviderEntry {
    fn from(provider: Box<dyn Provider>) -> Self {
        ProviderEntry::Owned(provider)
    }
}

impl From<Box<dyn SharedProvider>> for ProviderEntry {
    fn from(provider: Box<dyn SharedProvider>) -> Self {
        ProviderEntry::Shared(provider)
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::sync::Arc;

    use crate::container::injector::{InjectorError, MockInjector, TypedInjector};
    use crate::key::{self, KeyImpl};
    use crate::provider::{TypedProvider, TypedSharedProvider};
    use crate::util::any::Downcast;

    use super::*;

    #[test]
    fn type_slot_registry_register_succeeds() {
        let mut registry = ProviderMap::new();
        assert!(registry
            .insert(Box::new(TestProvider::new(42i32)))
            .is_none());
        assert!(registry
            .insert(Box::new(TestProvider::new("str")))
            .is_none());
        assert!(registry
            .insert(Box::new(TestProvider::new(42i32)))
            .is_some());
        assert!(registry
            .insert(Box::new(TestProvider::new("str")))
            .is_some());
    }

    #[test]
    fn type_slot_registry_register_shared_succeeds() {
        let mut registry = ProviderMap::new();
        assert!(registry
            .insert_shared(Box::new(TestProvider::new(Arc::new(42i32))))
            .is_none());
        assert!(registry
            .insert_shared(Box::new(TestProvider::new(Arc::new("str"))))
            .is_none());
        assert!(registry
            .insert_shared(Box::new(TestProvider::new(Arc::new(42i32))))
            .is_some());
        assert!(registry
            .insert_shared(Box::new(TestProvider::new(Arc::new("str"))))
            .is_some());
    }

    #[test]
    fn type_slot_registry_get_succeeds_when_provider_is_shared() {
        let mut registry = ProviderMap::new();
        assert!(registry
            .insert_shared(Box::new(TestProvider::new(Arc::new(42i32))))
            .is_none());

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

        fn provide<I>(&mut self, _injector: &mut I) -> Result<Self::Output, InjectorError>
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
