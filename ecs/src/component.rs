use std::{
    any::{Any, TypeId},
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use crate::errors::StoreError;

/// Defines a `Component`. Has a predefined memory size and can implement Any
///
/// `Component`s are data structs that can be dynamically attached to `Entity`ies.
pub trait Component: Send + Sync + 'static {
    /// Cast a boxed instance of a `Component` into a downcastable `Box<dyn Any>`
    fn to_any(self: Box<Self>) -> Box<dyn Any>;
    /// Cast a boxed instance of a `Component` into a `ComponentStore` containing itself
    fn to_store(self: Box<Self>) -> ComponentStore;
}

/// Defines a `ComponentBox`. Wraps a `Component, allowing it to be passed as established data`
///
/// Contians its wrapped component within a `Box`.
pub struct ComponentBox {
    component: Box<dyn Component>,
    type_id: TypeId,
}

impl ComponentBox {
    /// Create a new `ComponentBox` from an exposed `Component`
    pub fn new<T: Component>(comp: T) -> Self {
        Self {
            component: Box::new(comp),
            type_id: TypeId::of::<T>(),
        }
    }

    /// Attempts to downcast contained `Component` into the specified type, exposing it if successful
    pub fn cast_inner<T: Component>(self) -> Result<T, StoreError> {
        let inner: Box<T> = self
            .component
            .to_any()
            .downcast::<T>()
            .or(Err(StoreError::CannotCastToType))?;

        Ok(*inner)
    }

    /// Get the `type_id` of the contained `Component`
    pub fn inner_type_id(&self) -> TypeId {
        self.type_id
    }

    /// Consumes the `ComponentBox` to create a `ComponentStore` where the first index is populated by the
    /// inner `Component`
    pub fn create_store(self) -> ComponentStore {
        self.component.to_store()
    }
}

impl<T> From<T> for ComponentBox
where
    T: Component,
{
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

/// Defines a `ComponentVec`. Has implementations for up/downcasting between
/// native type and `Any`
///
/// `ComponentVec`s contain all of the information for `Entities` within a given `Archetype`.
pub trait ComponentVec {
    /// Casts to a downcastable &dyn Any
    fn to_any(&self) -> &dyn Any;
    /// Casts to a mutable downcastable &mut dyn Any
    fn to_any_mut(&mut self) -> &mut dyn Any;
    /// Pushes a given `ComponentBox` into the next available index of the vec, storing it as a `Component`
    fn push(&mut self, comp: ComponentBox) -> Result<(), StoreError>;
    /// Swap-removes a `Component` from the current row, returning it as a `ComponentBox`
    fn swap_remove(&mut self, row: usize) -> ComponentBox;
    /// Migrates the `Component` stored within the target row to the end of the target `ComponentStore`
    fn migrate(&mut self, row: usize, target: &ComponentStore) -> Result<(), StoreError>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
}

impl<T> ComponentVec for Vec<T>
where
    T: Component,
{
    fn to_any(&self) -> &dyn Any {
        self
    }

    fn to_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn push(&mut self, comp: ComponentBox) -> Result<(), StoreError> {
        self.push(comp.cast_inner::<T>()?);
        Ok(())
    }

    fn swap_remove(&mut self, row: usize) -> ComponentBox {
        self.swap_remove(row).into()
    }

    fn migrate(&mut self, row: usize, target: &ComponentStore) -> Result<(), StoreError> {
        let comp: T = self.swap_remove(row);
        target
            .inner_mut()
            .to_any_mut()
            .downcast_mut::<Vec<T>>()
            .ok_or(StoreError::CannotCastToType)?
            .push(comp);

        Ok(())
    }

    fn len(&self) -> usize {
        (*self).len()
    }

    fn is_empty(&self) -> bool {
        (*self).is_empty()
    }
}

/// Defines a `ComponentStore`. Contains and wraps around a `ComponentVec`
pub struct ComponentStore {
    store: Box<RwLock<dyn ComponentVec>>,
    type_id: TypeId,
}

impl ComponentStore {
    /// Fetches a read reference to the inner `ComponentVec`
    pub fn inner(&self) -> ReadGuard {
        self.store.read().unwrap()
    }

    /// Fetches a write reference to the inner `ComponentVec`
    pub fn inner_mut(&self) -> WriteGuard {
        self.store.write().unwrap()
    }

    /// Get the `TypeId` of the contained storage
    pub fn inner_type_id(&self) -> TypeId {
        self.type_id
    }
}

impl<T: Component> From<T> for ComponentStore {
    fn from(value: T) -> Self {
        Self {
            store: Box::new(RwLock::new(Vec::<T>::from([value]))),
            type_id: TypeId::of::<T>(),
        }
    }
}

pub type ReadGuard<'s> = RwLockReadGuard<'s, dyn ComponentVec + 'static>;
pub type WriteGuard<'s> = RwLockWriteGuard<'s, dyn ComponentVec + 'static>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;

    #[test]
    fn test_component_box_cast_inner_fails() {
        let comp: ComponentBox = ComponentBox::new(TestCompA::default());
        let res: Result<TestCompB, StoreError> = comp.cast_inner::<TestCompB>();

        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), StoreError::CannotCastToType));
    }

    #[test]
    fn test_component_box_cast_inner_succeeds() {
        let comp: ComponentBox = ComponentBox::new(TestCompA::default());
        let res: Result<TestCompA, StoreError> = comp.cast_inner::<TestCompA>();

        assert!(res.is_ok());
        assert!(res.unwrap() == TestCompA::default());
    }

    #[test]
    fn test_component_box_create_store() {
        let comp: ComponentBox = ComponentBox::new(TestCompA::default());
        let res: ComponentStore = comp.create_store();

        assert!(
            res.inner()
                .to_any()
                .downcast_ref::<Vec<TestCompA>>()
                .unwrap()[0]
                == TestCompA::default()
        );
    }
}
