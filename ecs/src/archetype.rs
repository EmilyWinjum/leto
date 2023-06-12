use std::{
    any::TypeId,
    collections::HashMap,
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use crate::{
    bundle::{ComponentBundle, TypeBundle},
    component::{ComponentBox, ComponentStore},
    entity::EntityId,
    errors::StoreError,
};

pub struct Archetype {
    index: HashMap<TypeId, usize>,
    storage: Box<[ComponentStore]>,
    entities: RwLock<Vec<EntityId>>,
    pub edges: HashMap<TypeId, usize>,
}

impl Archetype {
    fn get_last_entity(&self) -> EntityId {
        self.entities()[self.entities().len()]
    }

    pub fn entities(&self) -> RwLockReadGuard<Vec<EntityId>> {
        self.entities.read().unwrap()
    }

    fn entities_mut(&self) -> RwLockWriteGuard<Vec<EntityId>> {
        self.entities.write().unwrap()
    }

    pub fn get_storage(&self, type_id: TypeId) -> Result<&ComponentStore, StoreError> {
        self.index
            .get(&type_id)
            .map(|&idx| &self.storage[idx])
            .ok_or(StoreError::StorageNotFound)
    }

    pub fn get_entity(&self, row: usize) -> Option<EntityId> {
        self.entities().get(row).copied()
    }

    pub fn new(bundle: ComponentBundle, entity_id: EntityId) -> Self {
        let mut index: HashMap<TypeId, usize> = HashMap::new();
        let mut storage: Vec<ComponentStore> = Vec::new();
        bundle.component_iter().enumerate().for_each(|(idx, comp)| {
            index.insert(comp.inner_type_id(), idx);
            storage.push(comp.create_store());
        });

        Self {
            index,
            storage: storage.into(),
            entities: RwLock::new(Vec::from([entity_id])),
            edges: HashMap::new(),
        }
    }

    pub fn types(&self) -> TypeBundle {
        (&self.index).into()
    }

    pub fn has_type(&self, type_id: TypeId) -> bool {
        self.index.get(&type_id).is_some()
    }

    pub fn add(&self, bundle: ComponentBundle, entity_id: EntityId) -> usize {
        let row = self.entities().len();
        for comp in bundle.component_iter() {
            self.get_storage(comp.inner_type_id())
                .unwrap()
                .inner_mut()
                .push(comp)
                .unwrap();
        }
        self.entities_mut().push(entity_id);

        row
    }

    pub fn remove(&self, row: usize) -> EntityId {
        let entity: EntityId = self.get_last_entity();
        for idx in self.index.values() {
            self.storage[*idx].inner_mut().swap_remove(row);
        }
        self.entities_mut().swap_remove(row);
        entity
    }

    pub fn migrate(&self, target: &mut Self, row: usize, op: Migration) -> (EntityId, usize) {
        let moved: EntityId = self.get_last_entity();
        let target_row = target.entities().len();
        let current = self.entities_mut().swap_remove(row);
        target.entities_mut().push(current);
        match op {
            Migration::Add(comp) => {
                for (&type_id, &idx) in self.index.iter() {
                    let source_store: &ComponentStore = &self.storage[idx];
                    let target_store: &ComponentStore = target.get_storage(type_id).unwrap();
                    source_store.inner_mut().migrate(row, target_store).unwrap();
                }
                target
                    .get_storage(comp.inner_type_id())
                    .unwrap()
                    .inner_mut()
                    .push(comp)
                    .unwrap();
            }
            Migration::Remove(type_id) => {
                for (&type_id, &idx) in target.index.iter() {
                    let source_store: &ComponentStore = self.get_storage(type_id).unwrap();
                    let target_store: &ComponentStore = &mut target.storage[idx];
                    source_store.inner_mut().migrate(row, target_store).unwrap();
                }
                self.get_storage(type_id)
                    .unwrap()
                    .inner_mut()
                    .swap_remove(row);
            }
        }

        (moved, target_row)
    }

    pub fn migrate_to_bundle(&self, row: usize) -> (EntityId, ComponentBundle) {
        let mut bundle: ComponentBundle = ComponentBundle::default();
        for idx in self.index.values() {
            let comp: ComponentBox = self.storage[*idx].inner_mut().swap_remove(row);
            bundle.insert(comp);
        }
        let entity = self.get_last_entity();
        self.entities_mut().swap_remove(row);

        (entity, bundle)
    }
}

impl Default for Archetype {
    fn default() -> Self {
        Self {
            index: HashMap::new(),
            storage: Box::new([]),
            entities: RwLock::new(Vec::new()),
            edges: HashMap::new(),
        }
    }
}

pub enum Migration {
    Add(ComponentBox),
    Remove(TypeId),
}

impl Migration {
    pub fn is_add(&self) -> bool {
        matches!(self, Self::Add(_))
    }
}
