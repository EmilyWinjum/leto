use crate::{
    archetype::Archetype,
    bundle::TypeBundle,
    component::{ReadGuard, WriteGuard},
};

pub trait QueryModel {
    type Row<'r>;
    fn get_types() -> TypeBundle;
    fn get_reads(at: &Archetype) -> Vec<ReadGuard>;
    fn get_writes(at: &Archetype) -> Vec<WriteGuard>;
    fn process<F>(reads: Vec<ReadGuard>, writes: Vec<WriteGuard>, system: &mut F)
    where
        for<'m> F: FnMut(Self::Row<'m>);
}

/* EXAMPLE IMPL

impl Model for TestDataA<'_> {
    type Row<'r> = TestDataA<'r>;

    fn get_types() -> TypeBundle {
        TypeBundle::from([TypeId::of::<TestCompA>(), TypeId::of::<TestCompB>()].as_slice())
    }

    fn get_reads(at: &Archetype) -> Vec<ReadGuard> {
        vec![at.get_storage(TypeId::of::<TestCompA>()).unwrap().inner()]
    }

    fn get_writes(at: &Archetype) -> Vec<WriteGuard> {
        vec![at
            .get_storage(TypeId::of::<TestCompB>())
            .unwrap()
            .inner_mut()]
    }

    fn process<F>(
        reads: Vec<ReadGuard>,
        mut writes: Vec<WriteGuard>,
        row: usize,
        system: &mut F,
    ) where
        for<'a> F: FnMut(Self::Row<'a>),
    {
        let comp_a: &TestCompA = reads[0]
            .to_any()
            .downcast_ref::<Vec<TestCompA>>()
            .unwrap()
            .get(row)
            .unwrap();

        let comp_b: &mut TestCompB = writes[0]
            .to_any_mut()
            .downcast_mut::<Vec<TestCompB>>()
            .unwrap()
            .get_mut(row)
            .unwrap();

        let row: Self::Row<'_> = TestDataA { comp_a, comp_b };

        system(row);
    }
}
*/
