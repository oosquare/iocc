use std::any::Any;
use std::fmt::Debug;

use crate::container::injector::{DynInjector, InjectorError, TypedInjector};
use crate::container::Managed;
use crate::key::{DynKey, TypedKey};

pub type DynProvider = dyn Provider + Send + Sync + 'static;

pub trait Provider: Debug + Send + Sync + 'static {
    fn dyn_provide(&self, injector: &mut DynInjector) -> Result<Box<dyn Any>, InjectorError>;

    fn dyn_key(&self) -> &DynKey;
}

pub trait TypedProvider: Provider {
    type Key: TypedKey<Target = Self::Output>;

    type Output: Managed;

    fn provide<I>(&self, injector: &mut I) -> Result<Self::Output, InjectorError>
    where
        I: TypedInjector + ?Sized;

    fn key(&self) -> &Self::Key;
}
