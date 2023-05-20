use std::{ any::{Any, TypeId}, cell::RefCell, borrow::BorrowMut, collections::{hash_map::{IntoIter, Keys}, HashMap}, mem::swap, error::Error, };

use crate::errors::StoreError;


/// Defines a `Component`. Has a predefined memory size and can implement Any
/// 
/// `Component`s are data structs that can be dynamically attached to `Entity`ies.
pub trait Component: Any + 'static {
    fn to_any_box(self) -> Box<dyn Any>;
    fn to_component_store(&self) -> ComponentStore;
}

impl<T> Component for T
    where T: Any + 'static
{
    fn to_any_box(self) -> Box<dyn Any> {
        Box::new(self)
    }

    fn to_component_store(&self) -> ComponentStore {
        self.into()
    }
}


pub struct ComponentBox(Box<dyn Component>);

impl ComponentBox {
    pub fn new<T>(comp: T) -> Self 
        where T: Component
    {
        Self(Box::new(comp))
    }

    pub fn cast_inner<T>(self) -> Result<T, StoreError>
        where T: Component
    {
        if let Ok(inner) = self.to_any_box().downcast::<T>() {
            Ok(*inner)
        }
        else {
            Err(StoreError::MismatchedComponentTypes)
        }
    }

    pub fn create_store(&self) -> ComponentStore {
        self.0.to_component_store()
    }
}


/// Defines a `ComponentBundle`. Stores a single `Component with required type data.`
pub struct ComponentBundle(HashMap<TypeId, ComponentBox>);

impl ComponentBundle {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn push<T>(&mut self, comp: T)
        where T: Component
    {
        self.0.insert(TypeId::of::<T>(), ComponentBox::new(comp));
    }

    pub fn into_iter(self) -> IntoIter<TypeId, ComponentBox> {
        self.0.into_iter()
    }

    pub fn types(&self) -> Keys<TypeId, ComponentBox> {
        self.0.keys()
    }
}


/// Defines a `ComponentCollection`. Has implementations for up/downcasting between
/// native type and `Any`
/// 
/// `ComponentCollection`s contain all of the information for `Entities` within a given `Archetype`.
pub trait ComponentVec {
    fn to_any(&self) -> &dyn Any;
    fn to_any_mut(&mut self) -> &mut dyn Any;

    fn len(&self) -> usize;
    fn push(&mut self, comp: ComponentBox) -> Result<(), StoreError>;
    fn swap_remove(&mut self, row: usize);
}

impl<T> ComponentVec for RefCell<Vec<T>>
    where T: Component
{
    /// Upcast from self into `Any`
    fn to_any(&self) -> &dyn Any {
        self
    }

    /// Same as `as_any`, except mutable
    fn to_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    /// Get length for internal storage
    fn len(&self) -> usize {
        self.borrow().len()
    }

    fn push(&mut self, comp: ComponentBox) -> Result<(), StoreError> {
        self.get_mut().push(comp.cast_inner::<T>()?);
        Ok(())
    }

    fn swap_remove(&mut self, row: usize) {
        self.borrow_mut().swap_remove(row);
    }
}


/// Defines a `ComponentStore`. Contains a `ComponentCollection` and information about its TypeId
pub struct ComponentStore {
    type_id: TypeId,
    store: Box<dyn ComponentVec>,
}

impl ComponentStore {
    /// Constructs a new `ComponentStore` of a type matching the initial value added
    pub fn new<T>() -> Self
        where T: Component
    {
        Self{
            type_id: TypeId::of::<T>(),
            store: Box::new(RefCell::new(Vec::<T>::new()))
        }
    }

    pub fn push(&mut self, comp: ComponentBox) -> Result<(), StoreError> {  
        self.store.push(comp)
    }

    pub fn len(&self) -> usize {
        self.store.len()
    }
}

impl<T> From<&T> for ComponentStore
    where T: Component
{
    fn from(_: &T) -> Self {
        Self::new::<T>()
    }
}