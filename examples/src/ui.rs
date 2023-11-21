use modor::{systems, App, BuiltEntity, NoSystem, Single, SingleRef, SingletonComponent};
use modor_graphics::{
    model_2d, texture_target, window_target, Camera2D, Color, Material, Model2DMaterial,
    RenderTarget, Size, Window, ZIndex2D, TARGET_TEXTURES, TEXTURE_CAMERAS_2D, WINDOW_CAMERA_2D,
};
use modor_input::Mouse;
use modor_math::Vec2;
use modor_physics::Transform2D;
use modor_resources::ResKey;
use std::f32::consts::FRAC_PI_8;


// TODO: add notice in main README that this project is made for fun

pub fn main() {
    App::new()
        .with_entity(modor_ui::module())
        .with_entity(window_target())
        .with_entity(window_cursor())
        .with_entity(inner_target(0, Color::DARK_GRAY).component(InnerTarget))
        .with_entity(inner_scene(0, WINDOW_CAMERA_2D).component(InnerScene))
        .with_entity(inner_cursor(0, Color::GREEN).component(InnerCursor::default()))
        .with_entity(inner_target(1, Color::GRAY).component(InnerInnerTarget))
        .with_entity(inner_scene(1, TEXTURE_CAMERAS_2D.get(0)).component(InnerInnerScene))
        .with_entity(inner_cursor(1, Color::BLUE).component(InnerInnerCursor::default()))
        .run(modor_graphics::runner);
}

fn window_cursor() -> impl BuiltEntity {
    model_2d(WINDOW_CAMERA_2D, Model2DMaterial::Ellipse)
        .updated(|t: &mut Transform2D| t.size = Vec2::ONE * 0.05)
        .updated(|t: &mut Material| t.color = Color::RED)
        .component(ZIndex2D::from(1))
        .component(WindowCursor)
}

fn inner_target(id: usize, background_color: Color) -> impl BuiltEntity {
    texture_target(id, Size::new(800, 800), false)
        .updated(|t: &mut RenderTarget| t.background_color = background_color)
        .component(Transform2D::new())
        .with(|t| t.position = Vec2::new(0., 0.1))
}

fn inner_scene(target_id: usize, camera_key: ResKey<Camera2D>) -> impl BuiltEntity {
    model_2d(camera_key, Model2DMaterial::Rectangle)
        .updated(|t: &mut Transform2D| t.position = Vec2::new(0.1, -0.1))
        .updated(|t: &mut Transform2D| t.size = Vec2::ONE * 0.5)
        .updated(|t: &mut Transform2D| t.rotation = FRAC_PI_8)
        .updated(|t: &mut Material| t.texture_key = Some(TARGET_TEXTURES.get(target_id)))
        .component(ZIndex2D::from(2))
}

fn inner_cursor(target_id: usize, color: Color) -> impl BuiltEntity {
    model_2d(TEXTURE_CAMERAS_2D.get(target_id), Model2DMaterial::Ellipse)
        .updated(|t: &mut Transform2D| t.size = Vec2::ONE * 0.05)
        .updated(|t: &mut Material| t.color = color)
        .component(ZIndex2D::from(1))
}

#[derive(SingletonComponent)]
struct WindowCursor;

#[systems]
impl WindowCursor {
    #[run]
    fn update(
        transform: &mut Transform2D,
        window: Single<'_, Window, (&Window, &Camera2D)>,
        mouse: SingleRef<'_, '_, Mouse>,
    ) {
        let (window, camera) = window.get();
        let mouse = mouse.get();
        transform.position = camera.world_position(window.size(), mouse.position);
    }
}

#[derive(SingletonComponent, NoSystem)]
struct InnerTarget;

#[derive(SingletonComponent, NoSystem)]
struct InnerScene;

#[derive(SingletonComponent, Default)]
struct InnerCursor {
    position: Vec2,
}

// TODO: take into account camera Transform2D
#[systems]
impl InnerCursor {
    #[run]
    fn update(
        &mut self,
        window: Single<'_, Window, (&Window, &Camera2D)>,
        mouse: SingleRef<'_, '_, Mouse>,
        inner_scene_transform: Single<'_, InnerScene, &Transform2D>,
    ) {
        let (window, camera) = window.get();
        let mouse = mouse.get();
        let scene_transform = inner_scene_transform.get();
        self.position = fix_cursor_position(
            camera.world_position(window.size(), mouse.position),
            scene_transform,
        );
    }

    #[run_after_previous]
    fn update_transform(&self, transform: &mut Transform2D) {
        transform.position = self.position;
    }
}

#[derive(SingletonComponent, NoSystem)]
struct InnerInnerTarget;

#[derive(SingletonComponent, NoSystem)]
struct InnerInnerScene;

#[derive(SingletonComponent, Default)]
struct InnerInnerCursor {
    position: Vec2,
}

#[systems]
impl InnerInnerCursor {
    #[run]
    fn update(
        &mut self,
        window: Single<'_, Window, (&Window, &Camera2D)>,
        mouse: SingleRef<'_, '_, Mouse>,
        inner_scene_transform: Single<'_, InnerScene, &Transform2D>,
        inner_inner_scene_transform: Single<'_, InnerScene, &Transform2D>,
    ) {
        let (window, camera) = window.get();
        let mouse = mouse.get();
        let inner_scene_transform = inner_scene_transform.get();
        let inner_inner_scene_transform = inner_inner_scene_transform.get();
        self.position = fix_cursor_position(
            fix_cursor_position(
                camera.world_position(window.size(), mouse.position),
                inner_inner_scene_transform,
            ),
            inner_scene_transform,
        );
    }

    #[run_after_previous]
    fn update_transform(&self, transform: &mut Transform2D) {
        transform.position = self.position;
    }
}

fn fix_cursor_position(position: Vec2, transform: &Transform2D) -> Vec2 {
    (position - transform.position)
        .with_scale(Vec2::new(1. / transform.size.x, 1. / transform.size.y))
        .with_rotation(-transform.rotation)
}
