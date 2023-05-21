use std::{ any::{Any, TypeId}, cell::RefCell, collections::{HashMap, BTreeSet}};

use crate::errors::StoreError;


/// Defines the type identifier for an `Archetype`. all immutable instances are sorted
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Types(BTreeSet<TypeId>);

impl Default for Types {
    fn default() -> Self {
        Self(BTreeSet::new())
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
    where T: Any + 'static
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
        if let Ok(inner) = self.to_any_box().downcast::<T>() {
            Ok(*inner)
        }
        else {
            Err(StoreError::CannotCastToType)
        }
    }

    pub fn type_id(&self) -> TypeId {
        self.0.type_id()
    }

    pub fn create_store(self) -> ComponentStore {
        self.0.to_component_store()
    }
}


/// Defines a `ComponentBundle`. Stores a single `Component with required type data.`
pub struct ComponentBundle {
    types: HashMap<TypeId, usize>,
    components: Vec<ComponentBox>
}

impl ComponentBundle {
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
            components: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.types.len()
    }

    pub fn push<T>(&mut self, comp: T)
        where T: Component
    {
        self.types.insert(TypeId::of::<T>(), self.types.len());
        self.components.push(ComponentBox::new(comp));
    }

    pub fn components(self) -> Vec<ComponentBox> {
        self.components
    }

    pub fn types(&self) -> Types {
        Types(self.types.keys()
            .cloned()
            .collect())
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
        self.get_mut().swap_remove(row);
    }
}


/// Defines a `ComponentStore`. Contains a `ComponentCollection` and information about its TypeId
pub struct ComponentStore (Box<dyn ComponentVec>);

impl ComponentStore {
    /// Constructs a new `ComponentStore` of a type matching the initial value added
    pub fn new<T: Component>(comp: T) -> Self {
        Self(Box::new(RefCell::new(Vec::<T>::from([comp]))))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn push(&mut self, comp: ComponentBox) -> Result<(), StoreError> {  
        self.0.push(comp)
    }

    pub fn swap_remove(&mut self, row: usize) {
        self.0.swap_remove(row)
    }
}