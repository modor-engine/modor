use modor::{App, BuiltEntity, Component, NoSystem};
use modor_graphics::{
    instance_2d, window_target, MaterialSource, Shader, Texture, WINDOW_CAMERA_2D,
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
        .with_entity(Shader::from_path(BLUR_SHADER, "blur.wgsl"))
        .with_entity(Texture::from_path(TEXTURE, "smiley.png"))
        .with_entity(sprite(Vec2::new(-0.25, 0.25), 0))
        .with_entity(sprite(Vec2::new(0.25, 0.25), 3))
        .with_entity(sprite(Vec2::new(-0.25, -0.25), 6))
        .with_entity(sprite(Vec2::new(0.25, -0.25), 9))
        .run(modor_graphics::runner);
}

fn sprite(position: Vec2, sample_count: u32) -> impl BuiltEntity {
    let material = BlurMaterial {
        blur_factor: 0.005,
        sample_count,
    };
    instance_2d(WINDOW_CAMERA_2D, material)
        .updated(|t: &mut Transform2D| t.position = position)
        .updated(|t: &mut Transform2D| t.size = Vec2::ONE * 0.4)
}

#[derive(Component, NoSystem)]
struct BlurMaterial {
    blur_factor: f32,
    sample_count: u32,
}

impl MaterialSource for BlurMaterial {
    type Data = BlurMaterialData;

    fn data(&self) -> Self::Data {
        BlurMaterialData {
            blur_factor: self.blur_factor,
            sample_count: self.sample_count,
            padding: [0., 0.],
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
    sample_count: u32,
    padding: [f32; 2],
}
