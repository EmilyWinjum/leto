mod ecs;

use ecs::{
    components::*,
};

fn main() {
    let mut world = ecs::World::new();
    
    // Icarus's health is *not* looking good.
    world.new_entity(Some(Health(-10)), Some(Name("Icarus")));

    // Prometheus is very healthy.
    world.new_entity(Some(Health(100)), Some(Name("Prometheus")));

    // Note that Zeus does not have a `Health` component.
    world.new_entity(None, Some(Name("Zeus")));

    let zip = world
        .borrow_component_vec::<Health>()
        .unwrap()
        .iter()
        .zip(world.borrow_component_vec::<Name>().unwrap().iter());

    let with_health_and_name = zip.filter_map(|(health, name): (&Option<Health>, &Option<Name>)| {
        Some((health.as_ref()?, name.as_ref()?))
    });

    for (health, name) in with_health_and_name {
        if health.0 < 0 {
            println!("{} has perished!", name.0);
        } else {
            println!("{} is still healthy", name.0);
        }
    }
}
