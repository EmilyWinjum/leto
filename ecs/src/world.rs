use std::collections::HashMap;

use crate::{
    entity::{EntityStore, EntityId, Location,}, 
    errors::EcsError, component::{ComponentBundle, Types}, archetype::Archetype,
};


pub struct World<'a> {
    index: HashMap<Types, usize>,
    archetypes: Vec<Archetype<'a>>,
    entities: EntityStore,
}

impl World<'_> {
    pub fn init() -> Self {
        let default_archetype: Archetype = Archetype::default();
        Self {
            index: HashMap::from([(Types::default(), 0)]),
            archetypes: Vec::from([default_archetype]),
            entities: EntityStore::init(),
        }
    }

    pub fn spawn(&mut self, bundle: ComponentBundle) -> Result<EntityId, EcsError> {
        let entity: EntityId = self.entities.get_new_id()?;
        let types: Types = bundle.types();

        self.entities.set_location(
            entity, 
            if let Some(archetype_id) = self.get_archetype_id(types.clone()) {
                Location::new(
                    archetype_id, 
                    self.archetypes[archetype_id].add_entity(bundle, entity)
                )
            }
            else {
                let archetype_id: usize = self.archetypes.len();
                self.index.insert(types, archetype_id);
                self.archetypes.push(Archetype::new(bundle, entity));

                Location::new(archetype_id, 0)
            }
        );

        Ok(entity)
    }

    pub fn kill(&mut self, entity: EntityId) -> Result<(), EcsError> {
        let location = self.entities.get_location(entity)?;

        self.archetypes[location.archetype].remove_entity(location.row);
        self.entities.free(entity)?;

        Ok(())
    }

    fn get_archetype_id(&self, types: Types) -> Option<usize> {
        self.index.get(&types).copied()
    }

}
/*

impl World<'_> {
    pub fn new() -> Self {
        Self {
            archetypes: ArchetypeStore::init(),
            entities: EntityStore::init(),
        }
    }
    
    pub fn spawn(&mut self, entity: ComponentBundle) -> Result<EntityId, EcsError> {
        let id: EntityId = self.entities.get_new_id()?;

        let location = self.archetypes.add_entity(id, entity);

        todo!();

    }

    /// Stores data provided by an `EntityBuilder::Done` variant using the provided function to
    /// initialize storage for the new `Entity`
    pub fn store<F>(&mut self, data: EntityInitData, mut init: F) -> Result<EntityId, EntityError> 
    where F: FnMut(EntityInitData) -> Result<u32, ArchetypeError>
    {
            let archetype = data.archetype.clone();
            let id = self.get_id()?;
            let row = init(data)?;
            self.fetch_entity_mut(&id)?.location = Some(Location { archetype, row });

            Ok(id)
    }

    /// Frees the data of a given `Entity` in `Archetype` storage and marks it in
    /// the `freed` collection
    pub fn free<'a, F>(&mut self, id: &EntityId, mut clear: F) -> Result<(), EntityError>
        where F: FnMut(&Location) -> Result<EntityId, ArchetypeError> 
        // returns the entity in the last row of archetype before removal
    {
        let location = self.fetch_entity(id)?.unwrap_location()?;
        let moved = clear(&location)?;

        if moved != *id {
            self.fetch_entity_mut(&moved)?.location = Some(location);
        }

        self.fetch_entity_mut(id)?.reset();
        self.freed.push(id.id as usize);

        Ok(())
    }

    /// Edits the given `Entity` using the given function to modify the location of the entity
    pub fn edit<T, F>(&mut self, id: &EntityId, component: &T, mut edit: F) -> Result<(), EntityError>
        where F: FnMut(&Location, &T) -> Result<(EntityId, Location), ArchetypeError>
    {
        let location = self.fetch_entity(id)?.unwrap_location()?;
        let (moved, new_loc) = edit(&location, component)?;

        if moved != *id {
            self.fetch_entity_mut(&moved)?.location = Some(location);
        }

        self.fetch_entity_mut(id)?.location = Some(new_loc);

        Ok(())
    }
    
}*/