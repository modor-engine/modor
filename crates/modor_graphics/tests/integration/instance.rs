use modor::{App, BuiltEntity, EntityBuilder, QueryEntityFilter, QueryFilter, With};
use modor_graphics::testing::{has_pixel_diff, is_same};
use modor_graphics::{
    instance_2d, instance_group_2d, texture_target, InstanceGroup2D, InstanceRendering2D, Material,
    MaterialType, Size, TextureBuffer, TEXTURE_CAMERAS_2D,
};
use modor_math::Vec2;
use modor_physics::Transform2D;
use modor_resources::ResKey;
use std::f32::consts::FRAC_PI_2;

#[modor_test(disabled(macos, android, wasm))]
fn create_from_self() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(texture_target(0, Size::new(30, 20), true))
        .with_entity(self_instance_group())
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("instance#empty"))
        .with_component::<With<InstanceGroup2D>, _>(Transform2D::new)
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("instance#self_default_transform"))
        .with_update::<With<InstanceGroup2D>, _>(|t: &mut Transform2D| {
            t.position = Vec2::new(0.25, 0.25);
            t.size = Vec2::new(0.5, 0.25);
            t.rotation = FRAC_PI_2;
        })
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("instance#self_other_transform"))
        .with_deleted_components::<With<InstanceGroup2D>, Transform2D>()
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("instance#empty"));
}

#[modor_test(disabled(macos, android, wasm))]
fn create_from_filter() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(texture_target(0, Size::new(30, 20), true))
        .with_entity(filtered_instance_group::<With<Displayed>>())
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("instance#empty"))
        .with_entity(
            instance(Vec2::new(0.25, 0.25))
                .component(Temporary)
                .component(Displayed),
        )
        .with_entity(instance(Vec2::new(-0.25, -0.25)))
        .with_entity(instance(Vec2::new(0.25, -0.25)).component(Displayed))
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("instance#filter_all_instances"))
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("instance#filter_removed_instance"));
}

#[modor_test(disabled(macos, android, wasm))]
fn create_rectangle_instance_2d() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(texture_target(0, Size::new(30, 20), true))
        .with_entity(instance_2d(
            TEXTURE_CAMERAS_2D.get(0),
            MaterialType::Rectangle,
        ))
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("instance#instance_2d_rectangle"));
}

#[modor_test(disabled(macos, android, wasm))]
fn create_ellipse_instance_2d() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(texture_target(0, Size::new(30, 20), true))
        .with_entity(instance_2d(
            TEXTURE_CAMERAS_2D.get(0),
            MaterialType::Ellipse,
        ))
        .updated()
        .assert::<With<TextureBuffer>>(1, has_pixel_diff("instance#instance_2d_ellipse", 10));
}

#[modor_test(disabled(macos, android, wasm))]
fn create_instance_2d_with_custom_material() {
    let material_key = ResKey::new("custom");
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(texture_target(0, Size::new(30, 20), true))
        .with_entity(Material::ellipse(material_key))
        .with_entity(instance_2d(
            TEXTURE_CAMERAS_2D.get(0),
            MaterialType::Key(material_key),
        ))
        .updated()
        .assert::<With<TextureBuffer>>(1, has_pixel_diff("instance#instance_2d_ellipse", 10));
}

#[modor_test(disabled(macos, android, wasm))]
fn create_rectangle_instance_group_2d() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(texture_target(0, Size::new(30, 20), true))
        .with_entity(instance_group_2d::<With<Displayed>>(
            TEXTURE_CAMERAS_2D.get(0),
            MaterialType::Rectangle,
        ))
        .with_entity(instance(Vec2::new(0.25, 0.25)).component(Displayed))
        .with_entity(instance(Vec2::new(-0.25, -0.25)))
        .with_entity(instance(Vec2::new(0.25, -0.25)).component(Displayed))
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("instance#filter_all_instances"));
}

#[modor_test(disabled(macos, android, wasm))]
fn create_ellipse_instance_group_2d() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(texture_target(0, Size::new(30, 20), true))
        .with_entity(instance_group_2d::<With<Displayed>>(
            TEXTURE_CAMERAS_2D.get(0),
            MaterialType::Ellipse,
        ))
        .with_entity(instance(Vec2::new(0.25, 0.25)).component(Displayed))
        .with_entity(instance(Vec2::new(-0.25, -0.25)))
        .with_entity(instance(Vec2::new(0.25, -0.25)).component(Displayed))
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("instance#filter_all_instances_ellipse"));
}

#[modor_test(disabled(macos, android, wasm))]
fn create_instance_group_2d_with_custom_material() {
    let material_key = ResKey::new("custom");
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(texture_target(0, Size::new(30, 20), true))
        .with_entity(Material::ellipse(material_key))
        .with_entity(instance_group_2d::<With<Displayed>>(
            TEXTURE_CAMERAS_2D.get(0),
            MaterialType::Key(material_key),
        ))
        .with_entity(instance(Vec2::new(0.25, 0.25)).component(Displayed))
        .with_entity(instance(Vec2::new(-0.25, -0.25)))
        .with_entity(instance(Vec2::new(0.25, -0.25)).component(Displayed))
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("instance#filter_all_instances_ellipse"));
}

fn self_instance_group() -> impl BuiltEntity {
    let group_key = ResKey::new("self-instance-group");
    let material_key = ResKey::new("self-instance-group");
    EntityBuilder::new()
        .component(InstanceGroup2D::from_self(group_key))
        .component(InstanceRendering2D::new(
            group_key,
            TEXTURE_CAMERAS_2D.get(0),
            material_key,
        ))
        .component(Material::new(material_key))
}

fn filtered_instance_group<F>() -> impl BuiltEntity
where
    F: QueryEntityFilter,
{
    let group_key = ResKey::new("self-instance-group");
    let material_key = ResKey::new("self-instance-group");
    let filter = QueryFilter::new::<F>();
    EntityBuilder::new()
        .component(InstanceGroup2D::from_filter(group_key, filter))
        .component(InstanceRendering2D::new(
            group_key,
            TEXTURE_CAMERAS_2D.get(0),
            material_key,
        ))
        .component(Material::new(material_key))
}

fn instance(position: Vec2) -> impl BuiltEntity {
    EntityBuilder::new()
        .component(Transform2D::new())
        .with(|t| t.position = position)
        .with(|t| t.size = Vec2::new(0.4, 0.3))
}

#[derive(Component, NoSystem)]
struct Displayed;

#[derive(Component, TemporaryComponent)]
struct Temporary;
