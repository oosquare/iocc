mod wrapper;

use std::error::Error;

use crate::container::injector::{InjectorError, TypedInjector};
use crate::container::Managed;

pub use wrapper::ComponentProvider;

/// A type that has a dedicated constructor for dependency injection.
///
/// Usually, you don't need to implement the [`Component`] trait manually for
/// your components, because the [`component`] macro helps with this. In case
/// that you really want to write implementation in your own, take a look at
/// the following code snippet:
///
/// ```rust
/// # use std::sync::Arc;
/// # use std::convert::Infallible;
/// # use iocc::container::injector::{TypedInjector, InjectorError};
/// # use iocc::provider::component::Component;
/// # use iocc::key;
/// #
/// trait MyTrait: Send + Sync + 'static {}
///
/// struct MyComponent {
///     dep1: i32,
///     dep2: Arc<f64>,
/// }
///
/// impl MyTrait for MyComponent {}
///
/// impl Component for MyComponent {
///     type Constructed = Arc<dyn MyTrait>;
///
///     type Error = Infallible;
///
///     fn construct<I>(injector: &I) -> Result<Result<Self, Self::Error>, InjectorError>
///     where
///         I: TypedInjector + ?Sized
///     {
///         let dep1 = injector.get(key::of())?;
///         let dep2 = injector.get(key::of())?;
///         Ok(Ok(Self { dep1, dep2 }))
///     }
///
///     fn post_process(self) -> Self::Constructed {
///         Arc::new(self)
///     }
/// }
/// ```
///
/// In addition, you don't need to use functions in [`Component`] directly. The
/// most recommended way is to use `to_component()` [`dsl`]s offered by this
/// crate. Or wrap your [`Component`] in a [`ComponentProvider`] if you need
/// low-level control.
///
/// [`component`]: crate::component
/// [`dsl`]: crate::module::dsl
pub trait Component: Managed + Sized {
    /// The successfully constructed object. This can be not only `Self`, but
    /// also some boxed `Self`, such as `Arc<Self>` and `Arc<dyn Trait>`.
    type Constructed: Managed;

    /// The error occurred in object construction after all dependencies are
    /// retrieved.
    type Error: Into<Box<dyn Error + Send + Sync>>;

    /// Retrieves the dependencies from the injector and creates the object.
    ///
    /// # Errors
    ///
    /// Returns an error if all dependencies can't be fetched.
    ///
    /// Returns an inner error [`Component::Error`] wrapped in the outer [`Ok`]
    /// if the object construction fails.
    fn construct<I>(injector: &I) -> Result<Result<Self, Self::Error>, InjectorError>
    where
        I: TypedInjector + ?Sized;

    /// Converts `self` to [`Component::Constructed`]. Typical usages are
    /// putting `self` to an [`Arc`] and coercing it to an `Arc<dyn Trait>`.
    ///
    /// [`Arc`]: std::sync::Arc
    fn post_process(self) -> Self::Constructed;
}
