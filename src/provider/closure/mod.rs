mod closure_implementation;
mod raw_wrapper;
mod wrapper;

use std::error::Error;

use crate::container::injector::{Injector, InjectorError};
use crate::container::Managed;

pub use raw_wrapper::RawClosureProvider;
pub use wrapper::ClosureProvider;

pub trait Closure<D>
where
    Self: Send + Sync + 'static,
    D: Send + Sync + 'static,
{
    type Constructed: Managed;

    type Error: Into<Box<dyn Error + Send + Sync>>;

    fn run(
        &self,
        injector: &dyn Injector,
    ) -> Result<Result<Self::Constructed, Self::Error>, InjectorError>;
}

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
