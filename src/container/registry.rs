use snafu::prelude::*;

use crate::key::{DynKey, TypedKey};
use crate::provider::{DynProvider, TypedProvider};

pub type DynRegistry = dyn Registry + Send + Sync + 'static;

pub trait Registry: Send + Sync + 'static {
    fn dyn_register(
        &mut self,
        key: Box<DynKey>,
        provider: Box<DynProvider>,
    ) -> Result<(), RegistryError> {
        ensure!(
            &*key == provider.dyn_key(),
            KeyNotEqualSnafu {
                key,
                provider_key: provider.dyn_key().dyn_clone()
            }
        );
        self.dyn_register_key_unchecked(key, provider)
    }

    fn dyn_register_key_unchecked(
        &mut self,
        key: Box<DynKey>,
        provider: Box<DynProvider>,
    ) -> Result<(), RegistryError>;
}

pub trait TypedRegistry: Registry {
    fn register<K, P>(&mut self, key: K, provider: P) -> Result<(), RegistryError>
    where
        K: TypedKey,
        P: TypedProvider<Key = K>,
    {
        self.dyn_register(Box::new(key), Box::new(provider))
    }
}

impl<T> TypedRegistry for T where T: Registry + ?Sized {}

#[derive(Debug, Snafu)]
#[non_exhaustive]
pub enum RegistryError {
    #[snafu(display("the key {key} already exists in the registry"))]
    #[non_exhaustive]
    KeyDuplicated { key: Box<DynKey> },
    #[snafu(display("the key {key} is not identical to the provider's key {provider_key}"))]
    #[non_exhaustive]
    KeyNotEqual {
        key: Box<DynKey>,
        provider_key: Box<DynKey>,
    },
}

impl Clone for RegistryError {
    fn clone(&self) -> Self {
        match self {
            Self::KeyDuplicated { key } => Self::KeyDuplicated {
                key: key.dyn_clone(),
            },
            Self::KeyNotEqual { key, provider_key } => Self::KeyNotEqual {
                key: key.dyn_clone(),
                provider_key: provider_key.dyn_clone(),
            },
        }
    }
}
