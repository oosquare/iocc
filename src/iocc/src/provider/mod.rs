pub mod closure;
pub mod component;
pub mod instance;

use std::fmt::Debug;

use crate::container::injector::{Injector, InjectorError, TypedInjector};
use crate::container::{Managed, SharedManaged};
use crate::key::Key;

pub trait Provider: Debug + Send + Sync + 'static {
    fn dyn_provide(
        &self,
        injector: &dyn Injector,
        context: &CallContext<'_>,
    ) -> Result<Box<dyn Managed>, InjectorError>;
}

pub trait TypedProvider: Provider {
    type Output: Managed;

    fn provide<I>(
        &self,
        injector: &I,
        context: &CallContext<'_>,
    ) -> Result<Self::Output, InjectorError>
    where
        I: TypedInjector + ?Sized;
}

impl<T: TypedProvider> Provider for T {
    fn dyn_provide(
        &self,
        injector: &dyn Injector,
        context: &CallContext<'_>,
    ) -> Result<Box<dyn Managed>, InjectorError> {
        self.provide(injector, context)
            .map(|obj| -> Box<dyn Managed> { Box::new(obj) })
    }
}

pub trait SharedProvider: Provider {
    fn dyn_provide_shared(
        &self,
        injector: &dyn Injector,
        context: &CallContext<'_>,
    ) -> Result<Box<dyn SharedManaged>, InjectorError>;
}

pub trait TypedSharedProvider
where
    Self: SharedProvider + TypedProvider<Output: SharedManaged>,
{
}

impl<T: TypedSharedProvider> SharedProvider for T {
    fn dyn_provide_shared(
        &self,
        injector: &dyn Injector,
        context: &CallContext<'_>,
    ) -> Result<Box<dyn SharedManaged>, InjectorError> {
        self.provide(injector, context)
            .map(|obj| -> Box<dyn SharedManaged> { Box::new(obj) })
    }
}

#[derive(Clone)]
pub struct CallContext<'a> {
    key: &'a dyn Key,
}

impl<'a> CallContext<'a> {
    pub fn new(key: &'a dyn Key) -> Self {
        Self { key }
    }

    pub fn key(&self) -> &dyn Key {
        self.key
    }
}
