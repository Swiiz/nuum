use std::{
    any::{type_name, Any, TypeId},
    borrow::{Borrow, BorrowMut},
};

pub mod store;

pub trait DataStore: BorrowMut<Self> + Borrow<Self> {
    fn insert_dyn(&mut self, type_id: TypeId, value: Box<dyn Any>);
    fn contains_dyn(&self, type_id: TypeId) -> bool;
    fn get_or_insert_with_dyn<F: FnOnce() -> Box<dyn Any>>(
        &mut self,
        type_id: TypeId,
        f: F,
    ) -> &mut dyn Any;
    fn get_or_insert_dyn(&mut self, type_id: TypeId, value: Box<dyn Any>) -> &mut dyn Any;
    fn get_dyn(&self, type_id: TypeId) -> Option<&dyn Any>;
    fn get_mut_dyn(&mut self, type_id: TypeId) -> Option<&mut dyn Any>;
    fn remove_dyn(&mut self, type_id: TypeId) -> Option<Box<dyn Any>>;
}

pub trait Data: DataStore {
    fn insert<T: Any>(&mut self, value: T) {
        DataStore::insert_dyn(self, TypeId::of::<T>(), Box::new(value));
    }

    fn contains<T: Any>(&self) -> bool {
        DataStore::contains_dyn(self, TypeId::of::<T>())
    }

    fn get_or_insert_with<T: Any, F: FnOnce() -> T>(&mut self, f: F) -> &mut T {
        expect_downcast(
            DataStore::get_or_insert_with_dyn(self, TypeId::of::<T>(), || Box::new(f()))
                .downcast_mut(),
        )
    }

    fn get_or_insert<T: Any>(&mut self, value: T) -> &mut T {
        expect_downcast(
            DataStore::get_or_insert_dyn(self, TypeId::of::<T>(), Box::new(value)).downcast_mut(),
        )
    }

    fn get_or_default<T: Any + Default>(&mut self) -> &mut T {
        expect_downcast(
            DataStore::get_or_insert_dyn(self, TypeId::of::<T>(), Box::new(T::default()))
                .downcast_mut(),
        )
    }

    fn get<T: Any>(&self) -> Option<&T> {
        DataStore::get_dyn(self, TypeId::of::<T>()).map(|v| v.downcast_ref().unwrap())
    }

    fn get_mut<T: Any>(&mut self) -> Option<&mut T> {
        DataStore::get_mut_dyn(self, TypeId::of::<T>()).map(|v| v.downcast_mut().unwrap())
    }

    fn remove<T: Any>(&mut self) -> Option<T> {
        DataStore::remove_dyn(self, TypeId::of::<T>()).map(|v| *expect_downcast(v.downcast().ok()))
    }
}

impl<T: DataStore> Data for T {}

fn expect_downcast<T>(opt: Option<T>) -> T {
    opt.expect(
        format!("Failed to downcast value, the specified value ({}) type doesn't match the actual value type", type_name::<T>()).as_str(),
    )
}
