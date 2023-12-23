use modor::{
    App, BuiltEntity, Changed, Component, EntityBuilder, NoSystem, SystemParamWithLifetime, With,
};
use modor_graphics::{
    instance_group_2d, window_target, InstanceData, MaterialSource, Shader, Texture,
    WINDOW_CAMERA_2D,
};
use modor_math::Vec2;
use modor_physics::Transform2D;
use modor_resources::ResKey;

const BLUR_SHADER: ResKey<Shader> = ResKey::new("blur");
const TEXTURE: ResKey<Texture> = ResKey::new("sprite");

pub fn main() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(window_target())
        .with_entity(Shader::from_path::<BlurInstanceData>(
            BLUR_SHADER,
            "blur.wgsl",
            false,
        ))
        .with_entity(Texture::from_path(TEXTURE, "smiley.png"))
        .with_entity(sprite_group())
        .with_entity(sprite(Vec2::new(-0.25, 0.25), 0))
        .with_entity(sprite(Vec2::new(0.25, 0.25), 3))
        .with_entity(sprite(Vec2::new(-0.25, -0.25), 6))
        .with_entity(sprite(Vec2::new(0.25, -0.25), 9))
        .run(modor_graphics::runner);
}

fn sprite_group() -> impl BuiltEntity {
    instance_group_2d::<With<Blur>>(WINDOW_CAMERA_2D, BlurMaterial { blur_factor: 0.005 })
}

fn sprite(position: Vec2, sample_count: u32) -> impl BuiltEntity {
    EntityBuilder::new()
        .component(Transform2D::new())
        .with(|t| t.position = position)
        .with(|t| t.size = Vec2::ONE * 0.4)
        .component(Blur { sample_count })
}

#[derive(Component, NoSystem)]
struct Blur {
    sample_count: u32,
}

#[derive(Component, NoSystem)]
struct BlurMaterial {
    blur_factor: f32,
}

impl MaterialSource for BlurMaterial {
    type Data = BlurMaterialData;
    type InstanceData = BlurInstanceData;

    fn data(&self) -> Self::Data {
        BlurMaterialData {
            blur_factor: self.blur_factor,
            padding1: [0.],
            padding2: [0., 0.],
        }
    }

    fn texture_keys(&self) -> Vec<ResKey<Texture>> {
        vec![TEXTURE]
    }

    fn shader_key(&self) -> ResKey<Shader> {
        BLUR_SHADER
    }

    fn is_transparent(&self) -> bool {
        false
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct BlurMaterialData {
    blur_factor: f32,
    padding1: [f32; 1],
    padding2: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct BlurInstanceData {
    sample_count: u32,
}

impl InstanceData for BlurInstanceData {
    type Query = &'static Blur;
    type UpdateFilter = Changed<Blur>;

    fn data(item: <Self::Query as SystemParamWithLifetime<'_>>::Param) -> Self {
        Self {
            sample_count: item.sample_count,
        }
    }
}
