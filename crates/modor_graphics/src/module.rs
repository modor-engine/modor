use crate::surface::WindowInit;
use crate::SurfaceSize;
use modor::{Built, EntityBuilder};
use modor_physics::PhysicsModule;

pub struct GraphicsModule;

#[singleton]
impl GraphicsModule {
    pub fn build<T>(window_size: SurfaceSize, window_title: T) -> impl Built<Self>
    where
        T: Into<String>,
    {
        EntityBuilder::new(Self)
            .with_child(WindowInit::build(window_size, window_title.into()))
            .with_dependency(PhysicsModule::build())
    }

    pub fn build_windowless() -> impl Built<Self> {
        EntityBuilder::new(Self).with_dependency(PhysicsModule::build())
    }
}
