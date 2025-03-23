use std::any::TypeId;
use std::collections::HashMap;
use std::mem;

use crate::key::Key;
use crate::provider::{Provider, SharedProvider};
use crate::scope::Scope;

#[derive(Debug)]
pub struct ProviderMap<S: Scope> {
    providers: HashMap<TypeId, ProviderSlot<S>>,
}

impl<S: Scope> ProviderMap<S> {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    pub fn insert(
        &mut self,
        key: Box<dyn Key>,
        provider: Box<dyn Provider>,
    ) -> Option<ProviderEntry<S>> {
        self.insert_impl(ProviderEntry::new_owned(key, provider))
    }

    pub fn insert_shared(
        &mut self,
        key: Box<dyn Key>,
        provider: Box<dyn SharedProvider>,
        scope: S,
    ) -> Option<ProviderEntry<S>> {
        self.insert_impl(ProviderEntry::new_shared(key, provider, scope))
    }

    pub fn get(&self, key: &dyn Key) -> Option<&ProviderEntry<S>> {
        self.providers
            .get(&key.target_type())
            .and_then(|slot| slot.get(key))
    }

    fn insert_impl(&mut self, provider: ProviderEntry<S>) -> Option<ProviderEntry<S>> {
        let target = provider.dyn_key().target_type();
        if let Some(slot) = self.providers.get_mut(&target) {
            slot.insert(provider)
        } else {
            self.providers.insert(target, provider.into());
            None
        }
    }

    pub fn keys(&self, type_id: TypeId) -> Vec<Box<dyn Key>> {
        self.providers
            .get(&type_id)
            .map_or(Vec::new(), |slot| slot.keys())
    }
}

#[derive(Debug)]
enum ProviderSlot<S: Scope> {
    Singleton(ProviderEntry<S>),
    Map(HashMap<Box<dyn Key>, ProviderEntry<S>>),
}

impl<S: Scope> ProviderSlot<S> {
    fn insert(&mut self, provider: ProviderEntry<S>) -> Option<ProviderEntry<S>> {
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

    fn get(&self, key: &dyn Key) -> Option<&ProviderEntry<S>> {
        match self {
            Self::Singleton(entry) if entry.dyn_key() != key => None,
            Self::Singleton(entry) => Some(entry),
            Self::Map(entries) => entries.get(key),
        }
    }

    fn keys(&self) -> Vec<Box<dyn Key>> {
        match self {
            Self::Singleton(entry) => vec![entry.dyn_key().dyn_clone()],
            Self::Map(entries) => entries.keys().map(|key| key.dyn_clone()).collect(),
        }
    }
}

impl<S: Scope> From<ProviderEntry<S>> for ProviderSlot<S> {
    fn from(provider: ProviderEntry<S>) -> Self {
        Self::Singleton(provider)
    }
}

#[derive(Debug)]
pub enum ProviderEntry<S: Scope> {
    Shared {
        key: Box<dyn Key>,
        provider: Box<dyn SharedProvider>,
        scope: S,
    },
    Owned {
        key: Box<dyn Key>,
        provider: Box<dyn Provider>,
    },
}

impl<S: Scope> ProviderEntry<S> {
    pub fn new_shared(key: Box<dyn Key>, provider: Box<dyn SharedProvider>, scope: S) -> Self {
        Self::Shared {
            key,
            provider,
            scope,
        }
    }

    pub fn new_owned(key: Box<dyn Key>, provider: Box<dyn Provider>) -> Self {
        Self::Owned { key, provider }
    }

    pub fn dyn_key(&self) -> &dyn Key {
        match self {
            Self::Shared { key, .. } => key.as_ref(),
            Self::Owned { key, .. } => key.as_ref(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::sync::Arc;

    use crate::container::injector::{CallContext, InjectorError, TypedInjector};
    use crate::key;
    use crate::provider::{TypedProvider, TypedSharedProvider};
    use crate::scope::SingletonScope;

    use super::*;

    #[test]
    fn type_slot_registry_register_succeeds() {
        let mut registry: ProviderMap<SingletonScope> = ProviderMap::new();

        let key = Box::new(key::of::<i32>());
        let provider = Box::new(TestProvider::new(42i32));
        assert!(registry.insert(key, provider).is_none());

        let key = Box::new(key::of::<&'static str>());
        let provider = Box::new(TestProvider::new("str"));
        assert!(registry.insert(key, provider).is_none());

        let key = Box::new(key::of::<i32>());
        let provider = Box::new(TestProvider::new(42i32));
        assert!(registry.insert(key, provider).is_some());

        let key = Box::new(key::of::<&'static str>());
        let provider = Box::new(TestProvider::new("str"));
        assert!(registry.insert(key, provider).is_some());
    }

    #[test]
    fn type_slot_registry_register_shared_succeeds() {
        let mut registry = ProviderMap::new();
        let scope = SingletonScope;

        let key = Box::new(key::of::<Arc<i32>>());
        let provider = Box::new(TestProvider::new(Arc::new(42i32)));
        assert!(registry.insert_shared(key, provider, scope).is_none());

        let key = Box::new(key::of::<Arc<&'static str>>());
        let provider = Box::new(TestProvider::new(Arc::new("str")));
        assert!(registry.insert_shared(key, provider, scope).is_none());

        let key = Box::new(key::of::<Arc<i32>>());
        let provider = Box::new(TestProvider::new(Arc::new(42i32)));
        assert!(registry.insert_shared(key, provider, scope).is_some());

        let key = Box::new(key::of::<Arc<&'static str>>());
        let provider = Box::new(TestProvider::new(Arc::new("str")));
        assert!(registry.insert_shared(key, provider, scope).is_some());
    }

    #[test]
    fn type_slot_registry_get_succeeds_when_provider_is_shared() {
        let mut registry = ProviderMap::new();

        let key = Box::new(key::of::<Arc<i32>>());
        let provider = Box::new(TestProvider::new(Arc::new(42i32)));
        let scope = SingletonScope;
        assert!(registry.insert_shared(key, provider, scope).is_none());

        let provider = registry.get(&key::of::<Arc<i32>>()).unwrap();
        assert_eq!(provider.dyn_key(), &key::of::<Arc<i32>>() as &dyn Key);
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
