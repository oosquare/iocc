#![allow(dead_code)]
use crate::container::registry::RegistryError;
use crate::key::Key;
use crate::provider::{Provider, SharedProvider, TypedProvider, TypedSharedProvider};

pub trait InnerRegistry: Send + Sync + 'static {
    fn dyn_register(&mut self, provider: Box<dyn Provider>) -> Result<(), RegistryError>;

    fn dyn_register_shared(
        &mut self,
        provider: Box<dyn SharedProvider>,
    ) -> Result<(), RegistryError>;

    fn get(&mut self, key: &dyn Key) -> Option<&mut VarProvider>;
}

pub trait TypedInnerRegistry: InnerRegistry {
    fn register<P>(&mut self, provider: P) -> Result<(), RegistryError>
    where
        P: TypedProvider,
    {
        self.dyn_register(Box::new(provider))
    }

    fn register_shared<P>(&mut self, provider: P) -> Result<(), RegistryError>
    where
        P: TypedSharedProvider,
    {
        self.dyn_register_shared(Box::new(provider))
    }
}

impl<T> TypedInnerRegistry for T where T: InnerRegistry + ?Sized {}

#[derive(Debug)]
pub enum VarProvider {
    Shared(Box<dyn SharedProvider>),
    Owned(Box<dyn Provider>),
}

impl VarProvider {
    pub fn dyn_key(&self) -> &dyn Key {
        match self {
            Self::Shared(s) => s.dyn_key(),
            Self::Owned(s) => s.dyn_key(),
        }
    }

    pub fn as_shared(&mut self) -> Option<&mut dyn SharedProvider> {
        if let Self::Shared(v) = self {
            Some(v.as_mut())
        } else {
            None
        }
    }

    pub fn as_owned(&mut self) -> Option<&mut dyn Provider> {
        if let Self::Owned(v) = self {
            Some(v.as_mut())
        } else {
            None
        }
    }
}

impl From<Box<dyn Provider>> for VarProvider {
    fn from(provider: Box<dyn Provider>) -> Self {
        VarProvider::Owned(provider)
    }
}

impl From<Box<dyn SharedProvider>> for VarProvider {
    fn from(provider: Box<dyn SharedProvider>) -> Self {
        VarProvider::Shared(provider)
    }
}
