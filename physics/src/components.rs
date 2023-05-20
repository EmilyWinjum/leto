pub struct Health(pub i32);

pub struct Name(pub &'static str);

pub struct Position {
    pub x: i32,
    pub y: i32,
}

pub struct Velocity {
    pub dx: i32,
    pub dy: i32,
}

pub struct Acceleration {
    pub ddx: i32,
    pub y: i32,
}
