#![allow(clippy::cast_precision_loss, clippy::print_stdout, missing_docs)]

use modor::{entity, singleton, App, Built, Entity, EntityBuilder, Query, Single, World};
use modor_graphics::{Camera2D, Color, GraphicsModule, Mesh2D, SurfaceSize, WindowSettings};
use modor_input::{
    Finger, Gamepad, GamepadButton, GamepadStick, Key, Keyboard, Mouse, MouseButton,
};
use modor_math::Vec2;
use modor_physics::{Dynamics2D, Transform2D};
use std::io;
use std::io::Write;

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
pub fn main() {
    App::new()
        .with_entity(GraphicsModule::build(
            WindowSettings::default()
                .size(SurfaceSize::new(800, 600))
                .title("Modor - input")
                .has_visible_cursor(false),
        ))
        .with_entity(CustomCamera::build())
        .with_entity(MouseState::build())
        .with_entity(KeyboardState::build())
        .with_entity(TouchState::build())
        .with_entity(GamepadsState::build())
        .run(modor_graphics::runner);
}

struct CustomCamera;

#[singleton]
impl CustomCamera {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self)
            .inherit_from(Camera2D::build_rotated(
                Vec2::new(0.5, 0.5),
                Vec2::new(1.5, 1.5),
                20_f32.to_radians(),
            ))
            .with(Dynamics2D::new())
    }

    #[run]
    fn update(dynamics: &mut Dynamics2D, keyboard: Single<'_, Keyboard>) {
        let direction = keyboard.direction(Key::Numpad4, Key::Numpad6, Key::Numpad8, Key::Numpad2);
        *dynamics.velocity = direction;
    }
}

struct MouseState;

#[singleton]
impl MouseState {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(Transform2D::new().with_size(Vec2::ONE * 0.25))
            .with(Mesh2D::rectangle().with_color(Color::DARK_GRAY))
    }

    #[run]
    fn update_position(transform: &mut Transform2D, camera: Single<'_, Camera2D>) {
        *transform.position = camera.mouse_position();
    }

    #[run]
    fn update_color(mesh: &mut Mesh2D, mouse: Single<'_, Mouse>) {
        mesh.color.r += mouse.scroll_delta_in_lines(30., 30.).x / 50.;
        mesh.color.g += mouse.scroll_delta_in_lines(30., 30.).y / 50.;
        if mouse.button(MouseButton::Left).is_pressed {
            mesh.color = Color::BLUE;
        } else if mouse.button(MouseButton::Right).is_pressed {
            mesh.color = Color::DARK_GREEN;
        } else if mouse.button(MouseButton::Middle).is_pressed {
            mesh.color = Color::RED;
        }
    }
}

struct KeyboardState;

#[singleton]
impl KeyboardState {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(Transform2D::new().with_size(Vec2::ONE * 0.25))
            .with(Dynamics2D::new())
            .with(Mesh2D::rectangle().with_color(Color::DARK_GRAY))
    }

    #[run]
    fn update(dynamics: &mut Dynamics2D, mesh: &mut Mesh2D, keyboard: Single<'_, Keyboard>) {
        let direction = keyboard.direction(Key::Left, Key::Right, Key::Up, Key::Down);
        *dynamics.velocity = direction * 3.;
        mesh.color = if dynamics.velocity.magnitude() > 0. {
            Color::RED
        } else {
            Color::DARK_GRAY
        };
    }

    #[run]
    fn print_entered_text(keyboard: Single<'_, Keyboard>) {
        if !keyboard.text().is_empty() {
            print!("{}", keyboard.text());
            io::stdout()
                .flush()
                .expect("error when displaying text in terminal");
        }
        if keyboard.key(Key::Return).is_just_released {
            println!();
        }
    }
}

struct TouchState;

#[entity]
impl TouchState {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self)
    }

    #[run]
    fn create_fingers(
        entity: Entity<'_>,
        finger_states: Query<'_, &FingerState>,
        fingers: Query<'_, &Finger>,
        mut world: World<'_>,
    ) {
        for finger in fingers.iter() {
            if !finger_states.iter().any(|s| s.id == finger.id()) {
                world.create_child_entity(entity.id(), FingerState::build(finger.id()));
            }
        }
    }
}

struct FingerState {
    id: u64,
}

#[entity]
impl FingerState {
    fn build(id: u64) -> impl Built<Self> {
        EntityBuilder::new(Self { id })
            .with(
                Transform2D::new()
                    .with_position(Vec2::new(0.5, 0.5))
                    .with_size(Vec2::ONE * 0.25),
            )
            .with(Mesh2D::rectangle().with_color(Color::DARK_GRAY))
    }

    #[run]
    fn update(
        &self,
        transform: &mut Transform2D,
        fingers: Query<'_, &Finger>,
        camera: Single<'_, Camera2D>,
    ) {
        if let Some(finger) = fingers.iter().find(|f| f.id() == self.id) {
            if let Some(finger_position) = camera.finger_position(finger.id()) {
                *transform.position = finger_position;
            }
        }
    }

    #[run]
    fn delete(&self, entity: Entity<'_>, fingers: Query<'_, &Finger>, mut world: World<'_>) {
        if !fingers.iter().any(|f| f.id() == self.id) {
            world.delete_entity(entity.id());
        }
    }
}

struct GamepadsState;

#[entity]
impl GamepadsState {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self)
    }

    #[run]
    fn create_fingers(
        entity: Entity<'_>,
        gamepad_states: Query<'_, &GamepadState>,
        gamepads: Query<'_, &Gamepad>,
        mut world: World<'_>,
    ) {
        for gamepad in gamepads.iter() {
            if !gamepad_states.iter().any(|s| s.id == gamepad.id()) {
                world.create_child_entity(entity.id(), GamepadState::build(gamepad.id()));
            }
        }
    }
}

struct GamepadState {
    id: u64,
}

#[entity]
impl GamepadState {
    fn build(id: u64) -> impl Built<Self> {
        EntityBuilder::new(Self { id })
            .with(
                Transform2D::new()
                    .with_position(Vec2::new(0.5, 0.5))
                    .with_size(Vec2::ONE * 0.25),
            )
            .with(Dynamics2D::new())
            .with(Mesh2D::rectangle().with_color(Color::MAROON))
    }

    #[run]
    fn update(&self, mesh: &mut Mesh2D, dynamics: &mut Dynamics2D, gamepads: Query<'_, &Gamepad>) {
        if let Some(gamepad) = gamepads.iter().find(|f| f.id() == self.id) {
            let red = 1. - gamepad.button(GamepadButton::BackLeftTrigger).value();
            let green = 1. - gamepad.button(GamepadButton::BackRightTrigger).value();
            let blue = gamepad
                .button(GamepadButton::South)
                .state()
                .is_pressed
                .then(|| 0.)
                .unwrap_or(1.);
            mesh.color = Color::rgb(red, green, blue);
            let velocity1 = gamepad.stick_direction(GamepadStick::LeftStick);
            let velocity2 = gamepad.stick_direction(GamepadStick::RightStick);
            let velocity3 = gamepad.stick_direction(GamepadStick::DPad);
            let velocity4 = Vec2::new(gamepad.left_z_axis_value(), gamepad.right_z_axis_value());
            *dynamics.velocity = velocity1 + velocity2 + velocity3 + velocity4;
        }
    }

    #[run]
    fn delete(&self, entity: Entity<'_>, gamepads: Query<'_, &Gamepad>, mut world: World<'_>) {
        if !gamepads.iter().any(|f| f.id() == self.id) {
            world.delete_entity(entity.id());
        }
    }
}
