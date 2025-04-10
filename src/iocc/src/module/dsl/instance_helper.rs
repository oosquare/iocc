use crate::container::registry::{Configurer, TypedConfigurer};
use crate::container::{Managed, SharedManaged};
use crate::key::{self, TypedQualifier};
use crate::module::dsl::ToLifetime;
use crate::provider::instance::InstanceProvider;
use crate::scope::{Scope, Transient};

#[allow(private_bounds)]
pub struct InstanceBinding<KT, KQ, L>
where
    KT: Managed + Clone,
    KQ: TypedQualifier,
    L: ToLifetime,
{
    instance: KT,
    qualifier: KQ,
    lifetime: L,
}

#[allow(private_bounds)]
impl<KT, KQ, L> InstanceBinding<KT, KQ, L>
where
    KT: Managed + Clone,
    KQ: TypedQualifier,
    L: ToLifetime,
{
    pub(super) fn new(instance: KT, qualifier: KQ, lifetime: L) -> Self {
        Self {
            instance,
            qualifier,
            lifetime,
        }
    }

    pub fn qualified_by<NewKQ>(self, qualifier: NewKQ) -> InstanceBinding<KT, NewKQ, L>
    where
        NewKQ: TypedQualifier,
    {
        InstanceBinding::new(self.instance, qualifier, self.lifetime)
    }

    pub fn within<NewS>(self, scope: NewS) -> InstanceBinding<KT, KQ, NewS>
    where
        NewS: Scope,
    {
        InstanceBinding::new(self.instance, self.qualifier, scope)
    }

    pub fn as_transient(self) -> InstanceBinding<KT, KQ, Transient> {
        InstanceBinding::new(self.instance, self.qualifier, Transient)
    }
}

impl<KT, KQ, S> InstanceBinding<KT, KQ, S>
where
    KT: SharedManaged + Clone,
    KQ: TypedQualifier,
    S: Scope,
{
    pub fn set_on(self, configurer: &mut dyn Configurer<Scope = S>) {
        let key = key::qualified(self.qualifier);
        let provider = InstanceProvider::new(self.instance);
        configurer.register_shared(key, provider, self.lifetime);
    }
}

impl<KT, KQ> InstanceBinding<KT, KQ, Transient>
where
    KT: Managed + Clone,
    KQ: TypedQualifier,
{
    pub fn set_on<S>(self, configurer: &mut dyn Configurer<Scope = S>)
    where
        S: Scope,
    {
        let key = key::qualified(self.qualifier);
        let provider = InstanceProvider::new(self.instance);
        configurer.register(key, provider);
    }
}
