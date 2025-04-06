pub mod closure;
pub mod component;
pub mod instance;

use std::fmt::Debug;

use crate::container::injector::{CallContext, Injector, InjectorError, TypedInjector};
use crate::container::{Managed, SharedManaged};

/// A universal factory which constructs objects of one type.
///
/// A [`Provider`] is responsible for constructing a object on each request and
/// retrieving all dependencies from an [`Injector`].
///
/// In convention, a [`Provider`] is a stateless object and may be used by
/// multiple threads. Each request to a [`Provider`] should receive a new
/// object. Especially, if the underlying type is some kind of pointer, it
/// should points to a new object rather than a shared one, unless it's a
/// truly immutable object without interior mutability. Enforcing this avoids
/// unexpected mutation and makes codes easier to read.
///
/// Usually, you don't need to implement [`Provider`] manually, since this is
/// automatically done by [`TypedProvider`]'s blanket implementation. See
/// [`TypedProvider`] for more information.
pub trait Provider: Debug + Send + Sync + 'static {
    /// Provides a newly created type-erased object. A [`Injector`] is needed
    /// since some other objects may be dependent on the object requested by
    /// the caller. The `context` preserves all additional information.
    ///
    /// # Errors
    ///
    /// Returns an error if all dependencies can't be fetched or the object
    /// construction fails.
    fn dyn_provide(
        &self,
        injector: &dyn Injector,
        context: &CallContext<'_>,
    ) -> Result<Box<dyn Managed>, InjectorError>;
}

/// A static variant of the [`Provider`] trait, leveraging static dispatch and
/// type-safety.
///
/// Usually, you don't need to implement [`Provider`] manually, since this is
/// automatically done by [`TypedProvider`]'s blanket implementation.
pub trait TypedProvider: Provider {
    /// The return type in response to each request to the provider.
    type Output: Managed;

    /// Provides a newly created object of type [`TypedProvider::Output`]. A
    /// [`TypedInjector`] is needed since some other objects may be dependent
    /// on the object requested by the caller. The `context` preserves all
    /// additional information.
    ///
    /// # Errors
    ///
    /// Returns an error if all dependencies can't be fetched or the object
    /// construction fails.
    fn provide<I>(
        &self,
        injector: &I,
        context: &CallContext<'_>,
    ) -> Result<Self::Output, InjectorError>
    where
        I: TypedInjector + ?Sized;
}

impl<T: TypedProvider> Provider for T {
    fn dyn_provide(
        &self,
        injector: &dyn Injector,
        context: &CallContext<'_>,
    ) -> Result<Box<dyn Managed>, InjectorError> {
        self.provide(injector, context)
            .map(|obj| -> Box<dyn Managed> { Box::new(obj) })
    }
}

/// A variant of the [`TypedProvider`] trait, which produces a shareable object.
///
/// Even though the requested object is shareable, the basic contract that each
/// request should be provided with a newly created object should still be
/// satisfied. This is because the shared ownership is managed by containers,
/// not [`Provider`] or its variants.
pub trait SharedProvider: Provider {
    /// Provides a newly created sharedable type-erased object. A [`Injector`]
    /// is needed since some other objects may be dependent on the object
    /// requested by the caller. The `context` preserves all additional
    /// information.
    ///
    /// # Errors
    ///
    /// Returns an error if all dependencies can't be fetched or the object
    /// construction fails.
    fn dyn_provide_shared(
        &self,
        injector: &dyn Injector,
        context: &CallContext<'_>,
    ) -> Result<Box<dyn SharedManaged>, InjectorError>;

    /// Returns a reference to `self` as a [`Provider`].
    fn upcast_provider(&self) -> &dyn Provider;
}

/// A static variant of the [`Provider`] trait, which produces a shareable object.
///
/// Even though the requested object is shareable, the basic contract that each
/// request should be provided with a newly created object should still be
/// satisfied. This is because the shared ownership is managed by containers,
/// not [`Provider`] or its variants.
pub trait TypedSharedProvider
where
    Self: SharedProvider + TypedProvider<Output: SharedManaged>,
{
}

impl<T: TypedSharedProvider> SharedProvider for T {
    fn dyn_provide_shared(
        &self,
        injector: &dyn Injector,
        context: &CallContext<'_>,
    ) -> Result<Box<dyn SharedManaged>, InjectorError> {
        self.provide(injector, context)
            .map(|obj| -> Box<dyn SharedManaged> { Box::new(obj) })
    }

    fn upcast_provider(&self) -> &dyn Provider {
        self
    }
}
