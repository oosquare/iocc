use std::collections::HashMap;
use std::convert::Infallible;
use std::error::Error;

use iocc::prelude::*;
use iocc::scope::SingletonScope;

#[derive(Debug, PartialEq)]
pub struct Test {
    pub a: i32,
    pub b: &'static str,
    pub t: (i64, f64),
    pub m: HashMap<i32, &'static str>,
}

#[component]
impl Test {
    #[inject]
    pub fn new(
        a: i32,
        #[qualified(TestQualifier::Greet)] b: &'static str,
        #[named("tuple")] (c, d): (i64, f64),
        #[collect(key)] m: HashMap<i32, &'static str>,
    ) -> Self {
        Self { a, b, t: (c, d), m }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum TestQualifier {
    Greet,
}

struct TestModule;

impl Module for TestModule {
    type Scope = SingletonScope;

    fn configure(
        &self,
        configurer: &mut dyn Configurer<Scope = Self::Scope>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        bind::<i32>().to_instance(42).set_on(configurer);

        bind::<i64>().to_instance(64).set_on(configurer);

        bind::<f64>().to_instance(3.1415926).set_on(configurer);

        bind::<&'static str>()
            .to_instance("hello world")
            .qualified_by(TestQualifier::Greet)
            .set_on(configurer);

        bind::<&'static str>()
            .to_instance("str")
            .qualified_by(1)
            .set_on(configurer);

        bind::<&'static str>()
            .to_instance("abcdefg")
            .qualified_by(2)
            .set_on(configurer);

        bind::<(i64, f64)>()
            .to_closure(|c: i64, d: f64| Ok::<_, Infallible>((c, d)))
            .qualified_by("tuple")
            .set_on(configurer);

        bind::<Test>().set_on(configurer);

        Ok(())
    }
}

fn main() {
    let container = Container::init(TestModule).unwrap();
    let obj: Test = container.get(key::of()).unwrap();

    assert_eq!(
        dbg!(obj),
        Test::new(
            42,
            "hello world",
            (64, 3.1415926),
            HashMap::from([(1, "str"), (2, "abcdefg")]),
        )
    );
}
