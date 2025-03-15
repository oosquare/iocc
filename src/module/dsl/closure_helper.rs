use std::fmt::Debug;
use std::hash::Hash;

use crate::container::injector::{Injector, InjectorError};
use crate::container::registry::Configurer;
use crate::container::{Managed, SharedManaged};
use crate::key;
use crate::module::dsl::ToLifetime;
use crate::provider::closure::ClosureProvider;
use crate::scope::{Scope, Transient};

#[allow(private_bounds)]
pub struct ClosureBinding<KT, KQ, L, C>
where
    KT: Managed,
    KQ: Copy + Debug + Eq + Hash + Send + Sync + 'static,
    L: ToLifetime,
    C: Fn(&dyn Injector) -> Result<KT, InjectorError> + Send + Sync + 'static,
{
    closure: C,
    qualifier: KQ,
    lifetime: L,
}

#[allow(private_bounds)]
impl<KT, KQ, L, C> ClosureBinding<KT, KQ, L, C>
where
    KT: Managed,
    KQ: Copy + Debug + Eq + Hash + Send + Sync + 'static,
    L: ToLifetime,
    C: Fn(&dyn Injector) -> Result<KT, InjectorError> + Send + Sync + 'static,
{
    pub(super) fn new(closure: C, qualifier: KQ, lifetime: L) -> Self {
        Self {
            closure,
            qualifier,
            lifetime,
        }
    }

    pub fn qualified_by<NewKQ>(self, qualifier: NewKQ) -> ClosureBinding<KT, NewKQ, L, C>
    where
        NewKQ: Copy + Debug + Eq + Hash + Send + Sync + 'static,
    {
        ClosureBinding::new(self.closure, qualifier, self.lifetime)
    }

    pub fn within<NewS>(self, scope: NewS) -> ClosureBinding<KT, KQ, NewS, C>
    where
        NewS: Scope,
    {
        ClosureBinding::new(self.closure, self.qualifier, scope)
    }

    pub fn as_transient(self) -> ClosureBinding<KT, KQ, Transient, C> {
        ClosureBinding::new(self.closure, self.qualifier, Transient)
    }
}

impl<KT, KQ, S, C> ClosureBinding<KT, KQ, S, C>
where
    KT: SharedManaged,
    KQ: Copy + Debug + Eq + Hash + Send + Sync + 'static,
    S: Scope,
    C: Fn(&dyn Injector) -> Result<KT, InjectorError> + Send + Sync + 'static,
{
    pub fn set_on(self, configurer: &mut dyn Configurer<Scope = S>) {
        let key = key::qualified::<KT, _>(self.qualifier);
        let provider = ClosureProvider::new(key, self.closure);
        configurer.register_shared(Box::new(provider), self.lifetime);
    }
}

impl<KT, KQ, C> ClosureBinding<KT, KQ, Transient, C>
where
    KT: Managed,
    KQ: Copy + Debug + Eq + Hash + Send + Sync + 'static,
    C: Fn(&dyn Injector) -> Result<KT, InjectorError> + Send + Sync + 'static,
{
    pub fn set_on<S>(self, configurer: &mut dyn Configurer<Scope = S>)
    where
        S: Scope,
    {
        let key = key::qualified::<KT, _>(self.qualifier);
        let provider = ClosureProvider::new(key, self.closure);
        configurer.register(Box::new(provider));
    }
}
