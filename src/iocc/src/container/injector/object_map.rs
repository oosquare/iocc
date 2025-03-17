use std::any::TypeId;
use std::collections::HashMap;
use std::mem;

use crate::container::{Managed, SharedManaged};
use crate::key::Key;

pub struct ObjectMap {
    objects: HashMap<TypeId, Slot>,
}

impl ObjectMap {
    pub fn new() -> Self {
        Self {
            objects: HashMap::new(),
        }
    }

    pub fn insert(
        &mut self,
        key: Box<dyn Key>,
        object: Box<dyn SharedManaged>,
    ) -> Option<ObjectEntry> {
        let target = key.target();
        if let Some(slot) = self.objects.get_mut(&target) {
            slot.insert(key, ObjectEntry(object))
        } else {
            self.objects
                .insert(target, Slot::new(key, ObjectEntry(object)));
            None
        }
    }

    pub fn get(&self, key: &dyn Key) -> Option<&ObjectEntry> {
        self.objects
            .get(&key.target())
            .and_then(|slot| slot.get(key))
    }
}

enum Slot {
    Singleton(Box<dyn Key>, ObjectEntry),
    Map(HashMap<Box<dyn Key>, ObjectEntry>),
}

impl Slot {
    fn new(key: Box<dyn Key>, entry: ObjectEntry) -> Self {
        Self::Singleton(key, entry)
    }

    fn insert(&mut self, key: Box<dyn Key>, entry: ObjectEntry) -> Option<ObjectEntry> {
        match self {
            Self::Singleton(k, e) if k == &key => {
                let original = mem::replace(e, entry);
                Some(original)
            }
            Self::Singleton(_, _) => {
                let Self::Singleton(k, e) =
                    mem::replace(self, Self::Map(HashMap::with_capacity(2)))
                else {
                    unreachable!("`self` should match `Self::Singleton(_, _)`")
                };
                let Self::Map(entries) = self else {
                    unreachable!("`self` should alredy be assigned to `Self::Map(_)`")
                };
                entries.insert(k, e);
                entries.insert(key, entry);
                None
            }
            Self::Map(entries) => entries.insert(key, entry),
        }
    }

    fn get(&self, key: &dyn Key) -> Option<&ObjectEntry> {
        match self {
            Self::Singleton(k, _) if k.as_ref() != key => None,
            Self::Singleton(_, e) => Some(e),
            Self::Map(entries) => entries.get(key),
        }
    }
}

pub struct ObjectEntry(Box<dyn SharedManaged>);

impl ObjectEntry {
    pub fn clone_managed(&self) -> Box<dyn Managed> {
        self.0.dyn_clone().upcast_managed()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::key;
    use crate::util::any::DowncastRef;

    use super::*;

    #[test]
    fn object_map_insert_succeeds() {
        let mut map = ObjectMap::new();

        assert!(map
            .insert(Box::new(key::of::<Arc<i32>>()), Box::new(Arc::new(42i32)))
            .is_none());

        let obj = &map.get(&key::of::<Arc<i32>>()).unwrap().0;
        assert_eq!(**obj.downcast_ref::<Arc<i32>>().unwrap(), 42);
    }
}
