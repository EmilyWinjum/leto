use std::{
    any::TypeId,
    collections::{btree_set::Iter, BTreeSet, HashMap},
    vec::IntoIter,
};

use crate::{
    component::{Component, ComponentBox},
    errors::StoreError,
};

/// Defines the type identifier for an `Archetype`. all immutable instances are sorted
///
/// Uses a `BTreeSet` to remain hashable
#[derive(Default, Clone, PartialEq, Eq, Hash, Debug)]
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

    pub fn contains(&self, bundle: &Self) -> bool {
        self.0.is_superset(&bundle.0)
    }

    pub fn iter(&self) -> Iter<TypeId> {
        self.0.iter()
    }
}

impl From<&HashMap<TypeId, usize>> for TypeBundle {
    /// Generate a `TypeBundle` from an existing `HashMap`
    fn from(types: &HashMap<TypeId, usize>) -> Self {
        Self(types.keys().cloned().collect())
    }
}

impl From<&[TypeId]> for TypeBundle {
    fn from(types: &[TypeId]) -> Self {
        Self(types.iter().cloned().collect())
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
    /// Add a raw `Component` to the bundle
    pub fn insert<T: Component>(mut self, comp: T) -> Self {
        self.index.insert(TypeId::of::<T>(), self.index.len());
        self.components.push(comp.into());
        self
    }

    /// Add a `ComponentBox` to the bundle as a non-consuming reference
    pub fn insert_box(&mut self, comp: ComponentBox) {
        self.index.insert(comp.inner_type_id(), self.index.len());
        self.components.push(comp);
    }

    /// Remove a `ComponentBox` from the bundle matching the given type_id
    pub fn remove(&mut self, type_id: TypeId) -> Result<ComponentBox, StoreError> {
        let moved: TypeId = self
            .components
            .last()
            .expect("expected bundle to contain a value")
            .inner_type_id();
        let idx: usize = self
            .index
            .remove(&type_id)
            .ok_or(StoreError::TypeNotFound)?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;

    #[test]
    fn test_component_bundle_push() {
        let mut bundle: ComponentBundle = ComponentBundle::default().insert(TestCompA::default());

        assert!(bundle.types().0 == BTreeSet::from([TypeId::of::<TestCompA>()]));
        assert!(
            bundle
                .components
                .pop()
                .unwrap()
                .cast_inner::<TestCompA>()
                .unwrap()
                == TestCompA::default()
        );
    }

    #[test]
    fn test_component_bundle_remove() {
        let mut bundle: ComponentBundle = ComponentBundle::default()
            .insert(TestCompA::default())
            .insert(TestCompB::default());

        let res: Result<ComponentBox, StoreError> = bundle.remove(TypeId::of::<TestCompA>());

        assert!(res.is_ok());
        assert!(res.unwrap().cast_inner::<TestCompA>().unwrap() == TestCompA::default());
        assert!(bundle.index.len() == 1);

        let bad_res: Result<ComponentBox, StoreError> = bundle.remove(TypeId::of::<TestCompA>());
        assert!(bad_res.is_err());
        assert!(matches!(bad_res.err().unwrap(), StoreError::TypeNotFound));

        assert!(
            bundle
                .components
                .pop()
                .unwrap()
                .cast_inner::<TestCompB>()
                .unwrap()
                == TestCompB::default()
        );
    }
}
