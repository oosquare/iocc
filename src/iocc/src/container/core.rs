use std::any::TypeId;
use std::sync::Arc;

use crate::container::context::{LocalContext, SharedContext};
use crate::container::injector::{CallContext, Injector, InjectorError};
use crate::container::registry::{ConfigurerImpl, ProviderMap, Registry, RegistryError};
use crate::container::Managed;
use crate::key::Key;
use crate::module::Module;
use crate::scope::Scope;

pub struct Container<S: Scope> {
    context: LocalContext<S>,
}

impl<S: Scope> Container<S> {
    fn new_root(providers: ProviderMap<S>) -> Self {
        let shared = Arc::new(SharedContext::new_root(Arc::new(providers)));
        let local = LocalContext::new(shared);
        Self { context: local }
    }

    pub fn sub_container(&self) -> Option<Self> {
        if let Some(shared) = SharedContext::new_sub(self.context.shared()) {
            let local = LocalContext::new(Arc::new(shared));
            Some(Self { context: local })
        } else {
            None
        }
    }

    pub fn current_scope(&self) -> S {
        self.context.current_scope()
    }
}

impl<S: Scope> Clone for Container<S> {
    fn clone(&self) -> Self {
        let local = LocalContext::new(self.context.shared());
        Self { context: local }
    }
}

impl<S: Scope> Registry for Container<S> {
    type Scope = S;

    fn init<M>(module: M) -> Result<Self, Vec<RegistryError>>
    where
        M: Module<Scope = Self::Scope>,
    {
        let mut configurer = ConfigurerImpl::new();
        module.setup(&mut configurer);
        configurer.finish().map(Self::new_root)
    }
}

impl<S: Scope> Injector for Container<S> {
    fn dyn_get(&self, key: &dyn Key) -> Result<Box<dyn Managed>, InjectorError> {
        self.context.dyn_get(key)
    }

    fn dyn_get_dependency<'a>(
        &self,
        key: &dyn Key,
        context: &'a CallContext<'a>,
    ) -> Result<Box<dyn Managed>, InjectorError> {
        self.context.dyn_get_dependency(key, context)
    }

    fn keys(&self, type_id: TypeId) -> Vec<Box<dyn Key>> {
        self.context.shared_ref().keys(type_id)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::convert::Infallible;
    use std::error::Error;
    use std::thread;

    use parking_lot::Mutex;

    use crate::container::injector::TypedInjector;
    use crate::container::registry::{Configurer, TypedConfigurer};
    use crate::key::{self, KeyTypePattern};
    use crate::provider::component::{Component, ComponentProvider};
    use crate::provider::instance::InstanceProvider;
    use crate::scope::SingletonScope;

    use super::*;

    struct TestObject {
        value: Mutex<i32>,
        name: Arc<String>,
    }

    impl TestObject {
        fn new(name: Arc<String>) -> Self {
            TestObject {
                value: Mutex::new(0),
                name,
            }
        }

        fn set(&self, value: i32) {
            *self.value.lock() = value;
        }

        fn get(&self) -> i32 {
            *self.value.lock()
        }

        fn name(&self) -> &str {
            self.name.as_ref()
        }
    }

    impl Component for TestObject {
        type Constructed = Arc<Self>;

        type Error = Infallible;

        fn construct<I>(injector: &I) -> Result<Result<Self, Self::Error>, InjectorError>
        where
            I: TypedInjector + ?Sized,
        {
            Ok(Ok(Self::new(injector.get(key::of())?)))
        }

        fn post_process(self) -> Self::Constructed {
            Arc::new(self)
        }
    }

    struct TestModule;

    impl Module for TestModule {
        type Scope = SingletonScope;

        fn configure(
            &self,
            configurer: &mut dyn Configurer<Scope = Self::Scope>,
        ) -> Result<(), Box<dyn Error + Send + Sync>> {
            configurer.register_shared(
                key::of::<Arc<TestObject>>(),
                ComponentProvider::<TestObject>::new(),
                SingletonScope,
            );
            configurer.register_shared(
                key::of::<Arc<String>>(),
                InstanceProvider::new(Arc::new(String::from("test-object"))),
                SingletonScope,
            );
            configurer.register(key::named::<i32>("1"), InstanceProvider::new(1));
            configurer.register(key::named::<i32>("2"), InstanceProvider::new(2));
            Ok(())
        }
    }

    #[test]
    fn container_operations_succeeds() {
        let container = Container::init(TestModule).unwrap();

        let object: Arc<TestObject> = container.get(key::of()).unwrap();
        assert_eq!(object.get(), 0);
        assert_eq!(object.name(), "test-object");
        object.set(42);

        thread::spawn({
            let container = container.clone();
            move || {
                let object: Arc<TestObject> = container.get(key::of()).unwrap();
                assert_eq!(object.get(), 42);
            }
        });
        thread::spawn({
            let container = container.clone();
            move || {
                let object: Arc<TestObject> = container.get(key::of()).unwrap();
                assert_eq!(object.name(), "test-object");
            }
        });

        let objects: HashMap<&'static str, i32> = container.collect(KeyTypePattern::new()).unwrap();
        assert_eq!(objects.get(&"1"), Some(&1));
        assert_eq!(objects.get(&"2"), Some(&2));
    }
}
