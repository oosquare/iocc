use std::marker::PhantomData;

use crate::container::registry::{Configurer, TypedConfigurer};
use crate::container::SharedManaged;
use crate::key::{self, TypedQualifier};
use crate::module::dsl::ToLifetime;
use crate::provider::component::{Component, ComponentProvider};
use crate::scope::{Scope, Transient};

#[allow(private_bounds)]
pub struct ComponentBinding<C, KQ, L>
where
    C: Component,
    KQ: TypedQualifier,
    L: ToLifetime,
{
    qualifier: KQ,
    lifetime: L,
    _marker: PhantomData<C>,
}

#[allow(private_bounds)]
impl<C, KQ, L> ComponentBinding<C, KQ, L>
where
    C: Component,
    KQ: TypedQualifier,
    L: ToLifetime,
{
    pub(super) fn new(qualifier: KQ, lifetime: L) -> Self {
        Self {
            qualifier,
            lifetime,
            _marker: PhantomData,
        }
    }

    pub fn qualified_by<NewKQ>(self, qualifier: NewKQ) -> ComponentBinding<C, NewKQ, L>
    where
        NewKQ: TypedQualifier,
    {
        ComponentBinding::new(qualifier, self.lifetime)
    }

    pub fn within<NewS>(self, scope: NewS) -> ComponentBinding<C, KQ, NewS>
    where
        NewS: Scope,
    {
        ComponentBinding::new(self.qualifier, scope)
    }

    pub fn as_transient(self) -> ComponentBinding<C, KQ, Transient> {
        ComponentBinding::new(self.qualifier, Transient)
    }
}

impl<C, KQ, S> ComponentBinding<C, KQ, S>
where
    C: Component<Constructed: SharedManaged>,
    KQ: TypedQualifier,
    S: Scope,
{
    pub fn set_on(self, configurer: &mut dyn Configurer<Scope = S>) {
        let key = key::qualified(self.qualifier);
        let provider = ComponentProvider::<C>::new();
        configurer.register_shared(key, provider, self.lifetime);
    }
}

impl<C, KQ> ComponentBinding<C, KQ, Transient>
where
    C: Component,
    KQ: TypedQualifier,
{
    pub fn set_on<S>(self, configurer: &mut dyn Configurer<Scope = S>)
    where
        S: Scope,
    {
        let key = key::qualified(self.qualifier);
        let provider = ComponentProvider::<C>::new();
        configurer.register(key, provider);
    }
}
