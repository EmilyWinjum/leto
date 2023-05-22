use std::{collections::HashMap, any::TypeId};

use crate::{component::{ComponentStore, ComponentBundle}, entity::EntityId};


pub enum ArchetypeEdge<'a> {
    Add(&'a Archetype<'a>),
    Remove(&'a Archetype<'a>),
}


pub struct Archetype<'a> {
    index: HashMap<TypeId, usize>,
    storage: Box<[ComponentStore]>,
    entities: Vec<EntityId>,
    edges: HashMap<TypeId, ArchetypeEdge<'a>>,
}

impl Archetype<'_> {
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

    fn get_storage_mut(&mut self, type_id: TypeId) -> Option<&mut ComponentStore> {
        self.index.get(&type_id)
            .and_then(|&idx| Some(&mut self.storage[idx]))
    }

    pub fn add_entity(&mut self, bundle: ComponentBundle, entity_id: EntityId) -> usize {
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

    pub fn remove_entity(&mut self, row: usize) -> EntityId {
        let entity: EntityId = self.entities.last()
            .expect("expected archetype to contain entities")
            .clone();
        for idx in self.index.values() {
            let store = &mut self.storage[*idx];
            store.swap_remove(row);
        }
        self.entities.swap_remove(row);
        entity
    }
}

impl Default for Archetype<'_> {
    fn default() -> Self {
        Self {
            index: HashMap::new(),
            storage: Box::new([]),
            entities: Vec::new(),
            edges: HashMap::new(),
        }
    }
}