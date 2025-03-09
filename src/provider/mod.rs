pub mod adapter;
pub mod instance;

use std::fmt::Debug;

use crate::container::injector::{Injector, InjectorError, TypedInjector};
use crate::container::{Managed, SharedManaged};
use crate::key::{Key, TypedKey};

pub trait Provider: Debug + Send + Sync + 'static {
    fn dyn_provide(
        &mut self,
        injector: &mut dyn Injector,
    ) -> Result<Box<dyn Managed>, InjectorError>;

    fn dyn_key(&self) -> &dyn Key;
}

pub trait TypedProvider: Provider {
    type Key: TypedKey<Target = Self::Output>;

    type Output: Managed;

    fn provide<I>(&mut self, injector: &mut I) -> Result<Self::Output, InjectorError>
    where
        I: TypedInjector + ?Sized;

    fn key(&self) -> &Self::Key;
}

impl<T: TypedProvider> Provider for T {
    fn dyn_provide(
        &mut self,
        injector: &mut dyn Injector,
    ) -> Result<Box<dyn Managed>, InjectorError> {
        self.provide(injector)
            .map(|obj| -> Box<dyn Managed> { Box::new(obj) })
    }

    fn dyn_key(&self) -> &dyn Key {
        self.key()
    }
}

pub trait SharedProvider: Provider {
    fn dyn_provide_shared(
        &mut self,
        injector: &mut dyn Injector,
    ) -> Result<Box<dyn SharedManaged>, InjectorError>;
}

pub trait TypedSharedProvider
where
    Self: SharedProvider + TypedProvider<Output: SharedManaged>,
{
}

impl<T: TypedSharedProvider> SharedProvider for T {
    fn dyn_provide_shared(
        &mut self,
        injector: &mut dyn Injector,
    ) -> Result<Box<dyn SharedManaged>, InjectorError> {
        self.provide(injector)
            .map(|obj| -> Box<dyn SharedManaged> { Box::new(obj) })
    }
}
