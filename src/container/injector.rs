use snafu::prelude::*;

use crate::container::Managed;
use crate::key::{Key, TypedKey};
use crate::util::any::Downcast;

#[cfg_attr(test, mockall::automock)]
pub trait Injector: Send + Sync + 'static {
    fn dyn_get(&mut self, key: &dyn Key) -> Result<Box<dyn Managed>, InjectorError>;
}

pub trait TypedInjector: Injector {
    fn get<K>(&mut self, key: &K) -> Result<K::Target, InjectorError>
    where
        K: TypedKey<Target: Managed>,
    {
        self.dyn_get(key)
            .and_then(|boxed| {
                boxed
                    .downcast::<K::Target>()
                    .map_err(|_| InjectorError::TypeMismatched)
            })
            .map(|boxed| *boxed)
    }

    fn upcast_dyn(&mut self) -> &mut dyn Injector;
}

impl<T> TypedInjector for T
where
    T: Injector,
{
    fn upcast_dyn(&mut self) -> &mut dyn Injector {
        self
    }
}

impl TypedInjector for dyn Injector {
    fn upcast_dyn(&mut self) -> &mut dyn Injector {
        self
    }
}

#[derive(Debug, Snafu)]
#[non_exhaustive]
pub enum InjectorError {
    #[snafu(display("could not found the object identified by the given key {key}"))]
    #[non_exhaustive]
    NotFound { key: Box<dyn Key> },
    #[snafu(display("could not construct the object {key} which depends on itself somehow"))]
    #[non_exhaustive]
    CyclicDependency { key: Box<dyn Key> },
    #[snafu(display("could not downcast the object to the given concrete type"))]
    #[non_exhaustive]
    TypeMismatched,
}

impl Clone for InjectorError {
    fn clone(&self) -> Self {
        match self {
            Self::NotFound { key } => Self::NotFound {
                key: key.dyn_clone(),
            },
            Self::CyclicDependency { key } => Self::CyclicDependency {
                key: key.dyn_clone(),
            },
            Self::TypeMismatched => Self::TypeMismatched,
        }
    }
}
