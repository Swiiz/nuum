use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use super::DataStore;

pub type TypeMap = HashMap<TypeId, Box<dyn Any>>;

impl DataStore for TypeMap {
    fn insert_dyn(&mut self, type_id: TypeId, value: Box<dyn Any>) {
        self.insert(type_id, value);
    }

    fn contains_dyn(&self, type_id: TypeId) -> bool {
        self.contains_key(&type_id)
    }

    fn get_or_insert_with_dyn<F: FnOnce() -> Box<dyn Any>>(
        &mut self,
        type_id: TypeId,
        f: F,
    ) -> &mut dyn Any {
        self.entry(type_id).or_insert_with(f)
    }

    fn get_or_insert_dyn(&mut self, type_id: TypeId, value: Box<dyn Any>) -> &mut dyn Any {
        self.entry(type_id).or_insert(value)
    }

    fn get_dyn(&self, type_id: TypeId) -> Option<&dyn Any> {
        self.get(&type_id).map(|v| v.as_ref())
    }

    fn get_mut_dyn(&mut self, type_id: TypeId) -> Option<&mut dyn Any> {
        self.get_mut(&type_id).map(|v| v.as_mut())
    }

    fn remove_dyn(&mut self, type_id: TypeId) -> Option<Box<dyn Any>> {
        self.remove(&type_id)
    }
}
