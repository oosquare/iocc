pub(super) mod object_map;

use std::any;
use std::error::Error;

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
                boxed.downcast::<K::Target>().map_err(|_| {
                    TypeMismatchedSnafu {
                        expected_type: any::type_name::<K::Target>(),
                    }
                    .build()
                })
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
    #[snafu(display("could not find the object identified by the given key {key}"))]
    #[non_exhaustive]
    NotFound { key: Box<dyn Key> },
    #[snafu(display("could not construct the object {key} which depends on itself somehow"))]
    #[non_exhaustive]
    CyclicDependency { key: Box<dyn Key> },
    #[snafu(display("could not downcast the object to the given concrete type"))]
    #[non_exhaustive]
    TypeMismatched { expected_type: &'static str },
    #[snafu(display("could not get the object {key} from the adapter's inner"))]
    #[non_exhaustive]
    AdapterInner {
        key: Box<dyn Key>,
        #[snafu(source(from(InjectorError, Box::new)))]
        source: Box<InjectorError>,
    },
    #[snafu(display("could not construct the object {key}"))]
    #[non_exhaustive]
    ObjectConstruction {
        key: Box<dyn Key>,
        source: Box<dyn Error + Send + Sync>,
    },
}
