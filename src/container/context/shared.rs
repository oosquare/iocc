use std::collections::HashMap;
use std::sync::{Arc, RwLock, RwLockWriteGuard};
use std::thread::{self, ThreadId};

use oneshot::{Receiver, Sender};

use crate::container::injector::object_map::ObjectMap;
use crate::container::injector::{Injector, InjectorError};
use crate::container::registry::provider_map::{ProviderEntry, ProviderMap};
use crate::container::Managed;
use crate::key::Key;
use crate::provider::SharedProvider;
use crate::scope::Scope;

pub struct SharedContext<S: Scope> {
    parent: Option<Arc<Self>>,
    providers: Arc<ProviderMap<S>>,
    managed: RwLock<SharedManagedObjectData>,
    scope: S,
}

impl<S: Scope> SharedContext<S> {
    #[cfg_attr(not(test), expect(dead_code))]
    pub fn new_root(providers: Arc<ProviderMap<S>>) -> Self {
        Self::new_impl(None, providers, S::SINGLETON)
    }

    #[cfg_attr(not(test), expect(dead_code))]
    pub fn new_sub(parent: Arc<Self>) -> Option<Self> {
        if let Some(scope) = parent.scope.sub_scope() {
            let providers = Arc::clone(&parent.providers);
            Some(Self::new_impl(Some(parent), providers, scope))
        } else {
            None
        }
    }

    fn new_impl(parent: Option<Arc<Self>>, providers: Arc<ProviderMap<S>>, scope: S) -> Self {
        Self {
            parent,
            providers,
            managed: RwLock::new(SharedManagedObjectData::new()),
            scope,
        }
    }

    #[expect(dead_code)]
    pub fn scope(&self) -> S {
        self.scope
    }

    pub fn providers(&self) -> &ProviderMap<S> {
        self.providers.as_ref()
    }

    fn get_object(&self, key: &dyn Key) -> Result<Box<dyn Managed>, InjectorError> {
        if let Some(object) = self.try_get_constructed_object(key) {
            return Ok(object);
        }

        match self.try_get_provider_by_key(key)? {
            p @ ProviderEntry::Shared { provider, scope } => {
                if self.should_forward_request_to_parent(*scope) {
                    self.get_object_from_parent(key)
                } else if *scope == self.scope {
                    self.get_object_from_current_context(provider.as_ref())
                } else {
                    Err(InjectorError::ShortLifetime {
                        key: key.dyn_clone(),
                        lifetime: p.lifetime().to_str(),
                        scope: self.scope.to_str(),
                    })
                }
            }
            p @ ProviderEntry::Owned { .. } => Err(InjectorError::ShortLifetime {
                key: key.dyn_clone(),
                lifetime: p.lifetime().to_str(),
                scope: self.scope.to_str(),
            }),
        }
    }

    fn try_get_constructed_object(&self, key: &dyn Key) -> Option<Box<dyn Managed>> {
        let objects = &self.managed.read().unwrap().objects;
        objects.get(key).map(|entry| entry.clone_managed())
    }

    fn try_get_provider_by_key(&self, key: &dyn Key) -> Result<&ProviderEntry<S>, InjectorError> {
        if let Some(provider) = self.providers.get(key) {
            Ok(provider)
        } else {
            Err(InjectorError::NotFound {
                key: key.dyn_clone(),
            })
        }
    }

    fn should_forward_request_to_parent(&self, object_scope: S) -> bool {
        // The object's scope strictly outlives the current scope.
        object_scope.outlive(self.scope) && object_scope != self.scope
    }

    fn get_object_from_parent(&self, key: &dyn Key) -> Result<Box<dyn Managed>, InjectorError> {
        if let Some(parent) = self.parent.as_ref() {
            parent.dyn_get(key)
        } else {
            // If the parent context doesn't exist, `self` must be a root
            // context whose scope is `S::SINGLETON`, and no other scope
            // strictly outlives `S::SINGLETON`, so any object can be
            // constructed in this context, thus making forwarding unnecessary.
            unreachable!("Parent context should exist")
        }
    }

    fn get_object_from_current_context(
        &self,
        provider: &dyn SharedProvider,
    ) -> Result<Box<dyn Managed>, InjectorError> {
        let key = provider.dyn_key();
        let mut managed = self.managed.write().unwrap();

        if let Some(context) = managed.constructing.get_mut(key) {
            if context.is_constructed_by_current_thread() {
                Err(self.stop_construction_on_cyclic_dependency(managed, key))
            } else {
                self.wait_for_constructed_object(managed, key)
            }
        } else {
            self.construct_object(managed, provider, key)
        }
    }

    fn stop_construction_on_cyclic_dependency(
        &self,
        managed: RwLockWriteGuard<SharedManagedObjectData>,
        key: &dyn Key,
    ) -> InjectorError {
        let err = InjectorError::CyclicDependency {
            key: key.dyn_clone(),
        };
        let response = WaitResponse::Error(err.clone());
        self.notify_waiters(managed, key, response);
        err
    }

    fn wait_for_constructed_object(
        &self,
        managed: RwLockWriteGuard<SharedManagedObjectData>,
        key: &dyn Key,
    ) -> Result<Box<dyn Managed>, InjectorError> {
        let receiver = self.register_waiter_on_object_context(managed, key);
        self.get_object_on_object_context_response(receiver, key)
    }

    fn register_waiter_on_object_context(
        &self,
        mut managed: RwLockWriteGuard<SharedManagedObjectData>,
        key: &dyn Key,
    ) -> Receiver<WaitResponse> {
        let (sender, receiver) = oneshot::channel();
        let Some(context) = managed.constructing.get_mut(key) else {
            unreachable!("whether `context` exists should be checked before calling this method")
        };
        context.register_waiter(sender);
        receiver
    }

    fn get_object_on_object_context_response(
        &self,
        receiver: Receiver<WaitResponse>,
        key: &dyn Key,
    ) -> Result<Box<dyn Managed>, InjectorError> {
        match receiver.recv() {
            Ok(WaitResponse::Constructed) => {
                let managed = self.managed.read().unwrap();
                let Some(object) = managed.objects.get(key) else {
                    unreachable!("`object` should already be put into `self.managed.objects`")
                };
                Ok(object.clone_managed())
            }
            Ok(WaitResponse::Error(err)) => Err(err),
            Err(_) => unreachable!("the peer should send a message"),
        }
    }

    fn construct_object(
        &self,
        mut managed: RwLockWriteGuard<SharedManagedObjectData>,
        provider: &dyn SharedProvider,
        key: &dyn Key,
    ) -> Result<Box<dyn Managed>, InjectorError> {
        let on_thread = thread::current().id();
        let object_context = ConstructingObjectContext::new(on_thread);
        managed.constructing.insert(key.dyn_clone(), object_context);
        drop(managed);

        match provider.dyn_provide_shared(self) {
            Ok(object) => {
                let mut managed = self.managed.write().unwrap();
                managed.objects.insert(key.dyn_clone(), object.dyn_clone());
                self.notify_waiters(managed, key, WaitResponse::Constructed);
                Ok(object.upcast_managed())
            }
            Err(err) => {
                let managed = self.managed.write().unwrap();
                self.notify_waiters(managed, key, WaitResponse::Error(err.clone()));
                Err(err)
            }
        }
    }

    fn notify_waiters(
        &self,
        mut managed: RwLockWriteGuard<SharedManagedObjectData>,
        key: &dyn Key,
        response: WaitResponse,
    ) {
        if let Some(context) = managed.constructing.remove(key) {
            drop(managed);
            context.notify(response);
        }
    }
}

impl<S: Scope> Injector for SharedContext<S> {
    fn dyn_get(&self, key: &dyn Key) -> Result<Box<dyn Managed>, InjectorError> {
        self.get_object(key)
    }
}

struct SharedManagedObjectData {
    objects: ObjectMap,
    constructing: HashMap<Box<dyn Key>, ConstructingObjectContext>,
}

impl SharedManagedObjectData {
    fn new() -> Self {
        Self {
            objects: ObjectMap::new(),
            constructing: HashMap::new(),
        }
    }
}

struct ConstructingObjectContext {
    on_thread: ThreadId,
    waiters: Vec<Sender<WaitResponse>>,
}

impl ConstructingObjectContext {
    fn new(on_thread: ThreadId) -> Self {
        Self {
            on_thread,
            waiters: Vec::new(),
        }
    }

    fn is_constructed_by_current_thread(&self) -> bool {
        thread::current().id() == self.on_thread
    }

    fn register_waiter(&mut self, sender: Sender<WaitResponse>) {
        self.waiters.push(sender);
    }

    fn notify(self, response: WaitResponse) {
        for sender in self.waiters {
            let _ = sender.send(response.clone());
        }
    }
}

#[derive(Debug, Clone)]
enum WaitResponse {
    Constructed,
    Error(InjectorError),
}

#[cfg(test)]
mod tests {
    use crate::container::injector::TypedInjector;
    use crate::key;
    use crate::provider::closure::ClosureProvider;
    use crate::provider::instance::InstanceProvider;
    use crate::scope::WebScope;

    use super::*;

    struct TestObject {
        id: u32,
        sub_even: Option<Arc<TestObject>>,
        sub_odd: Option<Arc<TestObject>>,
    }

    impl TestObject {
        fn get_provider(id: u32) -> Box<dyn SharedProvider> {
            Box::new(ClosureProvider::new(key::qualified(id), move |injector| {
                if id <= 1 {
                    Ok(Arc::new(TestObject {
                        id,
                        sub_even: None,
                        sub_odd: None,
                    }))
                } else if id % 2 == 0 {
                    Ok(Arc::new(TestObject {
                        id,
                        sub_even: Some(injector.get(&key::qualified(id - 2))?),
                        sub_odd: Some(injector.get(&key::qualified(id - 1))?),
                    }))
                } else {
                    Ok(Arc::new(TestObject {
                        id,
                        sub_even: Some(injector.get(&key::qualified(id - 3))?),
                        sub_odd: Some(injector.get(&key::qualified(id - 2))?),
                    }))
                }
            }))
        }
    }

    struct RecursiveObject {
        _recursive: Arc<RecursiveObject>,
    }

    impl RecursiveObject {
        fn get_provider() -> Box<dyn SharedProvider> {
            Box::new(ClosureProvider::new(key::of(), move |injector| {
                Ok(Arc::new(RecursiveObject {
                    _recursive: injector.get(&key::of())?,
                }))
            }))
        }
    }

    #[test]
    fn shared_context_get_succeeds_when_lifetime_outlives_current_scope() {
        let mut providers = ProviderMap::new();
        providers.insert_shared(TestObject::get_provider(0u32), WebScope::Singleton);

        let root_context = Arc::new(SharedContext::new_root(Arc::new(providers)));
        let sub_context = Arc::new(SharedContext::new_sub(Arc::clone(&root_context)).unwrap());
        let key = key::qualified::<Arc<TestObject>, _>(0u32);

        let _ = sub_context.get(&key).unwrap();
        let object = sub_context.get(&key).unwrap();
        assert_eq!(object.id, 0u32);

        let managed = root_context.managed.read().unwrap();
        assert!(managed.objects.get(&key).is_some());

        let managed = sub_context.managed.read().unwrap();
        assert!(managed.objects.get(&key).is_none());
    }

    #[test]
    fn shared_context_get_succeeds_when_it_needs_complex_structure() {
        const NUM: u32 = 100;
        let mut providers = ProviderMap::new();

        for i in 0..NUM {
            providers.insert_shared(TestObject::get_provider(2 * i), WebScope::Singleton);
            providers.insert_shared(TestObject::get_provider(2 * i + 1), WebScope::Singleton);
        }

        let context = Arc::new(SharedContext::new_root(Arc::new(providers)));

        for i in (0..NUM).rev() {
            let ctx = Arc::clone(&context);
            thread::spawn(move || {
                let object: Arc<TestObject> = ctx.get(&key::qualified(2 * i)).unwrap();
                assert_eq!(object.id, 2 * i);
                assert!(object
                    .sub_even
                    .as_ref()
                    .is_none_or(|object| object.id == 2 * (i - 1)));
                assert!(object
                    .sub_odd
                    .as_ref()
                    .is_none_or(|object| object.id == 2 * (i - 1) + 1));
            });
            let ctx = Arc::clone(&context);
            thread::spawn(move || {
                let object: Arc<TestObject> = ctx.get(&key::qualified(2 * i + 1)).unwrap();
                assert_eq!(object.id, 2 * i + 1);
                assert!(object
                    .sub_even
                    .as_ref()
                    .is_none_or(|object| object.id == 2 * (i - 1)));
                assert!(object
                    .sub_odd
                    .as_ref()
                    .is_none_or(|object| object.id == 2 * (i - 1) + 1));
            });
        }
    }

    #[test]
    fn shared_context_get_fails_when_object_lifetime_is_within_scope() {
        let mut providers = ProviderMap::new();
        providers.insert_shared(TestObject::get_provider(0), WebScope::Session);
        providers.insert(Box::new(InstanceProvider::new(key::of(), 0i32)));

        let context = SharedContext::new_root(Arc::new(providers));

        assert!(matches!(
            context.get(&key::qualified::<Arc<TestObject>, u32>(0)),
            Err(InjectorError::ShortLifetime { .. })
        ));
        assert!(matches!(
            context.get(&key::of::<i32>()),
            Err(InjectorError::ShortLifetime { .. })
        ));
    }

    #[test]
    fn shared_context_get_fails_when_there_exists_cyclic_dependency() {
        let mut providers = ProviderMap::new();
        providers.insert_shared(RecursiveObject::get_provider(), WebScope::Singleton);

        let context = SharedContext::new_root(Arc::new(providers));

        assert!(matches!(
            context.get(&key::of::<Arc<RecursiveObject>>()),
            Err(InjectorError::CyclicDependency { .. })
        ));
    }

    #[test]
    fn shared_context_get_fails_when_key_not_found() {
        let providers: ProviderMap<WebScope> = ProviderMap::new();
        let context = SharedContext::new_root(Arc::new(providers));

        assert!(matches!(
            context.get(&key::of::<i32>()),
            Err(InjectorError::NotFound { .. })
        ));
    }
}
