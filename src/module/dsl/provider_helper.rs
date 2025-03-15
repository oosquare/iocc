use std::fmt::Debug;
use std::hash::Hash;

use crate::container::registry::Configurer;
use crate::container::{Managed, SharedManaged};
use crate::key::TypedKey;
use crate::module::dsl::ToLifetime;
use crate::provider::{TypedProvider, TypedSharedProvider};
use crate::scope::{Scope, Transient};

#[allow(private_bounds)]
pub struct ProviderBinding<KT, KQ, L, P>
where
    KT: Managed,
    KQ: Copy + Debug + Eq + Hash + Send + Sync + 'static,
    L: ToLifetime,
    P: TypedProvider<Key: TypedKey<Target = KT>>,
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
    P: TypedProvider<Key: TypedKey<Target = KT>>,
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
    P: TypedSharedProvider<Key: TypedKey<Target = KT, Qualifier = KQ>>,
{
    pub fn set_on(self, configurer: &mut dyn Configurer<Scope = S>) {
        configurer.register_shared(Box::new(self.provider), self.lifetime);
    }
}

impl<KT, KQ, P> ProviderBinding<KT, KQ, Transient, P>
where
    KT: Managed,
    KQ: Copy + Debug + Eq + Hash + Send + Sync + 'static,
    P: TypedProvider<Key: TypedKey<Target = KT, Qualifier = KQ>>,
{
    pub fn set_on<S>(self, configurer: &mut dyn Configurer<Scope = S>)
    where
        S: Scope,
    {
        configurer.register(Box::new(self.provider));
    }
}
