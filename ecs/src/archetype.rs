use std::{collections::HashMap, any::TypeId};

use crate::{component::{ComponentStore, ComponentBundle, ComponentBox, TypeBundle}, entity::EntityId, errors::ArchetypeError};


pub enum ArchetypeEdge {
    Add(usize),
    Remove(usize),
}

impl ArchetypeEdge {
    pub fn unwrap_add(&self) -> usize {
        match self {
            Self::Add(archetype) => *archetype,
            _ => panic!("expected edge to be for addition"),
        }
    }

    pub fn unwrap_remove(&self) -> usize {
        match self {
            Self::Remove(archetype) => *archetype,
            _ => panic!("expected edge to be for removal"),
        }
    }
}


pub struct Archetype {
    index: HashMap<TypeId, usize>,
    storage: Box<[ComponentStore]>,
    entities: Vec<EntityId>,
    pub edges: HashMap<TypeId, ArchetypeEdge>,
}

impl Archetype {
    pub fn new(bundle: ComponentBundle, entity_id: EntityId) -> Self {
        let mut index: HashMap<TypeId, usize> = HashMap::new();
        let mut storage: Vec<ComponentStore> = Vec::new();
        bundle.component_iter()
            .enumerate()
            .for_each(|(idx, comp)| {
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

    fn get_storage_mut(&mut self, type_id: TypeId) -> Option<&mut ComponentStore> {
        self.index.get(&type_id)
            .and_then(|&idx| Some(&mut self.storage[idx]))
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

    fn get_last_entity(&self) -> EntityId {
        self.entities[self.entities.len()]
    }

    pub fn remove(&mut self, row: usize) -> EntityId {
        let entity: EntityId = self.get_last_entity();
        for idx in self.index.values() {
            self.storage[*idx]
                .swap_remove(row);
        }
        self.entities.swap_remove(row);
        entity
    }

    pub fn migrate_add(&mut self, row: usize, target: &mut Archetype, comp: ComponentBox) -> (EntityId, usize) {
        let moved: EntityId = self.get_last_entity();
        let target_row: usize = target.entities.len();
        let current: EntityId = self.entities.swap_remove(row);
        target.entities.push(current);
        for (&type_id, &idx) in self.index.iter() {
            let source: &mut ComponentStore = &mut self.storage[idx];
            let target: &mut ComponentStore = &mut target.get_storage_mut(type_id)
                .expect("expected target to contain storage for type");
            source.migrate(row, target)
                .expect("expected types to be compatible");
        }
        target.get_storage_mut(comp.type_id())
            .expect("expected to find type in archetype")
            .push(comp)
            .expect("expected types to be compatible");

        (moved, target_row)
    }

    pub fn migrate_remove(&mut self, row: usize, target: &mut Archetype, type_id: TypeId) -> (EntityId, usize) {
        let moved: EntityId = self.get_last_entity();
        let target_row: usize = target.entities.len();
        let current: EntityId = self.entities.swap_remove(row);
        target.entities.push(current);
        for (&type_id, &idx) in target.index.iter() {
            let source: &mut ComponentStore = &mut self.get_storage_mut(type_id)
                .expect("expected self to contain storage for type");
            let target: &mut ComponentStore = &mut target.storage[idx];
            source.migrate(row, target)
                .expect("expected types to be compatible");
        }
        self.get_storage_mut(type_id)
            .expect("expected to find type in archetype")
            .swap_remove(row);

        (moved, target_row)
    }

    pub fn migrate_to_bundle(&mut self, row: usize) -> Result<(EntityId, ComponentBundle), ArchetypeError> {
        let mut bundle: ComponentBundle = ComponentBundle::new();
        for idx in self.index.values() {
            let comp: ComponentBox = self.storage[*idx]
                    .swap_remove_to_box(row);
            bundle.push(comp);
        }
        let entity = self.get_last_entity();
        self.entities.swap_remove(row);

        Ok((entity, bundle))
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