use std::any::TypeId;

use crate::container::injector::{Injector, InjectorError, TypedInjector};
use crate::container::Managed;
use crate::key::Key;
use crate::provider::context::CallContext;

pub struct ContextForwardingInjectorProxy<'a, I>
where
    I: TypedInjector + ?Sized,
{
    inner: &'a I,
    context: &'a CallContext<'a>,
}

impl<'a, I> ContextForwardingInjectorProxy<'a, I>
where
    I: TypedInjector + ?Sized,
{
    pub fn new(inner: &'a I, context: &'a CallContext<'a>) -> Self {
        Self { inner, context }
    }
}

impl<I> Injector for ContextForwardingInjectorProxy<'_, I>
where
    I: TypedInjector + ?Sized,
{
    fn dyn_get(&self, key: &dyn Key) -> Result<Box<dyn Managed>, InjectorError> {
        self.dyn_get_dependency(key, self.context)
    }

    fn dyn_get_dependency<'a>(
        &self,
        key: &dyn Key,
        context: &'a CallContext<'a>,
    ) -> Result<Box<dyn Managed>, InjectorError> {
        self.inner.dyn_get_dependency(key, context)
    }

    fn keys(&self, type_id: TypeId) -> Vec<Box<dyn Key>> {
        self.inner.keys(type_id)
    }
}
