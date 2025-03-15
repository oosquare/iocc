pub mod closure_helper;
pub mod component_helper;
pub mod instance_helper;
pub mod metadata_helper;
pub mod provider_helper;

use metadata_helper::MetadataBinding;

use crate::container::Managed;
use crate::scope::{Scope, Transient};

trait ToLifetime {}

impl<S: Scope> ToLifetime for S {}

impl ToLifetime for Transient {}

#[allow(private_bounds)]
pub fn bind<KT>() -> MetadataBinding<KT, (), Transient>
where
    KT: Managed,
{
    MetadataBinding::new((), Transient)
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;
    use std::error::Error;
    use std::sync::Arc;

    use crate::component::Component;
    use crate::container::injector::{InjectorError, TypedInjector};
    use crate::container::registry::Configurer;
    use crate::key;
    use crate::module::Module;
    use crate::provider::instance::InstanceProvider;
    use crate::scope::WebScope;

    use super::*;

    struct DslCompilationTest;

    impl Module for DslCompilationTest {
        type Scope = WebScope;

        fn configure(
            &self,
            configurer: &mut dyn Configurer<Scope = Self::Scope>,
        ) -> Result<(), Box<dyn Error + Send + Sync>> {
            bind::<TestObject>()
                .qualified_by(1)
                .as_transient()
                .set_on(configurer);

            bind::<Arc<dyn TestTrait>>()
                .qualified_by("qualifier")
                .within(WebScope::Singleton)
                .to_component::<TestDynObject>()
                .set_on(configurer);

            bind::<i64>()
                .to_closure(|_| Ok(42))
                .qualified_by("i64")
                .set_on(configurer);

            bind::<Arc<i32>>()
                .to_instance(Arc::new(42))
                .within(WebScope::Singleton)
                .set_on(configurer);

            bind::<&'static str>()
                .to_provider(InstanceProvider::new(key::of(), "str"))
                .set_on(configurer);

            Ok(())
        }
    }

    trait TestTrait: Send + Sync + 'static {}

    struct TestDynObject;

    impl TestTrait for TestDynObject {}

    impl Component for TestDynObject {
        type Output = Arc<dyn TestTrait>;

        type Error = Infallible;

        fn construct<I>(_injector: &I) -> Result<Result<Self, Self::Error>, InjectorError>
        where
            I: TypedInjector + ?Sized,
        {
            Ok(Ok(Self))
        }

        fn post_process(self) -> Self::Output {
            Arc::new(self)
        }
    }

    struct TestObject;

    impl Component for TestObject {
        type Output = Self;

        type Error = Infallible;

        fn construct<I>(_injector: &I) -> Result<Result<Self, Self::Error>, InjectorError>
        where
            I: TypedInjector + ?Sized,
        {
            Ok(Ok(Self))
        }

        fn post_process(self) -> Self::Output {
            Self
        }
    }
}
