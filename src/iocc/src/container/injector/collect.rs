use std::any::{self, TypeId};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, LinkedList, VecDeque};
use std::hash::Hash;

use crate::container::injector::{InjectorError, TypedInjector};
use crate::container::Managed;
use crate::key::{Key, Pattern, Qualifier, TypedQualifier};
use crate::util::any::{Downcast, DowncastRef};

pub trait Collect<P>: Sized
where
    P: Pattern,
{
    fn collect<'a, I, KI>(injector: &I, keys: KI, pattern: P) -> Result<Self, InjectorError>
    where
        I: TypedInjector + ?Sized,
        KI: Iterator<Item = &'a dyn Key>;
}

macro_rules! impl_collect_for_non_map_collections {
    ($collection:ident, [$($bounds:ident),*]) => {
        impl<T, P> Collect<P> for $collection<T>
        where
            $(T: $bounds,)*
            P: Pattern<Target = T>,
        {
            fn collect<'a, I, KI>(injector: &I, keys: KI, pattern: P) -> Result<Self, InjectorError>
            where
                I: TypedInjector + ?Sized,
                KI: Iterator<Item = &'a dyn Key>,
            {
                let collection = keys.filter(|key| key.target_type() == TypeId::of::<T>())
                    .filter(move |key| pattern.matches(*key))
                    .map(|key| {
                        injector.dyn_get(key).map(|object| {
                            *object
                                .downcast::<T>()
                                .unwrap_or_else(|_| unreachable!("in impl `Collect<P>` for non-maps, `object` should be `Box<T>`"))
                        })
                    })
                    .collect::<Result<Self, InjectorError>>()?;

                if !collection.is_empty() {
                    Ok(collection)
                } else {
                    Err(InjectorError::EmptyCollection {
                        collection: any::type_name::<Self>(),
                        pattern: any::type_name::<P>(),
                    })
                }
            }
        }
    };
}

impl_collect_for_non_map_collections!(Vec, [Managed]);
impl_collect_for_non_map_collections!(VecDeque, [Managed]);
impl_collect_for_non_map_collections!(LinkedList, [Managed]);

impl_collect_for_non_map_collections!(HashSet, [Managed, Eq, Hash]);
impl_collect_for_non_map_collections!(BTreeSet, [Managed, Ord]);

macro_rules! impl_collect_for_maps {
    ($collection:ident, [$($bounds:ident),*]) => {
        impl<T, Q, P> Collect<P> for $collection<Q, T>
        where
            T: Managed,
            $(Q: $bounds,)*
            P: Pattern<Target = T, Qualifier = Q>,
        {
            fn collect<'a, I, KI>(injector: &I, keys: KI, pattern: P) -> Result<Self, InjectorError>
            where
                I: TypedInjector + ?Sized,
                KI: Iterator<Item = &'a dyn Key>,
            {
                let collection = keys.filter(|key| key.target_type() == TypeId::of::<T>())
                    .filter(|key| key.qualifier_type() == TypeId::of::<Q>())
                    .filter(move |key| pattern.matches(*key))
                    .map(|key| {
                        let qualifier = *key.dyn_qualifier().downcast_ref::<Q>().unwrap_or_else(|| {
                            unreachable!("in impl `Collect<P>` for maps, `qualifier` should be `Q`")
                        });
                        let res = injector.dyn_get(key).map(|object| {
                            *object.downcast::<T>().unwrap_or_else(|_| {
                                unreachable!("in impl `Collect<P>` for maps, `object` should be `Box<T>`")
                            })
                        });
                        (qualifier, res)
                    })
                    .map(|(qualifier, res)| res.map(|object| (qualifier, object)))
                    .collect::<Result<Self, InjectorError>>()?;

                if !collection.is_empty() {
                    Ok(collection)
                } else {
                    Err(InjectorError::EmptyCollection {
                        collection: any::type_name::<Self>(),
                        pattern: any::type_name::<P>(),
                    })
                }
            }
        }
    };
}

impl_collect_for_maps!(HashMap, [TypedQualifier]);
impl_collect_for_maps!(BTreeMap, [TypedQualifier, Ord]);

impl<T, P> Collect<P> for HashMap<Box<dyn Qualifier>, T>
where
    T: Managed,
    P: Pattern<Target = T, Qualifier = Box<dyn Qualifier>>,
{
    fn collect<'a, I, KI>(injector: &I, keys: KI, pattern: P) -> Result<Self, InjectorError>
    where
        I: TypedInjector + ?Sized,
        KI: Iterator<Item = &'a dyn Key>,
    {
        let collection = keys.filter(|key| key.target_type() == TypeId::of::<T>())
            .filter(move |key| pattern.matches(*key))
            .map(|key| {
                let qualifier = key.dyn_qualifier().dyn_clone();
                let res = injector.dyn_get(key).map(|object| {
                    *object.downcast::<T>().unwrap_or_else(|_| {
                        unreachable!("in impl `Collect<Box<dyn Qualifier>>` for `HashMap`, `object` should be `Box<T>`")
                    })
                });
                (qualifier, res)
            })
            .map(|(qualifier, res)| res.map(|object| (qualifier, object)))
            .collect::<Result<Self, InjectorError>>()?;

        if !collection.is_empty() {
            Ok(collection)
        } else {
            Err(InjectorError::EmptyCollection {
                collection: any::type_name::<Self>(),
                pattern: any::type_name::<P>(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use std::iter;

    use crate::container::injector::MockInjector;
    use crate::key::{self, AnyPattern, KeyTypePattern};

    use super::*;

    #[test]
    fn vec_collect_succeeds() {
        let injector = make_injector();
        let keys = make_keys();
        let keys = keys.iter().map(AsRef::as_ref);

        let pattern = AnyPattern::new();
        let objects: Vec<i32> = Collect::collect(&injector, keys.clone(), pattern).unwrap();
        assert!(objects.contains(&42i32));
        assert!(objects.contains(&1i32));
        assert!(objects.contains(&2i32));

        let pattern: KeyTypePattern<_, ()> = KeyTypePattern::new();
        let objects: Vec<i32> = Collect::collect(&injector, keys, pattern).unwrap();
        assert!(objects.contains(&42i32));
    }

    #[test]
    fn hash_map_collect_succeeds() {
        let injector = make_injector();
        let keys = make_keys();
        let keys = keys.iter().map(AsRef::as_ref);

        let pattern = AnyPattern::new();
        let objects: HashMap<_, i32> = Collect::collect(&injector, keys.clone(), pattern).unwrap();
        assert_eq!(objects.get(().upcast_dyn()), Some(&42i32));
        assert_eq!(objects.get("1".upcast_dyn()), Some(&1i32));
        assert_eq!(objects.get("2".upcast_dyn()), Some(&2i32));

        let pattern: KeyTypePattern<_, &'static str> = KeyTypePattern::new();
        let objects: HashMap<_, i32> = Collect::collect(&injector, keys.clone(), pattern).unwrap();
        assert_eq!(objects.get("1"), Some(&1i32));
        assert_eq!(objects.get("2"), Some(&2i32));
    }

    #[test]
    fn vec_collect_fails_when_no_matching_key_exists() {
        let injector = make_injector();
        let keys = iter::empty();
        let pattern = AnyPattern::new();
        let res: Result<Vec<f64>, _> = Collect::collect(&injector, keys, pattern);
        assert!(matches!(res, Err(InjectorError::EmptyCollection { .. })));
    }

    fn make_keys() -> Vec<Box<dyn Key>> {
        vec![
            Box::new(key::of::<i32>()),
            Box::new(key::named::<i32>("1")),
            Box::new(key::named::<i32>("2")),
        ]
    }

    fn make_injector() -> impl TypedInjector {
        let mut providers: HashMap<Box<dyn Key>, Box<dyn Fn() -> Box<dyn Managed>>> =
            HashMap::new();
        providers.insert(
            Box::new(key::of::<i32>()),
            Box::new(|| -> Box<dyn Managed> { Box::new(42i32) }),
        );
        providers.insert(
            Box::new(key::named::<i32>("1")),
            Box::new(|| -> Box<dyn Managed> { Box::new(1i32) }),
        );
        providers.insert(
            Box::new(key::named::<i32>("2")),
            Box::new(|| -> Box<dyn Managed> { Box::new(2i32) }),
        );
        providers.insert(
            Box::new(key::of::<&'static str>()),
            Box::new(|| -> Box<dyn Managed> { Box::new("str") }),
        );

        let mut injector = MockInjector::new();
        injector
            .expect_dyn_get()
            .returning_st(move |key| Ok(providers.get(key).unwrap()()));
        injector
    }
}
