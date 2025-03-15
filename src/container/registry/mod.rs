mod configurer;
mod provider_map;

use std::error::Error;

use snafu::prelude::*;

use crate::key::Key;
use crate::module::Module;
use crate::provider::{Provider, SharedProvider};
use crate::scope::Scope;

pub use configurer::ConfigurerImpl;
pub use provider_map::{ProviderEntry, ProviderMap};

pub trait Registry: Sized + Send + Sync + 'static {
    type Scope: Scope;

    fn init<M>(module: M) -> Result<Self, Vec<RegistryError>>
    where
        M: Module<Scope = Self::Scope>;
}

pub trait Configurer: Send + Sync + 'static {
    type Scope: Scope;

    fn register(&mut self, key: Box<dyn Key>, provider: Box<dyn Provider>);

    fn register_shared(
        &mut self,
        key: Box<dyn Key>,
        provider: Box<dyn SharedProvider>,
        scope: Self::Scope,
    );

    fn report_module_error(&mut self, module: &'static str, err: Box<dyn Error + Send + Sync>);
}

#[derive(Debug, Snafu)]
#[non_exhaustive]
pub enum RegistryError {
    #[snafu(display("the key {key} already exists in the registry"))]
    #[non_exhaustive]
    KeyDuplicated { key: Box<dyn Key> },
    #[snafu(display("module {module} fails to setup the configuration"))]
    #[non_exhaustive]
    ModuleInner {
        module: &'static str,
        source: Box<dyn Error + Send + Sync>,
    },
}
