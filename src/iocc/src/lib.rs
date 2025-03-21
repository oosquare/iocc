#![allow(clippy::new_without_default)]

pub mod container;
pub mod key;
pub mod module;
pub mod provider;
pub mod scope;
mod util;

pub use iocc_derive::component;

pub mod prelude {
    pub use crate::component;
    pub use crate::container::injector::{InjectorError, TypedInjector};
    pub use crate::container::registry::{Configurer, Registry, RegistryError};
    pub use crate::container::Container;
    pub use crate::key;
    pub use crate::module::{bind, Configuration, Module};
}
