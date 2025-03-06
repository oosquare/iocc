pub mod injector;
pub mod registry;

use as_any::AsAny;

pub trait Managed: AsAny + Send + 'static {}

impl<T> Managed for T where T: AsAny + Send + 'static {}
