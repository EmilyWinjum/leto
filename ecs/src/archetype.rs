use std::{ collections::HashMap, any::TypeId, };

use crate::{component::{ComponentStore, ComponentBundle}, entity::{EntityId, Location}, errors::{ArchetypeError}};


/// Defines the type identifier for an `Archetype`. all immutable instances are sorted
#[derive(PartialEq, Clone, Hash, Eq)]
pub struct Types(Box<[TypeId]>);

impl From<Vec<TypeId>> for Types {
    fn from(id: Vec<TypeId>) -> Self {
        let mut copy: Vec<TypeId> = id.clone();
        copy.sort_unstable();

        Self(copy.as_slice().into())
    }
}

impl From<&ComponentBundle> for Types {
    fn from(bundle: &ComponentBundle) -> Self {
        let mut copy: Vec<TypeId> = bundle.types()
            .map(|ty: &TypeId| ty.clone())
            .collect();
        copy.sort_unstable();

        Self(copy.as_slice().into())
    }
}

impl Default for Types {
    fn default() -> Self {
        Self(Box::new([]))
    }
}


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
        let storage: Box<[ComponentStore]> = bundle.into_iter()
            .enumerate()
            .map(|(idx, (type_id, comp))| { 
                index.insert(type_id, idx); 
                let mut store = comp.create_store();
                store.push(comp);
                store
            })
            .collect();

        Self {
            index,
            storage,
            entities: Vec::from([entity_id]),
            edges: HashMap::new(),
        }
    }

    fn get_storage_for_type_mut(&mut self, type_id: TypeId) -> Result<&mut ComponentStore, ArchetypeError> {
        let &idx: &usize = self.index.get(&type_id)
            .ok_or(ArchetypeError::TypeNotAvailable)?;

        Ok(&mut self.storage[idx])
    }

    fn add_entity(&mut self, bundle: ComponentBundle, entity_id: EntityId) -> u32 {
        let row = self.entities.len();
        bundle.into_iter()
            .for_each(|(type_id, comp)| {
                self.get_storage_for_type_mut(type_id)
                    .expect("couldn't find storage")
                    .push(comp)
                    .expect("couldn't push to store");
            });
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

    pub fn add_entity(&mut self, entity_id: EntityId, bundle: ComponentBundle) -> Location {
        let types: Types = Types::from(&bundle);

        if let Some(&archetype_id) = self.index.get(&types) {
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
        }
        
    }

    fn lookup_archetype(&self, types: Types) {

    }

    fn get_archetype(&self, index: u32) {

    }

}