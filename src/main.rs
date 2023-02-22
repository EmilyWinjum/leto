extern crate leto;

use crate::leto::{
    world::world,
    components::*,
    systems::*,
};

fn main() {
    let mut world = world::World::new();
    
    // Icarus's health is *not* looking good.
    // world.new_entity(Some(Health(-10)), Some(Name("Icarus")));
    let icarus = world.new_entity();
    world.add_component_to_entity(icarus, Health(-10));
    world.add_component_to_entity(icarus, Name("Icarus"));

    // Prometheus is very healthy.
    // world.new_entity(Some(Health(100)), Some(Name("Prometheus")));
    let prometheus = world.new_entity();
    world.add_component_to_entity(prometheus, Health(100));
    world.add_component_to_entity(prometheus, Name("Prometheus"));


    // Note that Zeus does not have a `Health` component.
    // world.new_entity(None, Some(Name("Zeus")));
    let zeus = world.new_entity();
    world.add_component_to_entity(zeus, Name("Zeus"));

    for i in 0..5 {
        let mut healths = world.borrow_component_vec_mut::<Health>().unwrap();
        let mut names = world.borrow_component_vec_mut::<Name>().unwrap();
        let zip = healths.iter_mut().zip(names.iter_mut());
        let iter = zip.filter_map(|(health, name)| Some((health.as_mut()?, name.as_mut()?)));
        for (health, name) in iter {
            if health.0 < 0 {
                println!("{} has perished!", name.0);
            } else {
                println!("{} is still healthy", name.0);
            }
    
            if name.0 == "Icarus" && health.0 <= 0 {
                *health = Health(100);
                println!("{} has been revived! Health to {}", name.0, health.0);
            }
        }
        println!("Tick: {}", i);
    }
}
