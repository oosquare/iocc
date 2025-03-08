use std::any::Any;
use std::hash::{Hash, Hasher};

pub trait DynHash: Any {
    fn dyn_eq(&self, other: &dyn Any) -> bool;

    fn dyn_hash(&self, state: &mut dyn Hasher);
}

impl<T: Eq + Hash + 'static> DynHash for T {
    fn dyn_eq(&self, other: &dyn Any) -> bool {
        other
            .downcast_ref::<T>()
            .map_or(false, |other| self == other)
    }

    fn dyn_hash(&self, mut state: &mut dyn Hasher) {
        self.type_id().hash(&mut state);
        self.hash(&mut state);
    }
}

#[cfg(test)]
mod tests {
    use std::hash::DefaultHasher;

    use super::*;

    #[derive(PartialEq, Eq, Hash)]
    struct A {
        a: i32,
    }

    #[derive(PartialEq, Eq, Hash)]
    struct B {
        b: i32,
    }

    #[test]
    fn dyn_eq_succeeds() {
        let a1 = A { a: 0 };
        let a2 = A { a: 0 };
        let b1 = B { b: 0 };
        let b2 = B { b: 1 };
        assert!(a1.dyn_eq(&a2));
        assert!(!a1.dyn_eq(&b1));
        assert!(!b1.dyn_eq(&b2));
    }

    #[test]
    fn dyn_hash_succeeds() {
        let a1 = A { a: 0 };
        let a2 = A { a: 0 };
        let b1 = B { b: 0 };
        let b2 = B { b: 1 };
        assert_eq!(hash_val(&a1), hash_val(&a2));
        assert_ne!(hash_val(&a1), hash_val(&b1));
        assert_ne!(hash_val(&b1), hash_val(&b2));
    }

    fn hash_val(val: &dyn DynHash) -> u64 {
        let mut hasher = DefaultHasher::new();
        val.dyn_hash(&mut hasher);
        hasher.finish()
    }
}
