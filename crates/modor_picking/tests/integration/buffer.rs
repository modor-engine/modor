use crate::EntityPicker;
use modor::{App, With};
use modor_graphics::{
    instance_2d, texture_target, Camera2D, Default2DMaterial, Pixel, Size, TEXTURE_TARGETS,
};
use modor_resources::ResKey;

#[modor_test(disabled(macos, android, wasm))]
fn create_with_updated_camera() {
    let target_key = TEXTURE_TARGETS.get(0);
    App::new()
        .with_entity(instance_2d(CAMERA, Default2DMaterial::new()))
        .with_entity(modor_picking::module())
        .with_entity(texture_target(0, Size::new(30, 20), false))
        .with_entity(EntityPicker::new(Pixel::new(15, 10), target_key))
        .updated()
        .updated()
        .assert::<With<EntityPicker>>(1, has_picked_entity(None))
        .with_entity(Camera2D::new(CAMERA, target_key))
        .updated()
        .updated()
        .assert::<With<EntityPicker>>(1, has_picked_entity(Some(0)));
}

assertion_functions!(
    fn has_picked_entity(picker: &EntityPicker, entity_id: Option<usize>) {
        assert_eq!(picker.entity_id, entity_id);
    }
);

const CAMERA: ResKey<Camera2D> = ResKey::new("main");
