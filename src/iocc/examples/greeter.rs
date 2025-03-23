use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

use iocc::prelude::*;
use iocc::scope::SingletonScope;

fn main() {
    let container = Container::init(AppModule::new("greeter")).unwrap();
    let app = container.get(key::of::<Arc<App>>()).unwrap();
    app.run();
}

struct AppModule {
    app_name: &'static str,
}

impl AppModule {
    fn new(app_name: &'static str) -> Self {
        Self { app_name }
    }
}

impl Module for AppModule {
    type Scope = SingletonScope;

    fn configure(
        &self,
        configurer: &mut dyn Configurer<Scope = Self::Scope>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        bind::<&'static str>()
            .to_instance(self.app_name)
            .qualified_by("app_name")
            .set_on(configurer);

        bind::<Arc<dyn Logger>>()
            .to_component::<ConsoleLogger>()
            .within(SingletonScope)
            .set_on(configurer);

        bind::<Arc<dyn Greeter>>()
            .to_component::<EnglishGreeter>()
            .qualified_by(GreeterKind::English)
            .within(SingletonScope)
            .set_on(configurer);

        bind::<Arc<dyn Greeter>>()
            .to_component::<ChineseGreeter>()
            .qualified_by(GreeterKind::Chinese)
            .within(SingletonScope)
            .set_on(configurer);

        bind::<Arc<App>>().within(SingletonScope).set_on(configurer);

        Ok(())
    }
}

trait Logger: Send + Sync + 'static {
    fn log(&self, message: &str);
}

struct ConsoleLogger {
    app_name: &'static str,
}

#[component(Arc<dyn Logger>, Arc::new)]
impl ConsoleLogger {
    #[inject]
    pub fn new(#[named("app_name")] app_name: &'static str) -> Self {
        Self { app_name }
    }
}

impl Logger for ConsoleLogger {
    fn log(&self, message: &str) {
        eprintln!("[{}] {}", self.app_name, message);
    }
}

trait Greeter: Send + Sync + 'static {
    fn greet(&self);
}

struct EnglishGreeter {
    logger: Arc<dyn Logger>,
}

#[component(Arc<dyn Greeter>, Arc::new)]
impl EnglishGreeter {
    #[inject]
    fn new(logger: Arc<dyn Logger>) -> Self {
        Self { logger }
    }
}

impl Greeter for EnglishGreeter {
    fn greet(&self) {
        self.logger.log("Hello World!");
    }
}

struct ChineseGreeter {
    logger: Arc<dyn Logger>,
}

#[component(Arc<dyn Greeter>, Arc::new)]
impl ChineseGreeter {
    #[inject]
    fn new(logger: Arc<dyn Logger>) -> Self {
        Self { logger }
    }
}

impl Greeter for ChineseGreeter {
    fn greet(&self) {
        self.logger.log("你好世界!");
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GreeterKind {
    English,
    Chinese,
}

struct App {
    logger: Arc<dyn Logger>,
    greeters: HashMap<GreeterKind, Arc<dyn Greeter>>,
}

#[component(Arc<App>, Arc::new)]
impl App {
    #[inject]
    fn new(
        logger: Arc<dyn Logger>,
        #[collect(key)] greeters: HashMap<GreeterKind, Arc<dyn Greeter>>,
    ) -> Self {
        Self { logger, greeters }
    }

    fn run(&self) {
        self.logger.log("Greeting from IOCC managed objects:");
        for greeter in self.greeters.values() {
            greeter.greet();
        }
    }
}
