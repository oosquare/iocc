use crate::key::Key;

#[derive(Clone)]
pub struct CallContext<'a> {
    key: &'a dyn Key,
}

impl<'a> CallContext<'a> {
    pub fn new(key: &'a dyn Key) -> Self {
        Self { key }
    }

    pub fn key(&self) -> &dyn Key {
        self.key
    }
}
