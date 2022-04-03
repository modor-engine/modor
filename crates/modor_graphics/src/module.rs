use crate::window::WindowInit;
use crate::window::WindowSize;
use modor::{Built, EntityBuilder};
use modor_physics::PhysicsModule;

pub struct GraphicsModule;

#[singleton]
impl GraphicsModule {
    pub fn build<T>(window_size: WindowSize, window_title: T) -> impl Built<Self>
    where
        T: Into<String>,
    {
        EntityBuilder::new(Self)
            .with_child(WindowInit::build(window_size, window_title.into()))
            .with_dependency(PhysicsModule::build())
    }
}
