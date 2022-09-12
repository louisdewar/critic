use std::{
    any::{Any, TypeId},
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use parking_lot::{RwLockReadGuard, RwLockWriteGuard};

/// Represents a dynamic group of dependencies for a test or fixture
#[derive(Default)]
pub struct Dependencies<'dep> {
    owned: HashMap<TypeId, Box<dyn Any>>,
    shared: HashMap<TypeId, RwLockReadGuard<'dep, Box<dyn Any>>>,
    exclusive: HashMap<TypeId, RwLockWriteGuard<'dep, Box<dyn Any>>>,
}

pub struct ReadGuard<'a> {
    inner: RwLockReadGuard<'a, Box<dyn Any>>,
}

impl<'a> ReadGuard<'a> {
    pub fn guard_extract<T: 'static>(&self) -> &T {
        self.inner
            .deref()
            .downcast_ref()
            .expect("failed to downcast into inner type")
    }
}

pub struct WriteGuard<'a> {
    inner: RwLockWriteGuard<'a, Box<dyn Any>>,
}

impl<'a> WriteGuard<'a> {
    pub fn guard_extract<T: 'static>(&mut self) -> &mut T {
        self.inner
            .deref_mut()
            .downcast_mut()
            .expect("failed to downcast into inner type")
    }
}

/// Useful for codegen even though it isn't really a "guard"
pub struct OwnedGuard {
    inner: Box<dyn Any>,
}

impl OwnedGuard {
    pub fn guard_extract<T: 'static>(self) -> T {
        *self
            .inner
            .downcast()
            .expect("failed to downcast owned type")
    }
}

impl<'dep> Dependencies<'dep> {
    pub fn new() -> Self {
        Default::default()
    }

    pub(crate) fn add_owned<T: 'static>(&mut self, val: T) {
        self.owned
            .insert(TypeId::of::<T>(), Box::new(val) as Box<dyn Any>);
    }

    pub(crate) fn add_shared(&mut self, type_id: TypeId, val: RwLockReadGuard<'dep, Box<dyn Any>>) {
        self.shared.insert(type_id, val);
    }

    pub(crate) fn add_exclusive(
        &mut self,
        type_id: TypeId,
        val: RwLockWriteGuard<'dep, Box<dyn Any>>,
    ) {
        self.exclusive.insert(type_id, val);
    }

    pub fn owned(&mut self, type_id: TypeId) -> OwnedGuard {
        self.owned
            .remove(&type_id)
            .map(|val| OwnedGuard { inner: val })
            .expect("the owned type should exist")
    }

    pub fn shared(&mut self, type_id: TypeId) -> ReadGuard<'dep> {
        self.shared
            .remove(&type_id)
            .map(|val| ReadGuard { inner: val })
            .expect("the shared type should exist")
    }

    pub fn exclusive(&mut self, type_id: TypeId) -> WriteGuard<'dep> {
        self.exclusive
            .remove(&type_id)
            .map(|val| WriteGuard { inner: val })
            .expect("the exclusive type should exist")
    }
}
// impl<'dep> Dependencies<'dep> {
//     pub fn add<T: 'dep>(&mut self, val: T) {
//         self.items
//             .insert(TypeId::of::<T>(), Box::new(val) as Box<dyn Any>);
//     }
//
//     pub fn owned<T: 'dep>(&mut self) -> Option<T> {
//         self.items
//             .remove(&TypeId::of::<T>())
//             .map(|val| *val.downcast::<T>().expect("type ID should match type"))
//     }
//
//     // // fn rw_lock<T: 'static>(&mut self) -> Option<
//     //
//     // pub fn shared<T: 'static>(&mut self) -> Option<&T> {
//     //     todo!();
//     //     // self.items
//     //     //     .remove(&TypeId::of::<T>())
//     //     //     .map(|val| *val.downcast_::<T>().expect("type ID should match type"))
//     // }
//     //
//     // pub fn exclusive<'a, T: 'static>(&'a mut self) -> Option<impl DerefMut<Target = T> + 'a> {
//     //     self.items
//     //         .remove(&TypeId::of::<T>())
//     //         .map(|val| *val.downcast::<T>().expect("type ID should match type"))
//     // }
// }
