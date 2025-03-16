use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;
use std::sync::Arc;

use crate::container::registry::{Configurer, TypedConfigurer};
use crate::container::{Managed, SharedManaged};
use crate::key;
use crate::module::dsl::component_helper::ComponentBinding;
use crate::module::dsl::instance_helper::InstanceBinding;
use crate::module::dsl::provider_helper::ProviderBinding;
use crate::module::dsl::raw_closure_helper::RawClosureBinding;
use crate::module::dsl::ToLifetime;
use crate::provider::closure::RawClosure;
use crate::provider::component::{Component, ComponentProvider};
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
        C: Component<Constructed = KT>,
    {
        ComponentBinding::new(self.qualifier, self.lifetime)
    }

    pub fn to_raw_closure<C>(self, closure: C) -> RawClosureBinding<KT, KQ, L, C>
    where
        C: RawClosure<Constructed = KT>,
    {
        RawClosureBinding::new(closure, self.qualifier, self.lifetime)
    }

    pub fn to_instance(self, instance: KT) -> InstanceBinding<KT, KQ, L>
    where
        KT: Clone,
    {
        InstanceBinding::new(instance, self.qualifier, self.lifetime)
    }

    pub fn to_provider<P>(self, provider: P) -> ProviderBinding<KT, KQ, L, P>
    where
        P: TypedProvider<Output = KT>,
    {
        ProviderBinding::new(provider, self.qualifier, self.lifetime)
    }
}

impl<KT, KQ, S> MetadataBinding<KT, KQ, S>
where
    KT: SharedManaged + Component<Constructed = KT>,
    KQ: Copy + Debug + Eq + Hash + Send + Sync + 'static,
    S: Scope,
{
    pub fn set_on(self, configurer: &mut dyn Configurer<Scope = S>) {
        let key = key::qualified::<KT, _>(self.qualifier);
        let provider = ComponentProvider::<KT>::new();
        configurer.register_shared(key, provider, self.lifetime);
    }
}

impl<C, KQ, S> MetadataBinding<Arc<C>, KQ, S>
where
    C: Component<Constructed = Arc<C>>,
    KQ: Copy + Debug + Eq + Hash + Send + Sync + 'static,
    S: Scope,
{
    pub fn set_on(self, configurer: &mut dyn Configurer<Scope = S>) {
        let key = key::qualified::<Arc<C>, _>(self.qualifier);
        let provider = ComponentProvider::<C>::new();
        configurer.register_shared(key, provider, self.lifetime);
    }
}

impl<KT, KQ> MetadataBinding<KT, KQ, Transient>
where
    KT: Component<Constructed = KT>,
    KQ: Copy + Debug + Eq + Hash + Send + Sync + 'static,
{
    pub fn set_on<S>(self, configurer: &mut dyn Configurer<Scope = S>)
    where
        S: Scope,
    {
        let key = key::qualified::<KT, _>(self.qualifier);
        let provider = ComponentProvider::<KT>::new();
        configurer.register(key, provider);
    }
}
