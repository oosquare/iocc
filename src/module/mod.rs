use std::error::Error;

use crate::container::registry::Configurer;

pub trait Module: 'static {
    fn setup(&self, configurer: &mut dyn Configurer) {
        if let Err(err) = self.configure(configurer) {
            configurer.report_error(err);
        }
    }

    fn configure(
        &self,
        configurer: &mut dyn Configurer,
    ) -> Result<(), Box<dyn Error + Send + Sync>>;
}

#[derive(Default)]
pub struct Configuration {
    modules: Vec<Box<dyn Module>>,
}

impl Configuration {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with<M: Module>(mut self, module: M) -> Self {
        self.modules.push(Box::new(module));
        self
    }

    pub fn compose(mut self, mut other: Configuration) -> Self {
        self.modules.append(&mut other.modules);
        self
    }
}

impl Module for Configuration {
    fn configure(
        &self,
        configurer: &mut dyn Configurer,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.modules
            .iter()
            .for_each(|module| module.setup(configurer));
        Ok(())
    }
}
