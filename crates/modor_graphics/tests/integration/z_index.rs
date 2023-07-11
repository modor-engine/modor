use modor::{App, BuiltEntity, EntityBuilder, With};
use modor_graphics::testing::{has_component_diff, is_same};
use modor_graphics::{
    Camera2D, Color, Material, Model, RenderTarget, Size, Texture, TextureBuffer, ZIndex2D,
};
use modor_math::Vec2;
use modor_physics::Transform2D;
use modor_resources::ResKey;

#[modor_test(disabled(macos, android, wasm))]
fn create_for_opaque() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(resources())
        .with_entity(rectangle(-0.09, 0, OPAQUE_BLUE_MATERIAL))
        .with_entity(rectangle(0.03, u16::MAX - 1, OPAQUE_BLUE_MATERIAL))
        .with_entity(rectangle(-0.03, 1, OPAQUE_GREEN_MATERIAL))
        .with_entity(rectangle(0.09, u16::MAX, OPAQUE_GREEN_MATERIAL).component(Marker))
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
        .with_entity(rectangle(-0.09, 0, TRANSPARENT_BLUE_MATERIAL))
        .with_entity(rectangle(0.03, u16::MAX - 1, TRANSPARENT_BLUE_MATERIAL))
        .with_entity(rectangle(-0.03, 1, TRANSPARENT_GREEN_MATERIAL))
        .with_entity(rectangle(0.09, u16::MAX, TRANSPARENT_GREEN_MATERIAL).component(Marker))
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
        .with_entity(rectangle(-0.09, 0, OPAQUE_BLUE_MATERIAL))
        .with_entity(rectangle(0.03, u16::MAX - 1, OPAQUE_BLUE_MATERIAL))
        .with_entity(rectangle(-0.03, 1, TRANSPARENT_GREEN_MATERIAL))
        .with_entity(rectangle(0.09, u16::MAX, TRANSPARENT_GREEN_MATERIAL).component(Marker))
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
        .child_entity(target())
        .child_component(Material::new(OPAQUE_BLUE_MATERIAL))
        .with(|m| m.color = Color::BLUE)
        .child_component(Material::new(OPAQUE_GREEN_MATERIAL))
        .with(|m| m.color = Color::GREEN)
        .child_component(Material::new(TRANSPARENT_BLUE_MATERIAL))
        .with(|m| m.color = Color::BLUE.with_alpha(0.5))
        .child_component(Material::new(TRANSPARENT_GREEN_MATERIAL))
        .with(|m| m.color = Color::GREEN.with_alpha(0.5))
}

fn target() -> impl BuiltEntity {
    let target_key = ResKey::new("main");
    let texture_key = ResKey::new("target");
    EntityBuilder::new()
        .component(RenderTarget::new(target_key))
        .component(Texture::from_size(texture_key, Size::new(30, 20)))
        .component(TextureBuffer::default())
        .component(Camera2D::new(CAMERA, target_key))
}

fn rectangle(position: f32, z_index: u16, material_key: ResKey<Material>) -> impl BuiltEntity {
    EntityBuilder::new()
        .component(Transform2D::new())
        .with(|t| *t.position = Vec2::new(position, position))
        .with(|t| *t.size = Vec2::ONE * 0.3)
        .component(ZIndex2D::from(z_index))
        .component(Model::rectangle(material_key, CAMERA))
}

#[derive(Component, NoSystem)]
struct Marker;

const CAMERA: ResKey<Camera2D> = ResKey::new("main");
const OPAQUE_BLUE_MATERIAL: ResKey<Material> = ResKey::new("opaque-blue");
const OPAQUE_GREEN_MATERIAL: ResKey<Material> = ResKey::new("opaque-green");
const TRANSPARENT_BLUE_MATERIAL: ResKey<Material> = ResKey::new("transparent-blue");
const TRANSPARENT_GREEN_MATERIAL: ResKey<Material> = ResKey::new("transparent-green");
