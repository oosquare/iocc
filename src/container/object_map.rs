#![allow(dead_code)]
use std::any::TypeId;
use std::collections::HashMap;
use std::mem;

use crate::container::SharedManaged;
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

    pub fn insert_intermediate(&mut self, key: Box<dyn Key>) -> Option<ObjectEntry> {
        self.insert_impl(key, ObjectEntry::Intermediate)
    }

    pub fn insert_constructed(
        &mut self,
        key: Box<dyn Key>,
        object: Box<dyn SharedManaged>,
    ) -> Option<ObjectEntry> {
        self.insert_impl(key, ObjectEntry::Constructed(object))
    }

    fn insert_impl(&mut self, key: Box<dyn Key>, entry: ObjectEntry) -> Option<ObjectEntry> {
        let target = key.target();
        if let Some(slot) = self.objects.get_mut(&target) {
            slot.insert(key, entry)
        } else {
            self.objects.insert(target, Slot::new(key, entry));
            None
        }
    }

    pub fn remove(&mut self, key: &dyn Key) -> Option<ObjectEntry> {
        self.objects
            .get_mut(&key.target())
            .and_then(|slot| slot.remove(key))
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

    fn remove(&mut self, key: &dyn Key) -> Option<ObjectEntry> {
        match self {
            Self::Singleton(k, _) if k.as_ref() != key => None,
            Self::Singleton(_, _) => {
                let Self::Singleton(_, e) = mem::replace(self, Self::Map(HashMap::new())) else {
                    unreachable!("`self` should match `Self::Singleton(_, _)`")
                };
                Some(e)
            }
            Self::Map(entries) => entries.remove(key),
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

pub enum ObjectEntry {
    Intermediate,
    Constructed(Box<dyn SharedManaged>),
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
            .insert_intermediate(Box::new(key::of::<i32>()))
            .is_none());

        assert!(map
            .insert_constructed(Box::new(key::of::<i32>()), Box::new(Arc::new(42i32)))
            .is_some());

        match map.get(&key::of::<i32>()).unwrap() {
            ObjectEntry::Constructed(obj) => {
                assert_eq!(**obj.downcast_ref::<Arc<i32>>().unwrap(), 42)
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn object_map_remove_succeeds() {
        let mut map = ObjectMap::new();

        assert!(map
            .insert_intermediate(Box::new(key::of::<i32>()))
            .is_none());

        assert!(map.remove(&key::of::<i32>()).is_some());
        assert!(map.remove(&key::of::<i32>()).is_none());
    }
}
