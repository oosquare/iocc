use std::error::Error;
use std::marker::PhantomData;

use crate::container::registry::Configurer;
use crate::scope::Scope;
use crate::util::any::AsAny;

pub trait Module: AsAny + 'static {
    type Scope: Scope;

    fn setup(&self, configurer: &mut dyn Configurer<Scope = Self::Scope>) {
        if let Err(err) = self.configure(configurer) {
            configurer.report_module_error(self.type_name(), err);
        }
    }

    fn configure(
        &self,
        configurer: &mut dyn Configurer<Scope = Self::Scope>,
    ) -> Result<(), Box<dyn Error + Send + Sync>>;
}

pub struct Configuration<S: Scope> {
    modules: Vec<Box<dyn Module<Scope = S>>>,
    _marker: PhantomData<S>,
}

impl<S: Scope> Configuration<S> {
    pub fn new() -> Self {
        Self {
            modules: Vec::new(),
            _marker: PhantomData,
        }
    }

    pub fn with<M: Module<Scope = S>>(mut self, module: M) -> Self {
        self.modules.push(Box::new(module));
        self
    }

    pub fn compose(mut self, mut other: Self) -> Self {
        self.modules.append(&mut other.modules);
        self
    }
}

impl<S: Scope> Module for Configuration<S> {
    type Scope = S;

    fn configure(
        &self,
        configurer: &mut dyn Configurer<Scope = Self::Scope>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.modules
            .iter()
            .for_each(|module| module.setup(configurer));
        Ok(())
    }
}
