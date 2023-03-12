use crate::data::size::NonZeroSize;
use crate::gpu_data::uniform::Uniform;
use crate::render_target::{RenderTargetRegistry, RenderTargetUpdate};
use crate::{
    GraphicsModule, IntoResourceKey, RenderTarget, Resource, ResourceKey, ResourceRegistry,
    ResourceState, Window,
};
use fxhash::FxHashMap;
use modor::{Query, Single, SingleMut};
use modor_math::{Mat4, Quat, Vec2, Vec3};
use modor_physics::Transform2D;

pub(crate) type Camera2DRegistry = ResourceRegistry<Camera2D>;

#[must_use]
#[derive(Component, Debug)]
pub struct Camera2D {
    pub target_keys: Vec<ResourceKey>,
    key: ResourceKey,
    transform: Transform2D,
    target_uniforms: FxHashMap<ResourceKey, Uniform<CameraData>>,
}

#[systems]
impl Camera2D {
    const CAMERA_BINDING: u32 = 0;

    pub fn new(key: impl IntoResourceKey) -> Self {
        Self {
            target_keys: vec![],
            key: key.into_key(),
            transform: Transform2D::default(),
            target_uniforms: FxHashMap::default(),
        }
    }

    pub fn with_target_key(mut self, key: impl IntoResourceKey) -> Self {
        self.target_keys.push(key.into_key());
        self
    }

    #[run_after(RenderTargetUpdate)]
    fn update(
        &mut self,
        transform: Option<&Transform2D>,
        (mut target_registry, targets): (
            SingleMut<'_, RenderTargetRegistry>,
            Query<'_, &RenderTarget>,
        ),
        module: Option<Single<'_, GraphicsModule>>,
    ) {
        self.transform = transform.cloned().unwrap_or_default();
        if let Some(module) = module {
            for target_key in &self.target_keys {
                if let Some(surface_size) = target_registry
                    .get(target_key, &targets)
                    .and_then(RenderTarget::window_surface_size)
                {
                    let transform = self.gpu_matrix(surface_size).to_array();
                    self.target_uniforms
                        .entry(target_key.clone())
                        .and_modify(|u| u.transform = transform)
                        .or_insert_with(|| Self::create_uniform(&module, transform));
                }
            }
            self.target_uniforms.retain(|_, u| u.is_changed());
            for uniform in self.target_uniforms.values_mut() {
                uniform.sync(&module);
            }
        } else {
            self.target_uniforms.clear();
        }
    }

    pub(crate) fn uniform(&self, target_key: &ResourceKey) -> &Uniform<CameraData> {
        self.target_uniforms
            .get(target_key)
            .expect("internal error: camera uniform not initialized")
    }

    fn create_uniform(module: &GraphicsModule, transform: [[f32; 4]; 4]) -> Uniform<CameraData> {
        Uniform::new(
            CameraData { transform },
            Self::CAMERA_BINDING,
            &module.camera_bind_group_layout,
            "camera_2d",
            &module.device,
        )
    }

    pub fn world_position(&self, window: &Window, window_position: Vec2) -> Vec2 {
        let target_size: Vec2 = window.size.into();
        self.world_matrix(target_size)
            * Vec2::new(
                window_position.x / target_size.x - 0.5,
                0.5 - window_position.y / target_size.y,
            )
    }

    fn gpu_matrix(&self, surface_size: NonZeroSize) -> Mat4 {
        let surface_size: Vec2 = surface_size.into();
        let x_scale = 1.0_f32.min(surface_size.y / surface_size.x);
        let y_scale = 1.0_f32.min(surface_size.x / surface_size.y);
        let position = Vec3::new(-self.transform.position.x, -self.transform.position.y, -1.);
        let scale = Vec3::new(
            2. * x_scale / self.transform.size.x,
            2. * y_scale / self.transform.size.y,
            -1.,
        );
        Mat4::from_position(position)
            * Quat::from_z(*self.transform.rotation).matrix()
            * Mat4::from_scale(scale)
    }

    fn world_matrix(&self, target_size: Vec2) -> Mat4 {
        let x_scale = 1.0_f32.min(target_size.y / target_size.x);
        let y_scale = 1.0_f32.min(target_size.x / target_size.y);
        let scale = self
            .transform
            .size
            .with_scale(Vec2::new(1. / x_scale, 1. / y_scale));
        Mat4::from_scale(scale.with_z(1.))
            * Quat::from_z(-*self.transform.rotation).matrix()
            * Mat4::from_position(self.transform.position.with_z(0.))
    }
}

impl Resource for Camera2D {
    fn key(&self) -> &ResourceKey {
        &self.key
    }

    fn state(&self) -> ResourceState<'_> {
        ResourceState::Loaded
    }
}

#[repr(C)]
#[derive(Default, Clone, Copy, Debug, PartialEq, bytemuck::Zeroable, bytemuck::Pod)]
pub(crate) struct CameraData {
    pub(crate) transform: [[f32; 4]; 4],
}
