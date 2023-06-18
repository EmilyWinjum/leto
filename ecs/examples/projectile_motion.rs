use std::time::Instant;

use ecs::{bundle::ComponentBundle, world::World};
use ecs_derive::{Component, QueryModel};

#[derive(Component, Debug, Default)]
struct Position {
    // meters
    x: f64,
    y: f64,
}

#[derive(Component, Debug, Default)]
struct Velocity {
    // meters/tick
    dx: f64,
    dy: f64,
}

#[derive(Component, Debug, Default)]
struct Mass(pub f32); // kilograms

#[derive(Component, Debug)]
struct Time {
    pub last: Instant,
    pub total: f64,
}

#[derive(QueryModel)]
struct PhysicsQuery<'p> {
    pos: &'p mut Position,
    vel: &'p mut Velocity,
    _mass: &'p Mass,
    time: &'p mut Time,
}

fn gravity_system(is_moving: &mut bool, row: PhysicsQuery) {
    if row.pos.y < 0. {
        *is_moving = false;
        println!("landed at {:?} in {:?}", row.pos, row.time.total);
    }

    let step = row.time.last.elapsed().as_secs_f64();
    row.time.last = Instant::now();
    row.time.total += step;

    row.pos.x += row.vel.dx * step;
    row.pos.y += row.vel.dy * step - 4.9 * step.powi(2);

    row.vel.dy -= 9.8 * step;
}

fn main() {
    let start = Instant::now();
    let mut world = World::init();

    let bundle = ComponentBundle::default()
        .insert(Position { x: 0., y: 5. })
        .insert(Velocity { dx: 1., dy: 0. })
        .insert(Mass(1.))
        .insert(Time {
            last: Instant::now(),
            total: 0.,
        });

    world.spawn(bundle).unwrap();

    let mut is_moving = true;

    while is_moving {
        world.run_system::<PhysicsQuery, _>(&mut |row| gravity_system(&mut is_moving, row));
    }
    println!("{:?}", start.elapsed().as_secs_f64());
}
