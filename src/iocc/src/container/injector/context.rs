use crate::key::Key;

#[derive(Clone)]
pub struct CallContext<'a> {
    trace: InjectionTrace<'a>,
}

impl<'a> CallContext<'a> {
    pub fn new(key: &'a dyn Key) -> Self {
        Self {
            trace: InjectionTrace::new(key),
        }
    }

    pub fn append<'b>(&'b self, key: &'b dyn Key) -> CallContext<'b> {
        CallContext {
            trace: self.trace.append(key),
        }
    }

    pub fn key(&self) -> &dyn Key {
        self.trace.key()
    }

    pub fn trace(&self) -> &InjectionTrace<'_> {
        &self.trace
    }
}

#[derive(Clone)]
pub struct InjectionTrace<'a> {
    key: &'a dyn Key,
    previous: Option<&'a InjectionTrace<'a>>,
}

impl<'a> InjectionTrace<'a> {
    pub fn new(key: &'a dyn Key) -> Self {
        Self {
            key,
            previous: None,
        }
    }

    pub fn append<'b>(&'b self, key: &'b dyn Key) -> InjectionTrace<'b> {
        InjectionTrace {
            key,
            previous: Some(self),
        }
    }

    pub fn key(&self) -> &dyn Key {
        self.key
    }

    pub fn previous(&self) -> Option<&InjectionTrace<'a>> {
        self.previous
    }

    pub fn previous_exist_key(&self, key: &dyn Key) -> bool {
        let mut this = self;
        while let Some(previous) = this.previous() {
            if previous.key() == key {
                return true;
            }
            this = previous;
        }
        false
    }
}
