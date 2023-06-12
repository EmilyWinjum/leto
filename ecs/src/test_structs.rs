use crate::component::{Component, ComponentStore};
use ecs_derive::Component;

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
