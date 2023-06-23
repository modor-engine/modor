use modor::{App, BuiltEntity, EntityBuilder, With};
use modor_graphics::testing::{has_component_diff, is_same};
use modor_graphics::{
    Camera2D, Color, Material, Model, RenderTarget, Size, Texture, TextureBuffer, ZIndex2D,
};
use modor_math::Vec2;
use modor_physics::Transform2D;

#[modor_test(disabled(macos, android, wasm))]
fn create_for_opaque() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(resources())
        .with_entity(rectangle(-0.09, 0, MaterialKey::OpaqueBlue))
        .with_entity(rectangle(0.03, u16::MAX - 1, MaterialKey::OpaqueBlue))
        .with_entity(rectangle(-0.03, 1, MaterialKey::OpaqueGreen))
        .with_entity(rectangle(0.09, u16::MAX, MaterialKey::OpaqueGreen).with(Marker))
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("z_index#opaque"))
        .with_update::<(), _>(|i: &mut ZIndex2D| *i = ZIndex2D::from(u16::MAX - u16::from(*i)))
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("z_index#opaque_reversed"))
        .with_deleted_components::<With<Marker>, ZIndex2D>()
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("z_index#opaque_reversed"));
}

#[modor_test(disabled(macos, android, wasm))]
fn create_for_transparent() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(resources())
        .with_entity(rectangle(-0.09, 0, MaterialKey::TransparentBlue))
        .with_entity(rectangle(0.03, u16::MAX - 1, MaterialKey::TransparentBlue))
        .with_entity(rectangle(-0.03, 1, MaterialKey::TransparentGreen))
        .with_entity(rectangle(0.09, u16::MAX, MaterialKey::TransparentGreen).with(Marker))
        .updated()
        .assert::<With<TextureBuffer>>(1, has_component_diff("z_index#transparent", 1))
        .with_update::<(), _>(|i: &mut ZIndex2D| *i = ZIndex2D::from(u16::MAX - u16::from(*i)))
        .updated()
        .assert::<With<TextureBuffer>>(1, has_component_diff("z_index#transparent_reversed", 1))
        .with_deleted_components::<With<Marker>, ZIndex2D>()
        .updated()
        .assert::<With<TextureBuffer>>(1, has_component_diff("z_index#transparent_reversed", 1));
}

#[modor_test(disabled(macos, android, wasm))]
fn create_for_opaque_and_transparent() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(resources())
        .with_entity(rectangle(-0.09, 0, MaterialKey::OpaqueBlue))
        .with_entity(rectangle(0.03, u16::MAX - 1, MaterialKey::OpaqueBlue))
        .with_entity(rectangle(-0.03, 1, MaterialKey::TransparentGreen))
        .with_entity(rectangle(0.09, u16::MAX, MaterialKey::TransparentGreen).with(Marker))
        .updated()
        .assert::<With<TextureBuffer>>(1, has_component_diff("z_index#transparent_mix", 1))
        .with_update::<(), _>(|i: &mut ZIndex2D| *i = ZIndex2D::from(u16::MAX - u16::from(*i)))
        .updated()
        .assert::<With<TextureBuffer>>(1, has_component_diff("z_index#transparent_mix_reversed", 1))
        .with_deleted_components::<With<Marker>, ZIndex2D>()
        .updated()
        .assert::<With<TextureBuffer>>(
            1,
            has_component_diff("z_index#transparent_mix_reversed", 1),
        );
}

fn resources() -> impl BuiltEntity {
    EntityBuilder::new()
        .with_child(target())
        .with_child(Camera2D::new(CameraKey).with_target_key(TargetKey))
        .with_child(Material::new(MaterialKey::OpaqueBlue).with_color(Color::BLUE))
        .with_child(Material::new(MaterialKey::OpaqueGreen).with_color(Color::GREEN))
        .with_child(
            Material::new(MaterialKey::TransparentBlue).with_color(Color::BLUE.with_alpha(0.5)),
        )
        .with_child(
            Material::new(MaterialKey::TransparentGreen).with_color(Color::GREEN.with_alpha(0.5)),
        )
}

fn target() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(RenderTarget::new(TargetKey))
        .with(Texture::from_size(TargetTextureKey, Size::new(30, 20)))
        .with(TextureBuffer::default())
}

fn rectangle(position: f32, z_index: u16, material: MaterialKey) -> impl BuiltEntity {
    EntityBuilder::new()
        .with(
            Transform2D::new()
                .with_position(Vec2::new(position, position))
                .with_size(Vec2::ONE * 0.3),
        )
        .with(ZIndex2D::from(z_index))
        .with(Model::rectangle(material).with_camera_key(CameraKey))
}

#[derive(Component, NoSystem)]
struct Marker;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TargetKey;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TargetTextureKey;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum MaterialKey {
    OpaqueBlue,
    OpaqueGreen,
    TransparentBlue,
    TransparentGreen,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CameraKey;
