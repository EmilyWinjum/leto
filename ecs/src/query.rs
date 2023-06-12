use crate::{
    archetype::Archetype,
    bundle::TypeBundle,
    component::{ReadGuard, WriteGuard},
    world::World,
};

pub trait Model {
    type Row<'r>;
    fn get_types() -> TypeBundle;
    fn get_reads(at: &Archetype) -> Vec<ReadGuard>;
    fn get_writes(at: &Archetype) -> Vec<WriteGuard>;
    fn process<F>(reads: Vec<ReadGuard>, writes: Vec<WriteGuard>, row: usize, system: &mut F)
    where
        for<'m> F: FnMut(Self::Row<'m>);
}

fn run_system<M, F>(world: &World, system: &mut F)
where
    M: Model,
    for<'m> F: FnMut(M::Row<'m>),
{
    let bundle: TypeBundle = M::get_types();
    let archetypes: Vec<&Archetype> = world.get_archetypes_inclusive(&bundle);
    for (row, &at) in archetypes.iter().enumerate() {
        let reads: Vec<ReadGuard> = M::get_reads(at);
        let writes: Vec<WriteGuard> = M::get_writes(at);
        M::process(reads, writes, row, system)
    }
}
