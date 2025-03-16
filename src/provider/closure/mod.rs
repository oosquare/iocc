mod implementation;

use std::error::Error;

use crate::container::injector::{Injector, InjectorError};
use crate::container::Managed;

pub use implementation::RawClosureProvider;

pub trait RawClosure
where
    Self: Fn(&dyn Injector) -> Result<Result<Self::Constructed, Self::Error>, InjectorError>,
    Self: Send + Sync + 'static,
{
    type Constructed: Managed;

    type Error: Into<Box<dyn Error + Send + Sync>>;
}

impl<F, T, E> RawClosure for F
where
    T: Managed,
    E: Into<Box<dyn Error + Send + Sync>>,
    Self: Fn(&dyn Injector) -> Result<Result<T, E>, InjectorError>,
    Self: Send + Sync + 'static,
{
    type Constructed = T;

    type Error = E;
}
