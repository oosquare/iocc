use snafu::prelude::*;

use crate::key::Key;

#[derive(Debug, Snafu)]
#[non_exhaustive]
pub enum RegistryError {
    #[snafu(display("the key {key} already exists in the registry"))]
    #[non_exhaustive]
    KeyDuplicated { key: Box<dyn Key> },
}

impl Clone for RegistryError {
    fn clone(&self) -> Self {
        match self {
            Self::KeyDuplicated { key } => Self::KeyDuplicated {
                key: key.dyn_clone(),
            },
        }
    }
}
