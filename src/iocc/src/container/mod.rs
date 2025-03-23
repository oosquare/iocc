pub mod injector;
pub mod registry;

mod core;
mod handle;

use std::sync::Arc;

use crate::util::any::AsAny;

pub use handle::Container;

pub trait Managed: AsAny + Send + Sync + 'static {}

impl<T> Managed for T where T: AsAny + Send + Sync + 'static {}

pub trait SharedManaged: Managed {
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
