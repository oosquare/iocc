pub mod closure;
pub mod component;
pub mod instance;

use std::fmt::Debug;

use crate::container::injector::{Injector, InjectorError, TypedInjector};
use crate::container::{Managed, SharedManaged};

pub trait Provider: Debug + Send + Sync + 'static {
    fn dyn_provide(&self, injector: &dyn Injector) -> Result<Box<dyn Managed>, InjectorError>;
}

pub trait TypedProvider: Provider {
    type Output: Managed;

    fn provide<I>(&self, injector: &I) -> Result<Self::Output, InjectorError>
    where
        I: TypedInjector + ?Sized;
}

impl<T: TypedProvider> Provider for T {
    fn dyn_provide(&self, injector: &dyn Injector) -> Result<Box<dyn Managed>, InjectorError> {
        self.provide(injector)
            .map(|obj| -> Box<dyn Managed> { Box::new(obj) })
    }
}

pub trait SharedProvider: Provider {
    fn dyn_provide_shared(
        &self,
        injector: &dyn Injector,
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
    ) -> Result<Box<dyn SharedManaged>, InjectorError> {
        self.provide(injector)
            .map(|obj| -> Box<dyn SharedManaged> { Box::new(obj) })
    }
}
