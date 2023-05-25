use std::{any::TypeId, collections::HashMap};

use crate::{
    component::{ComponentBox, ComponentBundle, ComponentStore, TypeBundle},
    entity::EntityId,
};

pub struct Archetype {
    index: HashMap<TypeId, usize>,
    storage: Box<[ComponentStore]>,
    entities: Vec<EntityId>,
    pub edges: HashMap<TypeId, usize>,
}

impl Archetype {
    fn get_last_entity(&self) -> EntityId {
        self.entities[self.entities.len()]
    }

    fn get_storage_mut(&mut self, type_id: TypeId) -> Option<&mut ComponentStore> {
        self.index.get(&type_id).map(|&idx| &mut self.storage[idx])
    }

    pub fn new(bundle: ComponentBundle, entity_id: EntityId) -> Self {
        let mut index: HashMap<TypeId, usize> = HashMap::new();
        let mut storage: Vec<ComponentStore> = Vec::new();
        bundle.component_iter().enumerate().for_each(|(idx, comp)| {
            index.insert(comp.type_id(), idx);
            storage.push(ComponentStore::new(comp));
        });

        Self {
            index,
            storage: storage.into(),
            entities: Vec::from([entity_id]),
            edges: HashMap::new(),
        }
    }

    pub fn types(&self) -> TypeBundle {
        (&self.index).into()
    }

    pub fn has_type(&self, type_id: TypeId) -> bool {
        self.index.get(&type_id).is_some()
    }

    pub fn add(&mut self, bundle: ComponentBundle, entity_id: EntityId) -> usize {
        let row = self.entities.len();
        for comp in bundle.component_iter() {
            self.get_storage_mut(comp.type_id())
                .expect("should match storage types with bundle")
                .push(comp)
                .expect("expected to push");
        }
        self.entities.push(entity_id);

        row
    }

    pub fn remove(&mut self, row: usize) -> EntityId {
        let entity: EntityId = self.get_last_entity();
        for idx in self.index.values() {
            self.storage[*idx].swap_remove(row);
        }
        self.entities.swap_remove(row);
        entity
    }

    pub fn migrate(&mut self, target: &mut Self, row: usize, op: Migration) -> (EntityId, usize) {
        let moved: EntityId = self.get_last_entity();
        let target_row = target.entities.len();
        let current = self.entities.swap_remove(row);
        target.entities.push(current);
        match op {
            Migration::Add(comp) => {
                for (&type_id, &idx) in self.index.iter() {
                    let source_store: &mut ComponentStore = &mut self.storage[idx];
                    let target_store: &mut ComponentStore = target
                        .get_storage_mut(type_id)
                        .expect("expected target to contain storage for type");
                    source_store
                        .migrate(row, target_store)
                        .expect("expected types to be compatible");
                }
                target
                    .get_storage_mut(comp.type_id())
                    .expect("expected to find type in archetype")
                    .push(comp)
                    .expect("expected types to be compatible");
            }
            Migration::Remove(type_id) => {
                for (&type_id, &idx) in target.index.iter() {
                    let source_store: &mut ComponentStore = self
                        .get_storage_mut(type_id)
                        .expect("expected self to contain storage for type");
                    let target_store: &mut ComponentStore = &mut target.storage[idx];
                    source_store
                        .migrate(row, target_store)
                        .expect("expected types to be compatible");
                }
                self.get_storage_mut(type_id)
                    .expect("expected to find type in archetype")
                    .swap_remove(row);
            }
        }

        (moved, target_row)
    }

    pub fn migrate_to_bundle(&mut self, row: usize) -> (EntityId, ComponentBundle) {
        let mut bundle: ComponentBundle = ComponentBundle::default();
        for idx in self.index.values() {
            let comp: ComponentBox = self.storage[*idx].swap_remove_to_box(row);
            bundle.push(comp);
        }
        let entity = self.get_last_entity();
        self.entities.swap_remove(row);

        (entity, bundle)
    }
}

impl Default for Archetype {
    fn default() -> Self {
        Self {
            index: HashMap::new(),
            storage: Box::new([]),
            entities: Vec::new(),
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
