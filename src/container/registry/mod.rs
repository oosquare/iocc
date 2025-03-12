mod configurer;
mod provider_map;

use std::error::Error;

use snafu::prelude::*;

use crate::key::Key;
use crate::provider::{Provider, SharedProvider};

pub trait Configurer: Send + Sync + 'static {
    fn register(&mut self, provider: Box<dyn Provider>);

    fn register_shared(&mut self, provider: Box<dyn SharedProvider>);

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
