mod closure_implementation;
mod raw_wrapper;
mod wrapper;

use std::error::Error;

use crate::container::injector::{Injector, InjectorError};
use crate::container::Managed;

pub use raw_wrapper::RawClosureProvider;
pub use wrapper::ClosureProvider;

/// A specialized form of [`Fn`] that can be called by supplying arguments
/// retrieved from an [`Injector`].
///
/// Closures of `Fn(A1, A2, ...) -> Result<T, E> + Send + Sync + 'static`
/// where `Ai: Managed` are [`Closure`]. Note that each argument is fetched
/// without specifying a qualifier.
///
/// Usually you don't need to use a [`Closure`] directly. The most recommended
/// way is to use `to_closure()` [`dsl`]s offered by this crate. Or wrap your
/// [`Closure`] in a [`ClosureProvider`] if you need low-level control.
///
/// Theoretically, a [`Closure`] can be implemented by any closure of arbitrary
/// arity whose arguments are all [`Managed`]. Due to the lack of support for
/// functions of variable length parameters, [`Closure`] is only implemented by
/// any function whose arity is at most 16.
///
/// [`dsl`]: crate::module::dsl
pub trait Closure<D>
where
    Self: Send + Sync + 'static,
    D: Send + Sync + 'static,
{
    /// The successfully constructed object.
    type Constructed: Managed;

    /// The error occurred in object construction after all dependencies are
    /// retrieved.
    type Error: Into<Box<dyn Error + Send + Sync>>;

    /// Retrieves the dependencies from the injector and calls `self` with
    /// these dependencies.
    ///
    /// # Errors
    ///
    /// Returns an error if all dependencies can't be fetched.
    ///
    /// Returns an inner error [`Closure::Error`] wrapped in the outer [`Ok`]
    /// if the object construction fails.
    fn run(
        &self,
        injector: &dyn Injector,
    ) -> Result<Result<Self::Constructed, Self::Error>, InjectorError>;
}

/// A specialized form of [`Fn`] which directly accepts an [`Injector`] and
/// constructs objects.
///
/// Usually you don't need to use a [`RawClosure`] directly. The most
/// recommended way is to use `to_raw_closure()` [`dsl`]s offered by this
/// crate. Or wrap your [`RawClosure`] in a [`RawClosureProvider`] if you need
/// low-level control.
///
/// Usually you don't need to use a [`Closure`] directly. The most recommended
/// way is to use [`dsl`]s offered by this crate. Or wrap your [`Closure`] in a
/// [`ClosureProvider`] if you need low-level control.
///
/// [`dsl`]: crate::module::dsl
pub trait RawClosure
where
    Self: Fn(&dyn Injector) -> Result<Result<Self::Constructed, Self::Error>, InjectorError>,
    Self: Send + Sync + 'static,
{
    /// The successfully constructed object.
    type Constructed: Managed;

    /// The error occurred in object construction after all dependencies are
    /// retrieved.
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
