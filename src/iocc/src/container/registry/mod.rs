mod configurer;
mod provider_map;

use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};

use snafu::prelude::*;

use crate::container::SharedManaged;
use crate::key::{Key, TypedKey};
use crate::module::Module;
use crate::provider::{Provider, SharedProvider, TypedProvider, TypedSharedProvider};
use crate::scope::Scope;

pub(super) use configurer::ConfigurerImpl;
pub(super) use provider_map::{ProviderEntry, ProviderMap};

pub trait Registry: Sized + Send + Sync + 'static {
    type Scope: Scope;

    fn init<M>(module: M) -> Result<Self, RegistryError>
    where
        M: Module<Scope = Self::Scope>;
}

pub trait Configurer: Send + Sync + 'static {
    type Scope: Scope;

    #[doc(hidden)]
    #[allow(private_interfaces)]
    fn as_private(&mut self) -> &mut dyn ConfigurerPrivate<Scope = Self::Scope>;

    fn report_module_error(&mut self, module: &'static str, err: Box<dyn Error + Send + Sync>);
}

trait ConfigurerPrivate: Configurer {
    fn dyn_register(&mut self, key: Box<dyn Key>, provider: Box<dyn Provider>);

    fn dyn_register_shared(
        &mut self,
        key: Box<dyn Key>,
        provider: Box<dyn SharedProvider>,
        scope: Self::Scope,
    );
}

pub trait TypedConfigurer: Configurer {
    fn register<K, P>(&mut self, key: K, provider: P)
    where
        K: TypedKey,
        P: TypedProvider<Output = K::Target>,
    {
        self.as_private()
            .dyn_register(Box::new(key), Box::new(provider));
    }

    fn register_shared<K, P>(&mut self, key: K, provider: P, scope: Self::Scope)
    where
        K: TypedKey<Target: SharedManaged>,
        P: TypedSharedProvider<Output = K::Target>,
    {
        self.as_private()
            .dyn_register_shared(Box::new(key), Box::new(provider), scope);
    }
}

impl<T: Configurer + ?Sized> TypedConfigurer for T {}

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
    #[snafu(display("aggregated registry errors:\n{}", AggregatedDisplayer::new(errors)))]
    Aggregated { errors: Vec<RegistryError> },
}

struct AggregatedDisplayer<'a> {
    errors: &'a [RegistryError],
}

impl<'a> AggregatedDisplayer<'a> {
    fn new(errors: &'a [RegistryError]) -> Self {
        Self { errors }
    }
}

impl Display for AggregatedDisplayer<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        for (i, error) in self.errors.iter().enumerate() {
            writeln!(f, "{:4}: {}", i + 1, error)?;
        }
        Ok(())
    }
}
