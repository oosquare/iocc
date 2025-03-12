use std::collections::HashSet;

use crate::container::injector::object_map::ObjectMap;
use crate::container::injector::{Injector, InjectorError};
use crate::container::registry::configurer::ConfigurerImpl;
use crate::container::registry::provider_map::{ProviderEntry, ProviderMap};
use crate::container::registry::{Registry, RegistryError};
use crate::container::{Managed, SharedManaged};
use crate::key::Key;
use crate::module::Module;

pub struct CoreContainer {
    provider_map: ProviderMap,
    object_map: ObjectMap,
    constructing: HashSet<Box<dyn Key>>,
}

impl CoreContainer {
    fn get_impl(&mut self, key: &dyn Key) -> Result<Box<dyn Managed>, InjectorError> {
        if let Some(entry) = self.object_map.get(key) {
            return Ok(entry.clone_managed());
        }

        match self.provider_map.move_out(key) {
            Some(ProviderEntry::Owned(mut provider)) => {
                let res = provider.dyn_provide(self);
                self.provider_map.insert(provider);
                res
            }
            Some(ProviderEntry::Shared(mut provider)) => {
                let res = provider.dyn_provide_shared(self);
                if let Ok(object) = res.as_ref() {
                    self.object_map.insert(key.dyn_clone(), object.dyn_clone());
                }
                res.map(SharedManaged::upcast_managed)
            }
            Some(ProviderEntry::TemporaryMoved(key)) => Err(InjectorError::CyclicDependency {
                key: key.dyn_clone(),
            }),
            None => Err(InjectorError::NotFound {
                key: key.dyn_clone(),
            }),
        }
    }
}

impl Registry for CoreContainer {
    fn init<M: Module>(module: M) -> Result<Self, Vec<RegistryError>> {
        let mut configurer = ConfigurerImpl::new();
        module.setup(&mut configurer);

        Ok(CoreContainer {
            provider_map: configurer.finish()?,
            object_map: ObjectMap::new(),
            constructing: HashSet::new(),
        })
    }
}

impl Injector for CoreContainer {
    fn dyn_get(&mut self, key: &dyn Key) -> Result<Box<dyn Managed>, InjectorError> {
        if self.constructing.contains(key) {
            Err(InjectorError::CyclicDependency {
                key: key.dyn_clone(),
            })
        } else {
            self.constructing.insert(key.dyn_clone());
            let res = self.get_impl(key);
            self.constructing.remove(key);
            res
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::component::Component;
    use crate::container::injector::TypedInjector;
    use crate::key;
    use crate::provider::component::ComponentProvider;
    use crate::provider::instance::CloneableInstanceProvider;

    use super::*;

    struct A {
        a: i32,
        b: &'static str,
    }

    impl Component for A {
        type Output = Arc<A>;

        fn construct<I>(injector: &mut I) -> Result<Self, InjectorError>
        where
            I: TypedInjector + ?Sized,
        {
            Ok(A {
                a: injector.get(&key::of::<i32>())?,
                b: injector.get(&key::of::<&'static str>())?,
            })
        }

        fn post_process(self) -> Self::Output {
            Arc::new(self)
        }
    }

    #[derive(Debug)]
    struct B {
        _recursive: Box<B>,
    }

    impl Component for B {
        type Output = Box<B>;

        fn construct<I>(injector: &mut I) -> Result<Self, InjectorError>
        where
            I: TypedInjector + ?Sized,
        {
            Ok(B {
                _recursive: injector.get(&key::of::<Box<B>>())?,
            })
        }

        fn post_process(self) -> Self::Output {
            Box::new(self)
        }
    }

    #[test]
    fn core_container_get_succeeds() {
        let mut provider_map = ProviderMap::new();
        provider_map.insert(Box::new(CloneableInstanceProvider::new(key::of(), 42i32)));
        provider_map.insert(Box::new(CloneableInstanceProvider::new(key::of(), "str")));
        provider_map.insert_shared(Box::new(ComponentProvider::<_, A>::new(key::of())));

        let mut container = CoreContainer {
            provider_map,
            object_map: ObjectMap::new(),
            constructing: HashSet::new(),
        };

        let obj = container.get(&key::of::<Arc<A>>()).unwrap();
        assert_eq!(obj.a, 42i32);
        assert_eq!(obj.b, "str");
    }

    #[test]
    fn core_container_get_fails_when_cyclic_dependency_occurrs() {
        let mut provider_map = ProviderMap::new();
        provider_map.insert(Box::new(ComponentProvider::<_, B>::new(key::of())));

        let mut container = CoreContainer {
            provider_map,
            object_map: ObjectMap::new(),
            constructing: HashSet::new(),
        };

        let err = container.get(&key::of::<Box<B>>()).unwrap_err();
        assert!(matches!(err, InjectorError::CyclicDependency { .. }));
    }

    #[test]
    fn core_container_get_fails_when_key_not_found() {
        let mut container = CoreContainer {
            provider_map: ProviderMap::new(),
            object_map: ObjectMap::new(),
            constructing: HashSet::new(),
        };

        let err = container.get(&key::of::<i32>()).unwrap_err();
        assert!(matches!(err, InjectorError::NotFound { .. }));
    }
}
