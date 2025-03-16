mod implementation;

use crate::container::injector::{Injector, InjectorError};
use crate::container::Managed;

pub use implementation::RawClosureProvider;

pub trait RawClosure
where
    Self: Fn(&dyn Injector) -> Result<Self::Constructed, InjectorError>,
    Self: Send + Sync + 'static,
{
    type Constructed: Managed;
}

impl<F, T> RawClosure for F
where
    T: Managed,
    Self: Fn(&dyn Injector) -> Result<T, InjectorError>,
    Self: Send + Sync + 'static,
{
    type Constructed = T;
}
