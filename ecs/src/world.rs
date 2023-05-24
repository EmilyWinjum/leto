use std::{collections::HashMap, any::TypeId};

use crate::{
    entity::{EntityStore, EntityId, Location,}, 
    errors::EcsError, component::{ComponentBundle, TypeBundle}, archetype::{Archetype, Migration},
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

    pub fn edit_component(&mut self, entity: EntityId, op: Migration) -> Result<(), EcsError> {
        let location: Location = self.entities.get_location(entity)?;
        let source_idx: usize = location.archetype;
        let new_type: TypeId = match &op {
            Migration::Add(comp) => {
                let add = comp.type_id();
                assert!(!self.archetypes[source_idx].has_type(add));
                add
            }
            Migration::Remove(type_id) => {
                assert!(self.archetypes[source_idx].has_type(*type_id));
                *type_id
            }
        };
        let moved: EntityId;
        let new_row: usize;


        let target_idx: usize = if let Some(&target_idx) = self.archetypes[source_idx].edges.get(&new_type) {
            let (source, target) = self.mutate_archetypes(source_idx, target_idx);
            (moved, new_row) = source.migrate(target, location.row, op);

            target_idx
        }
        else {
            let old_bundle: TypeBundle = self.archetypes[source_idx].types();
            let type_bundle: TypeBundle = if op.is_add() {
                old_bundle.add_type(new_type)
            }
            else {
                old_bundle.remove_type(new_type)
            };


            let target_idx = if let Some(target_idx) = self.get_archetype_id(type_bundle.clone()) {
                let (source, target) = self.mutate_archetypes(source_idx, target_idx);
                (moved, new_row) = source.migrate(target, location.row, op);
                
                target_idx
            }
            else {
                let migration = self.archetypes[source_idx].migrate_to_bundle(location.row);
                (moved, new_row) = (migration.0, 0);
                let target_idx = self.archetypes.len();
                let new_archetype = Archetype::new(migration.1, entity);
                self.index.insert(type_bundle, target_idx);
                self.archetypes.push(new_archetype);

                target_idx
            };

            self.archetypes[source_idx].edges.insert(new_type, target_idx);
            self.archetypes[target_idx].edges.insert(new_type, source_idx);

            target_idx
        };

        self.entities.set_location(entity, Location::new(target_idx, new_row));
        self.entities.set_location(moved, location);

        Ok(())
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

    fn mutate_archetypes(&mut self, first: usize, second: usize) -> (&mut Archetype, &mut Archetype) {
        assert!(first < second);
        let (a, b) = self.archetypes.split_at_mut(second);
        (&mut a[first], &mut b[0])
    }

}