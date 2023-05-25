use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::{BTreeMap, BTreeSet, HashMap},
    vec::IntoIter,
};

use crate::errors::StoreError;

/// Defines the type identifier for an `Archetype`. all immutable instances are sorted
#[derive(Default, Clone, PartialEq, Eq, Hash)]
pub struct TypeBundle(BTreeSet<TypeId>);

impl TypeBundle {
    pub fn add_type(&self, type_id: TypeId) -> Self {
        let mut new = self.0.clone();
        new.insert(type_id);
        Self(new)
    }

    pub fn remove_type(&self, type_id: TypeId) -> Self {
        let mut new = self.0.clone();
        new.remove(&type_id);
        Self(new)
    }
}

impl From<&HashMap<TypeId, usize>> for TypeBundle {
    fn from(types: &HashMap<TypeId, usize>) -> Self {
        Self(types.keys().cloned().collect())
    }
}

impl From<&BTreeMap<TypeId, usize>> for TypeBundle {
    fn from(types: &BTreeMap<TypeId, usize>) -> Self {
        Self(types.keys().cloned().collect())
    }
}

/// Defines a `Component`. Has a predefined memory size and can implement Any
///
/// `Component`s are data structs that can be dynamically attached to `Entity`ies.
pub trait Component: Any + 'static {
    fn to_any_box(self) -> Box<dyn Any>;
    fn to_component_store(self) -> ComponentStore;
}

impl<T> Component for T
where
    T: Any + 'static,
{
    fn to_any_box(self) -> Box<dyn Any> {
        Box::new(self)
    }

    fn to_component_store(self) -> ComponentStore {
        ComponentStore::new(self)
    }
}

pub struct ComponentBox(Box<dyn Component>);

impl ComponentBox {
    pub fn new<T: Component>(comp: T) -> Self {
        Self(Box::new(comp))
    }

    pub fn cast_inner<T: Component>(self) -> Result<T, StoreError> {
        let inner = self
            .0
            .to_any_box()
            .downcast::<T>()
            .or(Err(StoreError::CannotCastToType))?;

        Ok(*inner)
    }

    pub fn type_id(&self) -> TypeId {
        self.0.type_id()
    }

    pub fn create_store(self) -> ComponentStore {
        self.0.to_component_store()
    }
}

/// Defines a `ComponentBundle`. Stores a single `Component with required type data.`
#[derive(Default)]
pub struct ComponentBundle {
    index: BTreeMap<TypeId, usize>,
    components: Vec<ComponentBox>,
}

impl ComponentBundle {
    pub fn push<T>(&mut self, comp: T)
    where
        T: Component,
    {
        let comp = ComponentBox::new(comp);
        self.index.insert(comp.type_id(), self.index.len());
        self.components.push(ComponentBox::new(comp));
    }

    pub fn remove(&mut self, type_id: TypeId) -> Result<(), StoreError> {
        let moved = self
            .components
            .last()
            .expect("expected bundle to contain a value")
            .type_id();
        let idx = self
            .index
            .remove(&type_id)
            .ok_or(StoreError::TypeNotInBundle)?;
        self.components.swap_remove(idx);
        self.index.insert(moved, idx);

        Ok(())
    }

    pub fn types(&self) -> TypeBundle {
        (&self.index).into()
    }

    pub fn component_iter(self) -> IntoIter<ComponentBox> {
        self.components.into_iter()
    }
}

/// Defines a `ComponentCollection`. Has implementations for up/downcasting between
/// native type and `Any`
///
/// `ComponentCollection`s contain all of the information for `Entities` within a given `Archetype`.
pub trait ComponentVec {
    fn to_any(&self) -> &dyn Any;
    fn push(&mut self, comp: ComponentBox) -> Result<(), StoreError>;
    fn swap_remove(&mut self, row: usize);
    fn swap_remove_to_box(&mut self, row: usize) -> ComponentBox;
    fn migrate(&mut self, row: usize, target: &ComponentStore) -> Result<(), StoreError>;
}

impl<T> ComponentVec for RefCell<Vec<T>>
where
    T: Component,
{
    fn to_any(&self) -> &dyn Any {
        self
    }

    fn push(&mut self, comp: ComponentBox) -> Result<(), StoreError> {
        self.get_mut().push(comp.cast_inner::<T>()?);
        Ok(())
    }

    fn swap_remove(&mut self, row: usize) {
        self.get_mut().swap_remove(row);
    }

    fn swap_remove_to_box(&mut self, row: usize) -> ComponentBox {
        ComponentBox::new(self.get_mut().swap_remove(row))
    }

    fn migrate(&mut self, row: usize, target: &ComponentStore) -> Result<(), StoreError> {
        let comp: T = self.get_mut().swap_remove(row);
        target
            .0
            .to_any()
            .downcast_ref::<RefCell<Vec<T>>>()
            .ok_or(StoreError::CannotCastToType)?
            .borrow_mut()
            .push(comp);

        Ok(())
    }
}

/// Defines a `ComponentStore`. Contains a `ComponentCollection` and information about its TypeId
pub struct ComponentStore(Box<dyn ComponentVec>);

impl ComponentStore {
    /// Constructs a new `ComponentStore` of a type matching the initial value added
    pub fn new<T: Component>(comp: T) -> Self {
        Self(Box::new(RefCell::new(Vec::<T>::from([comp]))))
    }

    pub fn push(&mut self, comp: ComponentBox) -> Result<(), StoreError> {
        self.0.push(comp)
    }

    pub fn swap_remove(&mut self, row: usize) {
        self.0.swap_remove(row)
    }

    pub fn swap_remove_to_box(&mut self, row: usize) -> ComponentBox {
        self.0.swap_remove_to_box(row)
    }

    pub fn migrate(&mut self, row: usize, target: &Self) -> Result<(), StoreError> {
        self.0.migrate(row, target)
    }
}
