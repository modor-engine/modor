use crate::rendering::Rendering;
use crate::targets::texture::TextureTarget;
use crate::targets::GpuDevice;
use log::info;
use modor::{Built, EntityBuilder, Single};
use modor_input::InputModule;
use modor_physics::PhysicsModule;

pub struct GraphicsModule {
    buffer: Vec<u8>,
}

#[singleton]
impl GraphicsModule {
    pub fn build() -> impl Built<Self> {
        info!("graphics module created with window");
        EntityBuilder::new(Self { buffer: vec![] })
            .with_dependency(PhysicsModule::build())
            .with_dependency(InputModule::build())
    }

    pub fn build_windowless() -> impl Built<Self> {
        info!("graphics module created without window`");
        EntityBuilder::new(Self { buffer: vec![] })
            .with_child(TextureTarget::build())
            .with_dependency(PhysicsModule::build())
    }

    #[run_after(component(Rendering))]
    fn retrieve_buffer(
        &mut self,
        target: Single<'_, TextureTarget>,
        device: Single<'_, GpuDevice>,
    ) {
        self.buffer = target.retrieve_buffer(&device.device);
    }

    // empty if window mode
    #[must_use]
    pub fn buffer(&self) -> &[u8] {
        &self.buffer
    }
}
