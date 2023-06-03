use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::{BTreeSet, HashMap},
    vec::IntoIter,
};

use crate::errors::StoreError;

/// Defines the type identifier for an `Archetype`. all immutable instances are sorted
///
/// Uses a `BTreeSet` to remain hashable
#[derive(Default, Clone, PartialEq, Eq, Hash)]
pub struct TypeBundle(BTreeSet<TypeId>);

impl TypeBundle {
    /// Create a new `TypeBundle` by adding the provided `type_id`
    pub fn add_type(&self, type_id: TypeId) -> Self {
        let mut new: BTreeSet<TypeId> = self.0.clone();
        new.insert(type_id);
        Self(new)
    }

    /// Create a new `TypeBundle` by removing the provided `type_id`
    pub fn remove_type(&self, type_id: TypeId) -> Self {
        let mut new: BTreeSet<TypeId> = self.0.clone();
        new.remove(&type_id);
        Self(new)
    }
}

impl From<&HashMap<TypeId, usize>> for TypeBundle {
    /// Generate a `TypeBundle` from an existing `HashMap`
    fn from(types: &HashMap<TypeId, usize>) -> Self {
        Self(types.keys().cloned().collect())
    }
}

/// Defines a `Component`. Has a predefined memory size and can implement Any
///
/// `Component`s are data structs that can be dynamically attached to `Entity`ies.
pub trait Component: Any + 'static {
    /// Cast a boxed instance of a `Component` into a downcastable `Box<dyn Any>`
    fn to_any(self: Box<Self>) -> Box<dyn Any>;
    /// Cast a boxed instance of a `Component` into a `ComponentStore` containing itself
    fn to_store(self: Box<Self>) -> ComponentStore;
}

/// Defines a `ComponentBox`. Wraps a `Component, allowing it to be passed as established data`
///
/// Contians its wrapped component within a `Box`.
pub struct ComponentBox(Box<dyn Component>);

impl ComponentBox {
    /// Create a new `ComponentBox` from an exposed `Component`
    pub fn new<T: Component>(comp: T) -> Self {
        Self(Box::new(comp))
    }

    /// Attempts to downcast contained `Component` into the specified type, exposing it if successful
    pub fn cast_inner<T: Component>(self) -> Result<T, StoreError> {
        let inner: Box<T> = self
            .0
            .to_any()
            .downcast::<T>()
            .or(Err(StoreError::CannotCastToType))?;

        Ok(*inner)
    }

    /// Get the `type_id` of the contained `Component`
    pub fn type_id(&self) -> TypeId {
        (*self.0).type_id()
    }

    /// Consumes the `ComponentBox` to create a `ComponentStore` where the first index is populated by the
    /// inner `Component`
    pub fn create_store(self) -> ComponentStore {
        self.0.to_store()
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

/// Defines a `ComponentBundle`. Stores a collection of unique `Components` associated with the same `Entity`
///
/// Uses a `HashMap` for type associations, storing references to related `Components`
#[derive(Default)]
pub struct ComponentBundle {
    index: HashMap<TypeId, usize>,
    components: Vec<ComponentBox>,
}

impl ComponentBundle {
    /// Add a `ComponentBox` to the bundle
    pub fn insert(&mut self, comp: ComponentBox) {
        self.index.insert(comp.type_id(), self.index.len());
        self.components.push(comp);
    }

    /// Remove a `ComponentBox` from the bundle matching the given type_id
    pub fn remove(&mut self, type_id: TypeId) -> Result<ComponentBox, StoreError> {
        let moved: TypeId = self
            .components
            .last()
            .expect("expected bundle to contain a value")
            .type_id();
        let idx: usize = self
            .index
            .remove(&type_id)
            .ok_or(StoreError::TypeNotInBundle)?;

        self.index.insert(moved, idx);
        let moved: ComponentBox = self.components.swap_remove(idx);

        Ok(moved)
    }

    /// Gets the associated `TypeBundle` for the bundle
    pub fn types(&self) -> TypeBundle {
        (&self.index).into()
    }

    /// Consumes the bundle to provide an `Iterator` over every contained `ComponentBox`
    pub fn component_iter(self) -> IntoIter<ComponentBox> {
        self.components.into_iter()
    }
}

/// Defines a `ComponentVec`. Has implementations for up/downcasting between
/// native type and `Any`
///
/// `ComponentCollection`s contain all of the information for `Entities` within a given `Archetype`.
pub trait ComponentVec {
    /// Casts to a downcastable &dyn Any
    fn to_any(&self) -> &dyn Any;
    /// Pushes a given `ComponentBox` into the next available index of the vec, storing it as a `Component`
    fn push(&self, comp: ComponentBox) -> Result<(), StoreError>;
    /// Swap-removes a `Component` from the current row, returning it as a `ComponentBox`
    fn swap_remove(&self, row: usize) -> ComponentBox;
    /// Migrates the `Component` stored within the target row to the end of the target `ComponentStore`
    fn migrate(&self, row: usize, target: &ComponentStore) -> Result<(), StoreError>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
}

impl<T> ComponentVec for RefCell<Vec<T>>
where
    T: Component,
{
    fn to_any(&self) -> &dyn Any {
        self
    }

    fn push(&self, comp: ComponentBox) -> Result<(), StoreError> {
        self.borrow_mut().push(comp.cast_inner::<T>()?);
        Ok(())
    }

    fn swap_remove(&self, row: usize) -> ComponentBox {
        ComponentBox::new(self.borrow_mut().swap_remove(row))
    }

    fn migrate(&self, row: usize, target: &ComponentStore) -> Result<(), StoreError> {
        let comp: T = self.borrow_mut().swap_remove(row);
        target
            .0
            .to_any()
            .downcast_ref::<RefCell<Vec<T>>>()
            .ok_or(StoreError::CannotCastToType)?
            .borrow_mut()
            .push(comp);

        Ok(())
    }

    fn len(&self) -> usize {
        self.borrow().len()
    }

    fn is_empty(&self) -> bool {
        self.borrow().is_empty()
    }
}

/// Defines a `ComponentStore`. Contains and wraps around a `ComponentVec`
pub struct ComponentStore(Box<dyn ComponentVec>);

impl ComponentStore {
    /// Constructs a new `ComponentStore` of a type matching the initial value added
    pub fn from<T: Component>(comp: T) -> Self {
        Self(Box::new(RefCell::new(Vec::<T>::from([comp]))))
    }

    /// Pushes a `ComponentBox` into the contianed `ComponentVec`
    pub fn push(&self, comp: ComponentBox) -> Result<(), StoreError> {
        self.0.push(comp)
    }

    /// Swap-removes a `Component` from the contained `ComponentVec`
    pub fn swap_remove(&self, row: usize) -> ComponentBox {
        self.0.swap_remove(row)
    }

    /// Migrates a `Component` from the contained `ComponentVec` and into the target
    pub fn migrate(&self, row: usize, target: &Self) -> Result<(), StoreError> {
        self.0.migrate(row, target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecs_derive::Component;

    #[derive(Component, Default, PartialEq, Debug)]
    struct TestCompOne {
        _one: u32,
        _two: String,
    }

    #[derive(Component, Default, PartialEq, Debug)]
    struct TestCompTwo {
        _three: u32,
        _four: String,
    }

    #[test]
    fn test_component_box_cast_inner_fails() {
        let comp: ComponentBox = ComponentBox::new(TestCompOne::default());
        let res: Result<TestCompTwo, StoreError> = comp.cast_inner::<TestCompTwo>();

        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), StoreError::CannotCastToType));
    }

    #[test]
    fn test_component_box_cast_inner_succeeds() {
        let comp: ComponentBox = ComponentBox::new(TestCompOne::default());
        let res: Result<TestCompOne, StoreError> = comp.cast_inner::<TestCompOne>();

        assert!(res.is_ok());
        assert!(res.unwrap() == TestCompOne::default());
    }

    #[test]
    fn test_component_box_create_store() {
        let comp: ComponentBox = ComponentBox::new(TestCompOne::default());
        let res: ComponentStore = comp.create_store();

        assert!(
            res.0
                .to_any()
                .downcast_ref::<RefCell<Vec<TestCompOne>>>()
                .unwrap()
                .borrow()[0]
                == TestCompOne::default()
        );
    }

    #[test]
    fn test_component_bundle_push() {
        let mut bundle: ComponentBundle = ComponentBundle::default();

        bundle.insert(ComponentBox::new(TestCompOne::default()));

        assert!(bundle.types().0 == BTreeSet::from([TypeId::of::<TestCompOne>()]));
        assert!(
            bundle
                .components
                .pop()
                .unwrap()
                .cast_inner::<TestCompOne>()
                .unwrap()
                == TestCompOne::default()
        );
    }

    #[test]
    fn test_component_bundle_remove() {
        let mut bundle: ComponentBundle = ComponentBundle {
            index: HashMap::new(),
            components: Vec::new(),
        };

        bundle.insert(TestCompOne::default().into());
        bundle.insert(TestCompTwo::default().into());

        let res: Result<ComponentBox, StoreError> = bundle.remove(TypeId::of::<TestCompOne>());

        assert!(res.is_ok());
        assert!(res.unwrap().cast_inner::<TestCompOne>().unwrap() == TestCompOne::default());
        assert!(bundle.index.len() == 1);

        let bad_res: Result<ComponentBox, StoreError> = bundle.remove(TypeId::of::<TestCompOne>());
        assert!(bad_res.is_err());
        assert!(matches!(
            bad_res.err().unwrap(),
            StoreError::TypeNotInBundle
        ));

        assert!(
            bundle
                .components
                .pop()
                .unwrap()
                .cast_inner::<TestCompTwo>()
                .unwrap()
                == TestCompTwo::default()
        );
    }

    #[test]
    fn test_component_store_migration() {
        let one: ComponentStore = Box::<TestCompOne>::default().to_store();
        let two: ComponentStore = Box::<TestCompOne>::default().to_store();

        assert!(one.migrate(0, &two).is_ok());
        assert!(one.0.is_empty());
        assert!(two.0.len() == 2);
    }
}
