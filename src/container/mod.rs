pub mod injector;
pub mod registry;

mod core;

use std::sync::Arc;

use crate::util::any::AsAny;

pub use core::CoreContainer;

pub trait Managed: AsAny + Send + 'static {}

impl<T> Managed for T where T: AsAny + Send + 'static {}

pub trait SharedManaged: Managed + Sync {
    fn dyn_clone(&self) -> Box<dyn SharedManaged>;

    fn upcast_managed(self: Box<Self>) -> Box<dyn Managed>;
}

impl<T> SharedManaged for Arc<T>
where
    T: Send + Sync + ?Sized + 'static,
{
    fn dyn_clone(&self) -> Box<dyn SharedManaged> {
        Box::new(Arc::clone(self))
    }

    fn upcast_managed(self: Box<Self>) -> Box<dyn Managed> {
        self
    }
}
