use std::fmt::Debug;
use std::hash::Hash;

use crate::container::registry::{Configurer, TypedConfigurer};
use crate::container::{Managed, SharedManaged};
use crate::key;
use crate::module::dsl::ToLifetime;
use crate::provider::{TypedProvider, TypedSharedProvider};
use crate::scope::{Scope, Transient};

#[allow(private_bounds)]
pub struct ProviderBinding<KT, KQ, L, P>
where
    KT: Managed,
    KQ: Copy + Debug + Eq + Hash + Send + Sync + 'static,
    L: ToLifetime,
    P: TypedProvider<Output = KT>,
{
    provider: P,
    qualifier: KQ,
    lifetime: L,
}

#[allow(private_bounds)]
impl<KT, KQ, L, P> ProviderBinding<KT, KQ, L, P>
where
    KT: Managed,
    KQ: Copy + Debug + Eq + Hash + Send + Sync + 'static,
    L: ToLifetime,
    P: TypedProvider<Output = KT>,
{
    pub(super) fn new(provider: P, qualifier: KQ, lifetime: L) -> Self {
        Self {
            provider,
            qualifier,
            lifetime,
        }
    }

    pub fn qualified_by<NewKQ>(self, qualifier: NewKQ) -> ProviderBinding<KT, NewKQ, L, P>
    where
        NewKQ: Copy + Debug + Eq + Hash + Send + Sync + 'static,
    {
        ProviderBinding::new(self.provider, qualifier, self.lifetime)
    }

    pub fn within<NewS>(self, scope: NewS) -> ProviderBinding<KT, KQ, NewS, P>
    where
        NewS: Scope,
    {
        ProviderBinding::new(self.provider, self.qualifier, scope)
    }

    pub fn as_transient(self) -> ProviderBinding<KT, KQ, Transient, P> {
        ProviderBinding::new(self.provider, self.qualifier, Transient)
    }
}

impl<KT, KQ, S, P> ProviderBinding<KT, KQ, S, P>
where
    KT: SharedManaged,
    KQ: Copy + Debug + Eq + Hash + Send + Sync + 'static,
    S: Scope,
    P: TypedSharedProvider<Output = KT>,
{
    pub fn set_on(self, configurer: &mut dyn Configurer<Scope = S>) {
        let key = key::qualified::<KT, _>(self.qualifier);
        configurer.register_shared(key, self.provider, self.lifetime);
    }
}

impl<KT, KQ, P> ProviderBinding<KT, KQ, Transient, P>
where
    KT: Managed,
    KQ: Copy + Debug + Eq + Hash + Send + Sync + 'static,
    P: TypedProvider<Output = KT>,
{
    pub fn set_on<S>(self, configurer: &mut dyn Configurer<Scope = S>)
    where
        S: Scope,
    {
        let key = key::qualified::<KT, _>(self.qualifier);
        configurer.register(key, self.provider);
    }
}
