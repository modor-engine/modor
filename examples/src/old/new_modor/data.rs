use crate::new_modor::App;
use std::any::Any;

pub trait Singleton: Default + Any {
    #[allow(unused_variables)]
    fn update(app: &mut App) {}
}

pub trait Data: Default + Any {
    #[allow(unused_variables)]
    fn update(app: &mut App) {}
}
