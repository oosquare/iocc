pub mod closure_helper;
pub mod component_helper;
pub mod instance_helper;
pub mod metadata_helper;
pub mod provider_helper;
pub mod raw_closure_helper;

use metadata_helper::MetadataBinding;

use crate::container::Managed;
use crate::key::TypedKey;
use crate::scope::{Scope, Transient};

#[allow(private_bounds)]
pub trait ToLifetime: Sealed {}

impl<S: Scope> ToLifetime for S {}

impl ToLifetime for Transient {}

trait Sealed {}

impl<S: Scope> Sealed for S {}

impl Sealed for Transient {}

pub fn bind<KT>() -> MetadataBinding<KT, (), Transient>
where
    KT: Managed,
{
    MetadataBinding::new((), Transient)
}

pub fn bind_key<K>(key: K) -> MetadataBinding<K::Target, K::Qualifier, Transient>
where
    K: TypedKey,
{
    MetadataBinding::new(key.qualifier(), Transient)
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;
    use std::error::Error;
    use std::sync::Arc;

    use crate::container::injector::{InjectorError, TypedInjector};
    use crate::container::registry::Configurer;
    use crate::module::Module;
    use crate::provider::component::Component;
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

            bind::<(TestObject, Arc<dyn TestTrait>)>()
                .to_closure(|a: TestObject, b: Arc<dyn TestTrait>| Ok::<_, Infallible>((a, b)))
                .as_transient();

            bind::<i64>()
                .to_raw_closure(|_| Ok(Ok::<_, Infallible>(42)))
                .qualified_by("i64")
                .set_on(configurer);

            bind::<Arc<i32>>()
                .to_instance(Arc::new(42))
                .within(WebScope::Singleton)
                .set_on(configurer);

            bind::<&'static str>()
                .to_provider(InstanceProvider::new("str"))
                .set_on(configurer);

            Ok(())
        }
    }

    trait TestTrait: Send + Sync + 'static {}

    struct TestDynObject;

    impl TestTrait for TestDynObject {}

    impl Component for TestDynObject {
        type Constructed = Arc<dyn TestTrait>;

        type Error = Infallible;

        fn construct<I>(_injector: &I) -> Result<Result<Self, Self::Error>, InjectorError>
        where
            I: TypedInjector + ?Sized,
        {
            Ok(Ok(Self))
        }

        fn post_process(self) -> Self::Constructed {
            Arc::new(self)
        }
    }

    struct TestObject;

    impl Component for TestObject {
        type Constructed = Self;

        type Error = Infallible;

        fn construct<I>(_injector: &I) -> Result<Result<Self, Self::Error>, InjectorError>
        where
            I: TypedInjector + ?Sized,
        {
            Ok(Ok(Self))
        }

        fn post_process(self) -> Self::Constructed {
            Self
        }
    }
}
