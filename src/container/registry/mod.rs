mod provider_map;

use std::error::Error;

use snafu::prelude::*;

use crate::key::Key;
use crate::provider::{Provider, SharedProvider};

pub trait Configurer: Send + Sync + 'static {
    fn register(&mut self, provider: Box<dyn Provider>);

    fn register_shared(&mut self, provider: Box<dyn SharedProvider>);

    fn report_error(&mut self, err: Box<dyn Error + Send + Sync>);

    fn finish(self) -> Result<(), Vec<RegistryError>>
    where
        Self: Sized;
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
