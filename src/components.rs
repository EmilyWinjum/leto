use crate::world::comp::Component;

pub struct Health(pub i32);

impl Component for Health { }

pub struct Name(pub &'static str);

impl Component for Name { }

pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Component for Position { }

pub struct Velocity {
    pub x: i32,
    pub y: i32,
}

impl Component for Velocity { }

pub struct Acceleration {
    pub ddx: i32,
    pub y: i32,
}

impl Component for Acceleration { }
