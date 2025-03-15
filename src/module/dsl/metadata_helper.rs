use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;
use std::sync::Arc;

use crate::component::Component;
use crate::container::injector::{Injector, InjectorError};
use crate::container::registry::Configurer;
use crate::container::{Managed, SharedManaged};
use crate::key::{self, TypedKey};
use crate::module::dsl::closure_helper::ClosureBinding;
use crate::module::dsl::component_helper::ComponentBinding;
use crate::module::dsl::instance_helper::InstanceBinding;
use crate::module::dsl::provider_helper::ProviderBinding;
use crate::module::dsl::ToLifetime;
use crate::provider::component::ComponentProvider;
use crate::provider::TypedProvider;
use crate::scope::{Scope, Transient};

#[allow(private_bounds)]
pub struct MetadataBinding<KT, KQ, L>
where
    KT: Managed,
    KQ: Copy + Debug + Eq + Hash + Send + Sync + 'static,
    L: ToLifetime,
{
    qualifier: KQ,
    lifetime: L,
    _marker: PhantomData<KT>,
}

#[allow(private_bounds)]
impl<KT, KQ, L> MetadataBinding<KT, KQ, L>
where
    KT: Managed,
    KQ: Copy + Debug + Eq + Hash + Send + Sync + 'static,
    L: ToLifetime,
{
    pub(super) fn new(qualifier: KQ, lifetime: L) -> Self {
        Self {
            qualifier,
            lifetime,
            _marker: PhantomData,
        }
    }

    pub fn qualified_by<NewKQ>(self, qualifier: NewKQ) -> MetadataBinding<KT, NewKQ, L>
    where
        NewKQ: Copy + Debug + Eq + Hash + Send + Sync + 'static,
    {
        MetadataBinding::new(qualifier, self.lifetime)
    }

    pub fn within<NewS>(self, scope: NewS) -> MetadataBinding<KT, KQ, NewS>
    where
        NewS: Scope,
    {
        MetadataBinding::new(self.qualifier, scope)
    }

    pub fn as_transient(self) -> MetadataBinding<KT, KQ, Transient> {
        MetadataBinding::new(self.qualifier, Transient)
    }

    pub fn to_component<C>(self) -> ComponentBinding<C, KQ, L>
    where
        C: Component<Output = KT>,
    {
        ComponentBinding::new(self.qualifier, self.lifetime)
    }

    pub fn to_closure<C>(self, closure: C) -> ClosureBinding<KT, KQ, L, C>
    where
        C: Fn(&dyn Injector) -> Result<KT, InjectorError> + Send + Sync + 'static,
    {
        ClosureBinding::new(closure, self.qualifier, self.lifetime)
    }

    pub fn to_instance(self, instance: KT) -> InstanceBinding<KT, KQ, L>
    where
        KT: Clone,
    {
        InstanceBinding::new(instance, self.qualifier, self.lifetime)
    }

    pub fn to_provider<P>(self, provider: P) -> ProviderBinding<KT, KQ, L, P>
    where
        P: TypedProvider<Key: TypedKey<Target = KT>>,
    {
        ProviderBinding::new(provider, self.qualifier, self.lifetime)
    }
}

impl<KT, KQ, S> MetadataBinding<KT, KQ, S>
where
    KT: SharedManaged + Component<Output = KT>,
    KQ: Copy + Debug + Eq + Hash + Send + Sync + 'static,
    S: Scope,
{
    pub fn set_on(self, configurer: &mut dyn Configurer<Scope = S>) {
        let key = key::qualified::<KT, _>(self.qualifier);
        let provider = ComponentProvider::<_, KT>::new(key);
        configurer.register_shared(Box::new(provider), self.lifetime);
    }
}

impl<C, KQ, S> MetadataBinding<Arc<C>, KQ, S>
where
    C: Component<Output = Arc<C>>,
    KQ: Copy + Debug + Eq + Hash + Send + Sync + 'static,
    S: Scope,
{
    pub fn set_on(self, configurer: &mut dyn Configurer<Scope = S>) {
        let key = key::qualified::<Arc<C>, _>(self.qualifier);
        let provider = ComponentProvider::<_, C>::new(key);
        configurer.register_shared(Box::new(provider), self.lifetime);
    }
}

impl<KT, KQ> MetadataBinding<KT, KQ, Transient>
where
    KT: Component<Output = KT>,
    KQ: Copy + Debug + Eq + Hash + Send + Sync + 'static,
{
    pub fn set_on<S>(self, configurer: &mut dyn Configurer<Scope = S>)
    where
        S: Scope,
    {
        let key = key::qualified::<KT, _>(self.qualifier);
        let provider = ComponentProvider::<_, KT>::new(key);
        configurer.register(Box::new(provider));
    }
}
