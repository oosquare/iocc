mod collect;
mod object_map;
mod proxy;

use std::any::TypeId;
use std::error::Error;
use std::sync::Arc;

use snafu::prelude::*;

use crate::container::Managed;
use crate::key::{Key, Pattern, TypedKey};
use crate::provider::context::CallContext;
use crate::util::any::Downcast;

pub use collect::Collect;
pub(super) use object_map::ObjectMap;
pub(crate) use proxy::ContextForwardingInjectorProxy;

#[cfg_attr(test, mockall::automock)]
pub trait Injector: Send + Sync {
    fn dyn_get(&self, key: &dyn Key) -> Result<Box<dyn Managed>, InjectorError>;

    fn dyn_get_dependency<'a>(
        &self,
        key: &dyn Key,
        context: &'a CallContext<'a>,
    ) -> Result<Box<dyn Managed>, InjectorError>;

    fn keys(&self, type_id: TypeId) -> Vec<Box<dyn Key>>;
}

pub trait TypedInjector: Injector {
    fn get<K>(&self, key: K) -> Result<K::Target, InjectorError>
    where
        K: TypedKey,
    {
        match self.dyn_get(&key) {
            Ok(boxed) => match boxed.downcast::<K::Target>() {
                Ok(object) => Ok(*object),
                Err(_) => unreachable!("the object's type should be `K::Target`"),
            },
            Err(err) => Err(err),
        }
    }

    fn collect<C, P>(&self, pattern: P) -> Result<C, InjectorError>
    where
        C: Collect<P>,
        P: Pattern,
    {
        let keys = self.keys(TypeId::of::<P::Target>());
        C::collect(self, keys.iter().map(AsRef::as_ref), pattern)
    }

    fn upcast_dyn(&self) -> &dyn Injector;
}

impl<T> TypedInjector for T
where
    T: Injector,
{
    fn upcast_dyn(&self) -> &dyn Injector {
        self
    }
}

impl TypedInjector for dyn Injector + '_ {
    fn upcast_dyn(&self) -> &dyn Injector {
        self
    }
}

#[derive(Debug, Snafu)]
#[non_exhaustive]
pub enum InjectorError {
    #[snafu(display("could not find the object identified by the given key {key}"))]
    #[non_exhaustive]
    NotFound { key: Box<dyn Key> },
    #[snafu(display("could not gather any object matching {pattern} to a {collection}"))]
    #[non_exhaustive]
    EmptyCollection {
        collection: &'static str,
        pattern: &'static str,
    },
    #[snafu(display("could not construct the object {key} which depends on itself somehow"))]
    #[non_exhaustive]
    CyclicDependency { key: Box<dyn Key> },
    #[snafu(display("could not build a object {key} of {lifetime} lifetime in a {scope} scope"))]
    #[non_exhaustive]
    ShortLifetime {
        key: Box<dyn Key>,
        lifetime: &'static str,
        scope: &'static str,
    },
    #[snafu(display("could not construct the object {key}"))]
    #[non_exhaustive]
    ObjectConstruction {
        key: Box<dyn Key>,
        source: Arc<dyn Error + Send + Sync>,
    },
}

impl Clone for InjectorError {
    fn clone(&self) -> Self {
        match self {
            Self::NotFound { key } => Self::NotFound {
                key: key.dyn_clone(),
            },
            Self::EmptyCollection {
                collection,
                pattern,
            } => Self::EmptyCollection {
                collection,
                pattern,
            },
            Self::CyclicDependency { key } => Self::CyclicDependency {
                key: key.dyn_clone(),
            },
            Self::ShortLifetime {
                key,
                lifetime,
                scope,
            } => Self::ShortLifetime {
                key: key.dyn_clone(),
                lifetime,
                scope,
            },
            Self::ObjectConstruction { key, source } => Self::ObjectConstruction {
                key: key.dyn_clone(),
                source: Arc::clone(source),
            },
        }
    }
}
