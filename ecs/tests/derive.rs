use ecs::query::Model;
use ecs::test_structs::*;
use ecs_derive::Model;

#[derive(Model)]
struct TestDataA<'a> {
    comp_a: &'a TestCompA,
    comp_b: &'a mut TestCompB,
}

fn test_system(data: TestDataA) {
    // do stuff
}

/* Used as example impl
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
