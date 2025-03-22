mod wrapper;

use std::error::Error;

use crate::container::injector::{InjectorError, TypedInjector};
use crate::container::Managed;

pub use wrapper::ComponentProvider;

pub trait Component: Managed + Sized {
    type Constructed: Managed;

    type Error: Into<Box<dyn Error + Send + Sync>>;

    fn construct<I>(injector: &I) -> Result<Result<Self, Self::Error>, InjectorError>
    where
        I: TypedInjector + ?Sized;

    fn post_process(self) -> Self::Constructed;
}
