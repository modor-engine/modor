use modor_graphics::{Camera2D, RenderTarget};
use modor_physics::Transform2D;
use modor_resources::ResKey;

// TODO: support Pressable in texture targets rendered on a model
// TODO: support UI ordering (ZIndex) by taking into account texture targets

#[derive(Component, Debug)]
pub struct WidgetState {
    pub camera_keys: Vec<ResKey<Camera2D>>,
    is_pressed: bool,
    is_just_pressed: bool,
    is_just_released: bool,
    is_hovered: bool,
    is_just_hovered: bool,
    is_just_left: bool,
}

#[systems]
impl WidgetState {
    pub fn new(camera_key: ResKey<Camera2D>) -> Self {
        Self {
            camera_keys: vec![camera_key],
            is_pressed: false,
            is_just_pressed: false,
            is_just_released: false,
            is_hovered: false,
            is_just_hovered: false,
            is_just_left: false,
        }
    }

    // TODO: require also a collider
    #[run]
    fn update(&mut self, transform: &Transform2D) {}
}

// To put on models rendering a texture render target
#[non_exhaustive]
#[derive(Component, NoSystem, Debug, Default)]
pub struct InnerWidgetStateTracking;

/*
Vec<ResKey<Camera2D>>
=> Camera2D
=> Vec<ResKey<RenderTarget>>

Query<InnerWidgetStateTracking + Model + Transform>
=> filter using Vec<ResKey<Camera2D>> from Model

===> maybe some issues, need to check
 */
