use std::{
    any::Any,
    cell::RefCell,
};


pub trait Component: Any + Sized { }

/// Trait to define component vector behavior
pub trait ComponentVec {

    /// Casts 
    fn as_any(&self) -> &dyn Any;

    /// 
    fn as_any_mut(&mut self) -> &mut dyn Any;


    fn push_none(&mut self);
}


impl<T: Component> ComponentVec for RefCell<Vec<Option<T>>> {
    fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self as &mut dyn Any
    }

    fn push_none(&mut self) {
        self.get_mut().push(None);
    }
}