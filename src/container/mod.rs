pub mod injector;
mod object_map;
mod provider_map;
pub mod registry;

use std::sync::Arc;

use crate::util::any::AsAny;

pub trait Managed: AsAny + Send + 'static {}

impl<T> Managed for T where T: AsAny + Send + 'static {}

pub trait SharedManaged: Managed {
    fn dyn_clone(&self) -> Box<dyn SharedManaged>;
}

impl<T> SharedManaged for Arc<T>
where
    T: Send + Sync + ?Sized + 'static,
{
    fn dyn_clone(&self) -> Box<dyn SharedManaged> {
        Box::new(Arc::clone(self))
    }
}
