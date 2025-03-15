use std::sync::Arc;

use crate::container::context::{LocalContext, SharedContext};
use crate::container::injector::{Injector, InjectorError};
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
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;
    use std::error::Error;
    use std::thread;

    use parking_lot::Mutex;

    use crate::component::Component;
    use crate::container::injector::TypedInjector;
    use crate::container::registry::Configurer;
    use crate::key;
    use crate::provider::component::ComponentProvider;
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
        type Output = Arc<Self>;

        type Error = Infallible;

        fn construct<I>(injector: &I) -> Result<Result<Self, Self::Error>, InjectorError>
        where
            I: TypedInjector + ?Sized,
        {
            Ok(Ok(Self::new(injector.get(&key::of())?)))
        }

        fn post_process(self) -> Self::Output {
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
                Box::new(key::of::<Arc<TestObject>>()),
                Box::new(ComponentProvider::<TestObject>::new()),
                SingletonScope,
            );
            configurer.register_shared(
                Box::new(key::of::<Arc<String>>()),
                Box::new(InstanceProvider::new(Arc::new(String::from("test-object")))),
                SingletonScope,
            );
            Ok(())
        }
    }

    #[test]
    fn container_operations_succeeds() {
        let container = Container::init(TestModule).unwrap();

        let object: Arc<TestObject> = container.get(&key::of()).unwrap();
        assert_eq!(object.get(), 0);
        assert_eq!(object.name(), "test-object");
        object.set(42);

        thread::spawn({
            let container = container.clone();
            move || {
                let object: Arc<TestObject> = container.get(&key::of()).unwrap();
                assert_eq!(object.get(), 42);
            }
        });
        thread::spawn({
            let container = container.clone();
            move || {
                let object: Arc<TestObject> = container.get(&key::of()).unwrap();
                assert_eq!(object.name(), "test-object");
            }
        });
    }
}
