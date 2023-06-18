use ecs::{archetype::Migration, bundle::ComponentBundle, world::World};
use ecs_derive::{Component, QueryModel};

#[derive(Component, Default, PartialEq, Debug)]
pub struct TestCompA {
    _one: u32,
    _two: String,
}

#[derive(Component, Default, PartialEq, Debug)]
pub struct TestCompB {
    _three: u32,
    _four: String,
}

#[derive(Component, Default, PartialEq, Debug)]
pub struct TestCompC {
    _five: u32,
    _six: String,
}

#[derive(QueryModel)]
pub struct TestDataA<'a> {
    comp_a: &'a TestCompA,
    comp_b: &'a mut TestCompB,
    comp_c: &'a TestCompC,
}

fn test_system(row: TestDataA) {
    let a = row.comp_a;
    let mut b = row.comp_b;
    let c = row.comp_c;

    println!("{:?}, {:?}, {:?}", a, b, c);

    b._three += 5;
    b._four = "Five".to_string();

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
