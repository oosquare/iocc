use std::marker::PhantomData;

use crate::container::registry::{Configurer, TypedConfigurer};
use crate::container::{Managed, SharedManaged};
use crate::key::{self, TypedQualifier};
use crate::module::dsl::ToLifetime;
use crate::provider::closure::{Closure, ClosureProvider};
use crate::scope::{Scope, Transient};

#[allow(private_bounds)]
pub struct ClosureBinding<KT, KQ, L, C, D>
where
    KT: Managed,
    KQ: TypedQualifier,
    L: ToLifetime,
    C: Closure<D, Constructed = KT>,
    D: Send + Sync + 'static,
{
    closure: C,
    qualifier: KQ,
    lifetime: L,
    _marker: PhantomData<(KT, D)>,
}

#[allow(private_bounds)]
impl<KT, KQ, L, C, D> ClosureBinding<KT, KQ, L, C, D>
where
    KT: Managed,
    KQ: TypedQualifier,
    L: ToLifetime,
    C: Closure<D, Constructed = KT>,
    D: Send + Sync + 'static,
{
    pub(super) fn new(closure: C, qualifier: KQ, lifetime: L) -> Self {
        Self {
            closure,
            qualifier,
            lifetime,
            _marker: PhantomData,
        }
    }

    pub fn qualified_by<NewKQ>(self, qualifier: NewKQ) -> ClosureBinding<KT, NewKQ, L, C, D>
    where
        NewKQ: TypedQualifier,
    {
        ClosureBinding::new(self.closure, qualifier, self.lifetime)
    }

    pub fn within<NewS>(self, scope: NewS) -> ClosureBinding<KT, KQ, NewS, C, D>
    where
        NewS: Scope,
    {
        ClosureBinding::new(self.closure, self.qualifier, scope)
    }

    pub fn as_transient(self) -> ClosureBinding<KT, KQ, Transient, C, D> {
        ClosureBinding::new(self.closure, self.qualifier, Transient)
    }
}

impl<KT, KQ, S, C, D> ClosureBinding<KT, KQ, S, C, D>
where
    KT: SharedManaged,
    KQ: TypedQualifier,
    S: Scope,
    C: Closure<D, Constructed = KT>,
    D: Send + Sync + 'static,
{
    pub fn set_on(self, configurer: &mut dyn Configurer<Scope = S>) {
        let key = key::qualified(self.qualifier);
        let provider = ClosureProvider::new(self.closure);
        configurer.register_shared(key, provider, self.lifetime);
    }
}

impl<KT, KQ, C, D> ClosureBinding<KT, KQ, Transient, C, D>
where
    KT: Managed,
    KQ: TypedQualifier,
    C: Closure<D, Constructed = KT>,
    D: Send + Sync + 'static,
{
    pub fn set_on<S>(self, configurer: &mut dyn Configurer<Scope = S>)
    where
        S: Scope,
    {
        let key = key::qualified(self.qualifier);
        let provider = ClosureProvider::new(self.closure);
        configurer.register(key, provider);
    }
}
