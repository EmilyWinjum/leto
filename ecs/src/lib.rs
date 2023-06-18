pub mod archetype;
pub mod bundle;
pub mod component;
pub mod entity;
pub mod errors;
pub mod query;
pub mod world;

#[cfg(test)]
pub mod test_utils {
    use crate as ecs;
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
}
