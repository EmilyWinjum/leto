mod utils;
use utils::*;

use ecs::{archetype::Migration, bundle::ComponentBundle, world::World};
use ecs_derive::QueryModel;

#[derive(QueryModel)]
pub struct TestDataA<'a> {
    comp_a: &'a TestCompA,
    comp_b: &'a mut TestCompB,
    comp_c: &'a TestCompC,
}

fn test_system(row: TestDataA) {
    let a: &TestCompA = row.comp_a;
    let mut b: &mut TestCompB = row.comp_b;
    let c: &TestCompC = row.comp_c;

    println!("{:?}, {:?}, {:?}", a, b, c);

    b.three += 5.;
    b.four = "Five".to_string();

    println!("{:?}, {:?}, {:?}", a, b, c);
}

#[test]
fn test_world() {
    let mut world: World = World::init();

    let bundle: ComponentBundle = ComponentBundle::default()
        .insert(TestCompA::default())
        .insert(TestCompC::default());

    let entity_a = world.spawn(bundle).unwrap();

    world
        .migrate(entity_a, Migration::Add(TestCompB::default().into()))
        .unwrap();

    world.run_system::<TestDataA, _>(&mut test_system);
}
