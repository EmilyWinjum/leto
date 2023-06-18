use std::{any::TypeId, collections::HashMap};

use crate::{
    archetype::{Archetype, Migration},
    bundle::{ComponentBundle, TypeBundle},
    component::{ReadGuard, WriteGuard},
    entity::{EntityId, EntityStore, Location},
    errors::{EcsError, EntityError},
    query::QueryModel,
};

pub struct World {
    index: HashMap<TypeBundle, usize>,
    archetypes: Vec<Archetype>,
    entities: EntityStore,
    inclusive_index: HashMap<TypeBundle, Vec<usize>>,
}

impl World {
    pub fn init() -> Self {
        let default_archetype: Archetype = Archetype::default();
        Self {
            index: HashMap::from([(TypeBundle::default(), 0)]),
            archetypes: Vec::from([default_archetype]),
            entities: EntityStore::default(),
            inclusive_index: HashMap::new(),
        }
    }

    pub fn spawn(&mut self, bundle: ComponentBundle) -> Result<EntityId, EcsError> {
        let entity: EntityId = self.entities.get_new_id()?;
        let types: TypeBundle = bundle.types();

        let location: Location = if let Some(archetype_id) = self.get_archetype_id(&types) {
            Location::new(
                archetype_id,
                self.archetypes[archetype_id].add(bundle, entity),
            )
        } else {
            Location::new(self.push_archetype(bundle, entity), 0)
        };

        self.entities.set_location(entity, location);

        Ok(entity)
    }

    pub fn migrate(&mut self, entity: EntityId, op: Migration) -> Result<(), EcsError> {
        let location: Location = self
            .entities
            .entity_status(entity)?
            .ok_or(EntityError::NotFound)?;
        let source_idx: usize = location.archetype;
        let new_type: TypeId = match &op {
            Migration::Add(comp) => {
                let add: TypeId = comp.inner_type_id();
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

        let target_idx: usize = if let Some(&target_idx) =
            self.archetypes[source_idx].edges.get(&new_type)
        {
            let (source, target) = self.mutate_archetypes(source_idx, target_idx);
            (moved, new_row) = source.migrate(target, location.row, op);

            target_idx
        } else {
            let old_bundle: TypeBundle = self.archetypes[source_idx].types();
            let type_bundle: TypeBundle = if op.is_add() {
                old_bundle.add_type(new_type)
            } else {
                old_bundle.remove_type(new_type)
            };

            let target_idx: usize = if let Some(target_idx) = self.get_archetype_id(&type_bundle) {
                let (source, target) = self.mutate_archetypes(source_idx, target_idx);
                (moved, new_row) = source.migrate(target, location.row, op);

                target_idx
            } else {
                let migration: (EntityId, ComponentBundle) =
                    self.archetypes[source_idx].migrate_to_bundle(location.row, op);

                moved = migration.0;
                new_row = 0;
                self.push_archetype(migration.1, entity)
            };

            self.archetypes[source_idx]
                .edges
                .insert(new_type, target_idx);
            self.archetypes[target_idx]
                .edges
                .insert(new_type, source_idx);

            target_idx
        };

        self.entities
            .set_location(entity, Location::new(target_idx, new_row));
        self.entities.set_location(moved, location);

        Ok(())
    }

    pub fn kill(&mut self, entity: EntityId) -> Result<(), EcsError> {
        let location = self.entities.free(entity)?;
        self.archetypes[location.archetype].remove(location.row);

        Ok(())
    }

    pub fn run_system<M, F>(&self, system: &mut F)
    where
        M: QueryModel,
        for<'m> F: FnMut(M::Row<'m>),
    {
        let bundle: TypeBundle = M::get_types();
        let archetypes: Vec<&Archetype> = self.get_archetypes_inclusive(&bundle);
        for &at in archetypes.iter() {
            let reads: Vec<ReadGuard> = M::get_reads(at);
            let writes: Vec<WriteGuard> = M::get_writes(at);
            M::process(reads, writes, system);
        }
    }

    pub fn get_archetypes_inclusive(&self, types: &TypeBundle) -> Vec<&Archetype> {
        self.inclusive_index
            .get(types)
            .cloned()
            .unwrap_or_default()
            .iter()
            .map(|idx| &self.archetypes[*idx])
            .collect()
    }

    fn get_archetype_id(&self, types: &TypeBundle) -> Option<usize> {
        self.index.get(types).copied()
    }

    fn push_archetype(&mut self, bundle: ComponentBundle, entity: EntityId) -> usize {
        let types: TypeBundle = bundle.types();
        let archetype_id: usize = self.archetypes.len();
        self.index.insert(types.clone(), archetype_id);
        self.archetypes.push(Archetype::new(bundle, entity));
        self.update_inclusive_index(types, archetype_id);

        archetype_id
    }

    fn update_inclusive_index(&mut self, types: TypeBundle, archetype_id: usize) {
        self.inclusive_index.iter_mut().for_each(|(t, v)| {
            if types.contains(t) {
                v.push(archetype_id)
            }
        });
        let ids: Vec<usize> = self
            .index
            .iter()
            .filter(|(t, _)| t.contains(&types))
            .map(|(_, &a)| a)
            .collect();
        self.inclusive_index.insert(types, ids);
    }

    fn mutate_archetypes(
        &mut self,
        first: usize,
        second: usize,
    ) -> (&mut Archetype, &mut Archetype) {
        assert!(first < second);
        let (a, b) = self.archetypes.split_at_mut(second);
        (&mut a[first], &mut b[0])
    }
}
