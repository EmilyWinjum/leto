use crate as ecs;
use ecs_derive::Component;

#[derive(Component, Default, PartialEq, Debug)]
pub struct TestCompA {
    pub one: u32,
    pub two: String,
}

#[derive(Component, Default, PartialEq, Debug)]
pub struct TestCompB {
    pub three: f32,
    pub four: String,
}

#[derive(Component, Default, PartialEq, Debug)]
pub struct TestCompC {
    pub five: Vec<usize>,
    pub six: String,
}
