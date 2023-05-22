use std::ops::Range;

use crate::errors::EntityError;


/// Defines an `EntityId`. Contains both an `id` and `generation`
/// 
/// `EntityId`s contain identifiers for unique entites, iterating upwards by
/// generation when freed.
#[derive(PartialEq, Clone, Copy)]
pub struct EntityId {
    id: u32,
    generation: u32,
}


/// Defines a `Location`. Contains information about entity storage location
/// 
/// `Location`s contain information for an `Entity`'s linked `Archetype` and
/// its row within its storage.
#[derive(Clone, Copy)]
pub struct Location {
    pub archetype: usize,
    pub row: usize,
}

impl Location {
    pub fn new(archetype: usize, row: usize) -> Self {
        Self {
            archetype,
            row,
        }
    }
}


/// Defines an `Entity`. Contains storage data and an identifier
/// 
/// `Entity` structs contain lookup information for finding attached
/// components within their associated archetypes.
pub struct Entity {
    generation: u32,
    location: Option<Location>,
}

impl Entity {
    /// Create a new `Entity` of `generation` 0 that has not been given storage.
    pub fn new() -> Self {
        Self {
            generation: 0,
            location: None,
        }
    }
}


/// Defines an `EntityStore`. Contains a list of `Entity`s in service as well as freed `EntityId`s
/// for reuse.
/// 
/// `EntityStore`s track all `EntityId`s and ensures their uniqueness.
pub struct EntityStore {
    entities: Vec<Entity>,
    freed: Vec<EntityId>,
}

impl EntityStore {
    /// Creates a new, empty `EntityStore`
    pub fn init() -> Self {
        Self {
            entities: Vec::new(),
            freed: Vec::new(),
        }
    }

    fn entity_status(&self, id: EntityId) -> Result<Option<Location>, EntityError> {
        let location = self.entities.get(id.id as usize)
            .ok_or(EntityError::NotFound)?
            .location;

        Ok(location)
    }

    /// Mutably gets an entity matching by both index and generation
    fn get_mut_entity(&mut self, id: EntityId) -> &mut Entity {
        self.entities.get_mut(id.id as usize).expect("expected to find entity")
    }

    /// Allocates new `entity`s into the `entities` collection, returning their ids
    fn seed_new_ids(&mut self, count: u32) -> Result<Range<u32>, EntityError> {
        let entities_len = self.entities.len() as u32;

        if let Some(new_len) = entities_len.checked_add(count) {
            self.entities.extend(
                (entities_len..new_len).map(|_| Entity::new())
            );

            Ok(entities_len..new_len)
        }
        else {
            Err(EntityError::TooManyEntities("".into()))
        }
    }

    /// Gets a collection of unique `EntityId`s from a combination of the `freed` list or
    /// by creating new ids as a fallback
    pub fn get_new_ids(&mut self, count: u32) -> Result<Vec<EntityId>, EntityError> {
        let free_count = count.min(self.freed.len() as u32);
        let mut ids: Vec<EntityId> = self.freed
            .drain(self.freed.len() - free_count as usize..)
            .collect();
        
        if count > free_count {
            ids.extend(
                &mut self.seed_new_ids(count - free_count)?
                .into_iter()
                .map(|id| EntityId { id, generation: 0, })
            );
        }
        
        Ok(ids)
    }

    /// Gets a unique `EntityId` from either the `freed` list or by creating a new id as a fallback
    pub fn get_new_id(&mut self) -> Result<EntityId, EntityError> {
        if let Some(id) = self.freed.pop() {
            Ok(id)
        }
        else {
            let id = u32::try_from(self.entities.len())?;
            self.entities.push(Entity::new());
            Ok(EntityId { id, generation: 0, })
        }
    }

    /// Resets the location for a given `EntityId`, adding it to the `freed` list
    /// 
    /// Returns the freed location, expecting this data to be cleared
    pub fn free(&mut self, id: EntityId) -> Result<Location, EntityError> {
        let location = self.get_location(id)?;
        let entity: &mut Entity = self.get_mut_entity(id);
        entity.location = None;
        self.freed.push(id.clone());

        Ok(location)
    }

    pub fn get_location(&self, id: EntityId) -> Result<Location, EntityError> {
        self.entity_status(id)?
            .ok_or(EntityError::AlreadyFreed)
    }

    /// Updates the inner `Location` for a given `EntityId`
    /// 
    /// # !!! Expects previous location to be irrelevant. Can cause storage leaks otherwise !!!
    pub fn set_location(&mut self, id: EntityId, location: Location) {
        let entity: &mut Entity = self.get_mut_entity(id);
        entity.location = Some(location);
    }

    pub fn clear_location(&mut self, id: EntityId) {
        self.get_mut_entity(id).location = None;
    }

    /// Updates the locations of continuous `Entities` within an archetype
    /// 
    /// # !!! Expects previous location to be irrelevant. Can cause storage leaks otherwise !!!
    pub fn set_many_location(&mut self, ids: &[EntityId], start: Location) {
        for (count, id) in ids.iter().cloned().enumerate() {
            let entity: &mut Entity = self.get_mut_entity(id);
            entity.location = Some(Location {
                archetype: start.archetype,
                row: start.row + count
            })
        }
    }
}