use crate::container::injector::{InjectorError, TypedInjector};
use crate::container::Managed;

pub trait Component: Managed + Sized {
    type Output: Managed;

    fn construct<I>(injector: &mut I) -> Result<Self, InjectorError>
    where
        I: TypedInjector + ?Sized;

    fn post_process(self) -> Self::Output;
}
