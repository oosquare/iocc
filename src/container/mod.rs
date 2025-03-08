pub mod injector;
pub mod registry;

use std::sync::Arc;

use crate::util::any::AsAny;

pub trait Managed: AsAny + Send + 'static {}

impl<T> Managed for T where T: AsAny + Send + 'static {}

pub trait SharedManaged: Managed {
    fn dyn_clone(&self) -> Box<dyn SharedManaged + Send + 'static>;
}

impl<T> SharedManaged for Arc<T>
where
    T: Send + Sync + 'static,
{
    fn dyn_clone(&self) -> Box<dyn SharedManaged + Send + 'static> {
        Box::new(Arc::clone(self))
    }
}
