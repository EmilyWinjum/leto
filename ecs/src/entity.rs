use std::ops::Range;

use crate::errors::EntityError;

/// Defines an `EntityId`. Contains both an `id` and `generation`
///
/// `EntityId`s contain identifiers for unique entites, iterating upwards by
/// generation when freed.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct EntityId {
    id: u32,
    generation: u32,
}

/// Defines a `Location`. Contains information about entity storage location
///
/// `Location`s contain information for an `Entity`'s linked `Archetype` and
/// its row within its storage.
#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Location {
    pub archetype: usize,
    pub row: usize,
}

impl Location {
    /// Generate a new location with given archetype and row
    pub fn new(archetype: usize, row: usize) -> Self {
        Self { archetype, row }
    }
}

/// Defines an `Entity`. Contains storage data and an identifier
///
/// `Entity` structs contain lookup information for finding attached
/// components within their associated archetypes.
#[derive(Debug, Default)]
pub struct Entity {
    generation: u32,
    location: Option<Location>,
}

impl Entity {
    /// checks the generation of a given `Entity` against the given target value
    fn check_generation(&self, gen: u32) -> Result<(), EntityError> {
        if gen == self.generation {
            Ok(())
        } else {
            Err(EntityError::WrongGen)
        }
    }
}

/// Defines an `EntityStore`. Contains a list of `Entity`s in service as well as freed `EntityId`s
/// for reuse.
///
/// `EntityStore`s track all `EntityId`s and ensures their uniqueness.
#[derive(Default)]
pub struct EntityStore {
    entities: Vec<Entity>,
    freed: Vec<u32>,
    count: u32,
}

impl EntityStore {
    /// Get the location option from the target entity, returning an error if nothing was found
    pub fn entity_status(&self, id: EntityId) -> Result<Option<Location>, EntityError> {
        let entity: &Entity = self
            .entities
            .get(id.id as usize)
            .ok_or(EntityError::NotFound)?;

        entity
            .check_generation(id.generation)
            .and(Ok(entity.location))
    }

    /// Mutably gets an entity matching by both index and generation
    fn get_mut_entity(&mut self, id: EntityId) -> Result<&mut Entity, EntityError> {
        let entity: &mut Entity = self
            .entities
            .get_mut(id.id as usize)
            .ok_or(EntityError::NotFound)?;

        entity.check_generation(id.generation).and(Ok(entity))
    }

    /// Allocates new `entity`s into the `entities` collection, returning their ids
    fn seed_new_ids(&mut self, count: u32) -> Result<Range<u32>, EntityError> {
        let old_count: u32 = self.count;

        if let Some(new_count) = old_count.checked_add(count) {
            self.count = new_count;
            self.entities
                .extend((old_count..new_count).map(|_| Entity::default()));

            Ok(old_count..new_count)
        } else {
            Err(EntityError::TooManyEntities)
        }
    }

    /// Gets a collection of unique `EntityId`s from a combination of the `freed` list or
    /// by creating new ids as a fallback
    pub fn get_new_ids(&mut self, count: u32) -> Result<Vec<EntityId>, EntityError> {
        let free_count: u32 = count.min(self.freed.len() as u32);
        let mut ids: Vec<EntityId> = self
            .freed
            .drain(self.freed.len() - free_count as usize..)
            .map(|id| {
                let generation = self.entities[id as usize].generation;
                EntityId { id, generation }
            })
            .collect();

        if count > free_count {
            ids.extend(
                &mut self
                    .seed_new_ids(count - free_count)?
                    .map(|id| EntityId { id, generation: 0 }),
            );
        }

        Ok(ids)
    }

    /// Gets a unique `EntityId` from either the `freed` list or by creating a new id as a fallback
    pub fn get_new_id(&mut self) -> Result<EntityId, EntityError> {
        if let Some(id) = self.freed.pop() {
            let generation: u32 = self.entities[id as usize].generation;

            Ok(EntityId { id, generation })
        } else if self.count < u32::MAX {
            let id: u32 = self.count;
            self.count += 1;
            self.entities.push(Entity::default());

            Ok(EntityId { id, generation: 0 })
        } else {
            Err(EntityError::TooManyEntities)
        }
    }

    /// Resets the location for a given `EntityId`, adding it to the `freed` list
    ///
    /// Returns the freed location, expecting this data to be cleared in its `Archetype`
    pub fn free(&mut self, id: EntityId) -> Result<Location, EntityError> {
        let entity: &mut Entity = self.get_mut_entity(id)?;
        let old_location: Location = entity.location.ok_or(EntityError::AlreadyFreed)?;
        entity.location = None;
        entity.generation += 1;
        self.freed.push(id.id);

        Ok(old_location)
    }

    /// Updates the inner `Location` for a given `EntityId`
    ///
    /// Returns the freed location, expecting this data to be or have been cleared already in its `Archetype`
    pub fn set_location(&mut self, id: EntityId, location: Location) -> Option<Location> {
        let entity: &mut Entity = self.get_mut_entity(id).unwrap();
        let old_location: Option<Location> = entity.location;
        entity.location = Some(location);

        old_location
    }

    /// Updates the locations of continuous `Entities` within an `Archetype`
    ///
    /// Expects all provided ids to contain no `Locations`
    pub fn set_many_location(&mut self, ids: &[EntityId], start: Location) {
        for (count, id) in ids.iter().cloned().enumerate() {
            let entity: &mut Entity = self.get_mut_entity(id).unwrap();
            entity.location = Some(Location::new(start.archetype, start.row + count))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_entity(generation: u32, location: Option<Location>) -> Entity {
        Entity {
            location,
            generation,
        }
    }

    #[test]
    fn test_get_new_id() -> Result<(), EntityError> {
        let mut store: EntityStore = EntityStore::default();

        let id: EntityId = store.get_new_id()?;

        assert!(
            id == EntityId {
                id: 0,
                generation: 0
            }
        );
        assert!(store.entity_status(id)?.is_none());

        Ok(())
    }

    #[test]
    fn test_get_new_id_freed() -> Result<(), EntityError> {
        let mut store: EntityStore = EntityStore {
            entities: Vec::from([mock_entity(1, None)]),
            freed: Vec::from([0]),
            count: 1,
        };

        let id: EntityId = store.get_new_id()?;

        assert!(
            id == EntityId {
                id: 0,
                generation: 1,
            }
        );
        assert!(store.entity_status(id)?.is_none());

        Ok(())
    }

    #[test]
    fn test_get_new_id_full() -> Result<(), EntityError> {
        let mut store: EntityStore = EntityStore {
            entities: Vec::new(),
            freed: Vec::new(),
            count: u32::MAX,
        };

        let id = store.get_new_id();

        assert!(id.is_err());
        assert!(matches!(id.unwrap_err(), EntityError::TooManyEntities));

        Ok(())
    }

    #[test]
    fn test_free_id() -> Result<(), EntityError> {
        let location = Location::new(0, 0);

        let mut store: EntityStore = EntityStore {
            entities: Vec::from([mock_entity(0, Some(location))]),
            freed: Vec::new(),
            count: 1,
        };

        let mut id: EntityId = EntityId {
            id: 0,
            generation: 0,
        };

        let free_res: Result<Location, EntityError> = store.free(id);

        assert!(free_res.is_ok() && free_res? == location);

        assert!(store.freed[0] == id.id);
        assert!(store.entity_status(id).is_err());

        id.generation += 1;
        assert!(store.entity_status(id)?.is_none());

        Ok(())
    }

    #[test]
    fn test_free_id_bad_id() {
        let bad_id: EntityId = EntityId {
            id: 0,
            generation: 0,
        };

        let mut store: EntityStore = EntityStore {
            entities: Vec::new(),
            freed: Vec::new(),
            count: 0,
        };

        let free_res: Result<Location, EntityError> = store.free(bad_id);

        assert!(free_res.is_err());
        assert!(matches!(free_res.unwrap_err(), EntityError::NotFound));
    }

    #[test]
    fn test_free_id_already_freed() {
        let id: EntityId = EntityId {
            id: 0,
            generation: 0,
        };

        let mut store: EntityStore = EntityStore {
            entities: Vec::from([mock_entity(id.generation, None)]),
            freed: Vec::new(),
            count: 1,
        };

        let free_res: Result<Location, EntityError> = store.free(id);

        assert!(free_res.is_err());
        assert!(matches!(free_res.unwrap_err(), EntityError::AlreadyFreed));
    }

    #[test]
    fn test_set_location() {
        let location = Location::new(0, 0);

        let mut store = EntityStore {
            entities: Vec::from([Entity::default()]),
            freed: Vec::new(),
            count: 1,
        };

        let previous: Option<Location> = store.set_location(
            EntityId {
                id: 0,
                generation: 0,
            },
            location,
        );

        assert!(previous.is_none());
        assert!(store.entities[0].location == Some(location));
    }
}
