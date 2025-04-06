use crate::container::registry::{Configurer, TypedConfigurer};
use crate::container::{Managed, SharedManaged};
use crate::key::{self, TypedQualifier};
use crate::module::dsl::ToLifetime;
use crate::provider::closure::{RawClosure, RawClosureProvider};
use crate::scope::{Scope, Transient};

#[allow(private_bounds)]
pub struct RawClosureBinding<KT, KQ, L, C>
where
    KT: Managed,
    KQ: TypedQualifier,
    L: ToLifetime,
    C: RawClosure<Constructed = KT>,
{
    closure: C,
    qualifier: KQ,
    lifetime: L,
}

#[allow(private_bounds)]
impl<KT, KQ, L, C> RawClosureBinding<KT, KQ, L, C>
where
    KT: Managed,
    KQ: TypedQualifier,
    L: ToLifetime,
    C: RawClosure<Constructed = KT>,
{
    pub(super) fn new(closure: C, qualifier: KQ, lifetime: L) -> Self {
        Self {
            closure,
            qualifier,
            lifetime,
        }
    }

    pub fn qualified_by<NewKQ>(self, qualifier: NewKQ) -> RawClosureBinding<KT, NewKQ, L, C>
    where
        NewKQ: TypedQualifier,
    {
        RawClosureBinding::new(self.closure, qualifier, self.lifetime)
    }

    pub fn within<NewS>(self, scope: NewS) -> RawClosureBinding<KT, KQ, NewS, C>
    where
        NewS: Scope,
    {
        RawClosureBinding::new(self.closure, self.qualifier, scope)
    }

    pub fn as_transient(self) -> RawClosureBinding<KT, KQ, Transient, C> {
        RawClosureBinding::new(self.closure, self.qualifier, Transient)
    }
}

impl<KT, KQ, S, C> RawClosureBinding<KT, KQ, S, C>
where
    KT: SharedManaged,
    KQ: TypedQualifier,
    S: Scope,
    C: RawClosure<Constructed = KT>,
{
    pub fn set_on(self, configurer: &mut dyn Configurer<Scope = S>) {
        let key = key::qualified(self.qualifier);
        let provider = RawClosureProvider::new(self.closure);
        configurer.register_shared(key, provider, self.lifetime);
    }
}

impl<KT, KQ, C> RawClosureBinding<KT, KQ, Transient, C>
where
    KT: Managed,
    KQ: TypedQualifier,
    C: RawClosure<Constructed = KT>,
{
    pub fn set_on<S>(self, configurer: &mut dyn Configurer<Scope = S>)
    where
        S: Scope,
    {
        let key = key::qualified(self.qualifier);
        let provider = RawClosureProvider::new(self.closure);
        configurer.register(key, provider);
    }
}
