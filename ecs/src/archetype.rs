use std::{collections::HashMap, any::TypeId};

use crate::{component::{ComponentStore, ComponentBundle, Types}, entity::{EntityId, Location}, errors::ArchetypeError};


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
        bundle.components()
            .into_iter()
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
        if let Some(&idx) = self.index.get(&type_id) {
            Some(&mut self.storage[idx])
        }
        else {
            None
        }
    }

    fn add_entity(&mut self, bundle: ComponentBundle, entity_id: EntityId) -> u32 {
        let row = self.entities.len();
        for comp in bundle.components() {
            self.get_storage_mut(comp.type_id())
            .expect("should match storage types with bundle")
            .push(comp)
            .expect("expected to push");
        }
        self.entities.push(entity_id);
        self.assert_intact();

        row as u32
    }

    fn assert_intact(&self) {
        assert!(self.index.len() == self.storage.len());

        let len = self.entities.len();
        self.storage.iter()
            .for_each(|cs: &ComponentStore| assert!(cs.len() == len))
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


pub struct ArchetypeStore<'a> {
    index: HashMap<Types, usize>,
    archetypes: Vec<Archetype<'a>>
}

impl ArchetypeStore<'_> {
    pub fn init() -> Self {
        let default_archetype: Archetype = Archetype::default();
        Self {
            index: HashMap::from([(Types::default(), 0)]),
            archetypes: Vec::from([default_archetype]),
        }
    }

    pub fn add_entity(&mut self, entity_id: EntityId, bundle: ComponentBundle) -> Result<Location, ArchetypeError> {
        let types: Types = bundle.types();

        let location: Location = if let Some(&archetype_id) = self.index.get(&types) {
            Location::new(
                archetype_id, 
                self.archetypes[archetype_id].add_entity(bundle, entity_id)
            )
        }
        else {
            let archetype_id: usize = self.archetypes.len();
            self.index.insert(types, archetype_id);
            self.archetypes.push(Archetype::new(bundle, entity_id));

            Location::new(archetype_id, 0)
        };

        Ok(location)
    }

}