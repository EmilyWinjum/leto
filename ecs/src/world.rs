use std::collections::HashMap;

use crate::{
    entity::{EntityStore, EntityId, Location,}, 
    errors::EcsError, component::{ComponentBundle, TypeBundle, ComponentBox}, archetype::{Archetype, ArchetypeEdge},
};


pub struct World {
    index: HashMap<TypeBundle, usize>,
    archetypes: Vec<Archetype>,
    entities: EntityStore,
}

impl World {
    pub fn init() -> Self {
        let default_archetype: Archetype = Archetype::default();
        Self {
            index: HashMap::from([(TypeBundle::default(), 0)]),
            archetypes: Vec::from([default_archetype]),
            entities: EntityStore::init(),
        }
    }

    pub fn spawn(&mut self, bundle: ComponentBundle) -> Result<EntityId, EcsError> {
        let entity: EntityId = self.entities.get_new_id()?;
        let types: TypeBundle = bundle.types();

        self.entities.set_location(
            entity, 
            if let Some(archetype_id) = self.get_archetype_id(types.clone()) {
                Location::new(
                    archetype_id, 
                    self.archetypes[archetype_id].add(bundle, entity)
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

    pub fn add_component(&mut self, entity: EntityId, comp: ComponentBox) -> Result<(), EcsError> {
        let location: Location = self.entities.get_location(entity)?;
        let source_idx: usize = location.archetype;

        if let Some(edge) = self.archetypes[source_idx].edges.get(&comp.type_id()) {
            let target_idx = edge.unwrap_add();
            (&mut self.archetypes[source_idx])
            .migrate_add(location.row, &mut self.archetypes[target_idx], comp);
        }

        todo!();
    }

    pub fn kill(&mut self, entity: EntityId) -> Result<(), EcsError> {
        let location = self.entities.get_location(entity)?;

        self.archetypes[location.archetype].remove(location.row);
        self.entities.free(entity)?;

        Ok(())
    }

    fn get_archetype_id(&self, types: TypeBundle) -> Option<usize> {
        self.index.get(&types).copied()
    }

}


/// A helper to get two mutable borrows from the same slice.
fn index_twice<T>(slice: &mut [T], first: usize, second: usize) -> (&mut T, &mut T) {
    if first < second {
        let (a, b) = slice.split_at_mut(second);
        (&mut a[first], &mut b[0])
    } else {
        let (a, b) = slice.split_at_mut(first);
        (&mut b[0], &mut a[second])
    }
}