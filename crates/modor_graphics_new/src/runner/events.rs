use crate::runner::frame_rate;
use crate::runner::inputs;
use crate::runner::inputs::Gamepads;
use crate::settings::frame_rate::FrameRate;
use crate::settings::frame_rate::FrameRateLimit;
use crate::settings::rendering::Resolution;
use crate::targets::texture::TextureTarget;
use crate::targets::window::WindowTarget;
use instant::Instant;
use modor::App;
use modor_input::{
    GamepadEvent, InputEvent, InputEventCollector, KeyboardEvent, MouseEvent, MouseScrollUnit,
    TouchEvent,
};
use modor_math::Vec2;
use modor_physics::DeltaTime;
use std::time::Duration;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{
    DeviceEvent, ElementState, Event, KeyboardInput, MouseButton, MouseScrollDelta, Touch,
    TouchPhase, WindowEvent,
};

pub(super) struct AppState {
    app: App,
    gamepads: Gamepads,
    previous_update_end: Instant,
    is_suspended: bool,
}

impl AppState {
    pub(super) fn new(app: App) -> Self {
        Self {
            app,
            gamepads: Gamepads::new(),
            previous_update_end: Instant::now(),
            is_suspended: false,
        }
    }

    pub(super) fn init(&mut self) {
        self.app.update_singleton(|_: &mut TextureTarget| {
            panic!("graphics runner cannot be used in windowless mode");
        });
        for gamepad_id in self.gamepads.plugged_gamepads_ids().collect::<Vec<_>>() {
            self.send_gamepad_event(GamepadEvent::Plugged(gamepad_id));
        }
    }

    pub(super) fn invalidate_surface(&mut self) {
        self.app.update_singleton(WindowTarget::invalidate_surface);
    }

    pub(super) fn gamepad_events(&mut self) -> Vec<gilrs::ev::Event> {
        let mut events = Vec::new();
        while let Some(event) = self.gamepads.next_event() {
            events.push(event);
        }
        events
    }

    pub(super) fn update(&mut self) {
        frame_rate::run_with_frame_rate(self.previous_update_end, self.frame_rate(), || {
            self.app.update();
        });
        let update_end = Instant::now();
        let delta_time = if self.is_suspended {
            self.is_suspended = false;
            Duration::ZERO
        } else {
            update_end - self.previous_update_end
        };
        self.app
            .update_singleton(|t: &mut DeltaTime| t.set(delta_time));
        self.previous_update_end = update_end;
    }

    pub(super) fn treat_window_event(&mut self, event: Event<'_, ()>) {
        match event {
            Event::Suspended => self.is_suspended = true,
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => self.send_mouse_motion(delta),
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(size)
                | WindowEvent::ScaleFactorChanged {
                    new_inner_size: &mut size,
                    ..
                } => self.update_resolution(size),
                WindowEvent::MouseInput { button, state, .. } => {
                    self.send_mouse_button(button, state);
                }
                WindowEvent::MouseWheel { delta, .. } => self.send_mouse_wheel(delta),
                WindowEvent::CursorMoved { position, .. } => self.send_mouse_position(position),
                WindowEvent::KeyboardInput { input, .. } => self.send_keyboard_key(input),
                WindowEvent::ReceivedCharacter(character) => self.send_character(character),
                WindowEvent::Touch(touch) => match touch.phase {
                    TouchPhase::Started => self.send_started_touch(touch),
                    TouchPhase::Moved => self.send_moved_touch(touch),
                    TouchPhase::Ended | TouchPhase::Cancelled => self.send_ended_touch(touch),
                },
                WindowEvent::Moved(_)
                | WindowEvent::CloseRequested
                | WindowEvent::Destroyed
                | WindowEvent::DroppedFile(_)
                | WindowEvent::HoveredFile(_)
                | WindowEvent::HoveredFileCancelled
                | WindowEvent::Focused(_)
                | WindowEvent::ModifiersChanged(_)
                | WindowEvent::Ime(_)
                | WindowEvent::CursorEntered { .. }
                | WindowEvent::CursorLeft { .. }
                | WindowEvent::TouchpadPressure { .. }
                | WindowEvent::AxisMotion { .. }
                | WindowEvent::ThemeChanged(_)
                | WindowEvent::Occluded(_) => (),
            },
            Event::NewEvents(_)
            | Event::DeviceEvent { .. }
            | Event::UserEvent(_)
            | Event::Resumed
            | Event::MainEventsCleared
            | Event::RedrawRequested(_)
            | Event::RedrawEventsCleared
            | Event::LoopDestroyed => (),
        }
    }

    pub(super) fn treat_gamepad_event(&mut self, event: gilrs::ev::Event) {
        let gilrs::ev::Event { id, event, .. } = event;
        let id = <_ as Into<usize>>::into(id) as u64;
        match event {
            gilrs::EventType::Connected => self.send_gamepad_event(GamepadEvent::Plugged(id)),
            gilrs::EventType::Disconnected => self.send_gamepad_event(GamepadEvent::Unplugged(id)),
            gilrs::EventType::ButtonPressed(button, _) => {
                self.send_pressed_gamepad_button(id, button);
            }
            gilrs::EventType::ButtonReleased(button, _) => {
                self.send_released_gamepad_button(id, button);
            }
            gilrs::EventType::ButtonChanged(button, value, _) => {
                self.send_changed_gamepad_button(id, button, value);
            }
            gilrs::EventType::AxisChanged(axis, value, _) => {
                self.send_changed_gamepad_axis(id, axis, value);
            }
            gilrs::EventType::Dropped | gilrs::EventType::ButtonRepeated(_, _) => {}
        }
    }

    fn frame_rate(&mut self) -> FrameRateLimit {
        let mut frame_rate = FrameRateLimit::VSync;
        self.app
            .update_singleton(|f: &mut FrameRate| frame_rate = f.limit);
        frame_rate
    }

    fn update_resolution(&mut self, size: PhysicalSize<u32>) {
        let PhysicalSize { width, height } = size;
        self.app
            .update_singleton(|r: &mut Resolution| *r = Resolution { width, height });
    }

    #[allow(clippy::cast_possible_truncation)]
    fn send_mouse_motion(&mut self, winit_delta: (f64, f64)) {
        let delta = Vec2::new(winit_delta.0 as f32, -winit_delta.1 as f32);
        self.send_mouse_event(MouseEvent::Moved(delta));
    }

    fn send_mouse_button(&mut self, button: MouseButton, state: ElementState) {
        let button = inputs::convert_mouse_button(button);
        let event = match state {
            ElementState::Pressed => MouseEvent::PressedButton(button),
            ElementState::Released => MouseEvent::ReleasedButton(button),
        };
        self.send_mouse_event(event);
    }

    fn send_mouse_wheel(&mut self, delta: MouseScrollDelta) {
        let event = match delta {
            MouseScrollDelta::LineDelta(columns, rows) => {
                let delta = Vec2::new(columns, -rows);
                MouseEvent::Scroll(delta, MouseScrollUnit::Line)
            }
            MouseScrollDelta::PixelDelta(delta) => {
                let delta = Self::winit_pos_to_vec2(delta);
                MouseEvent::Scroll(Vec2::new(delta.x, -delta.y), MouseScrollUnit::Pixel)
            }
        };
        self.send_mouse_event(event);
    }

    fn send_mouse_position(&mut self, position: PhysicalPosition<f64>) {
        let position = Self::winit_pos_to_vec2(position);
        self.send_mouse_event(MouseEvent::UpdatedPosition(position));
    }

    fn send_keyboard_key(&mut self, input: KeyboardInput) {
        if let Some(code) = input.virtual_keycode {
            let key = inputs::convert_keyboard_key(code);
            let event = match input.state {
                ElementState::Pressed => KeyboardEvent::PressedKey(key),
                ElementState::Released => KeyboardEvent::ReleasedKey(key),
            };
            self.send_keyboard_event(event);
        }
    }

    fn send_character(&mut self, character: char) {
        self.send_keyboard_event(KeyboardEvent::EnteredText(character.into()));
    }

    fn send_started_touch(&mut self, touch: Touch) {
        self.send_touch_event(TouchEvent::Started(touch.id));
        self.send_touch_event(TouchEvent::UpdatedPosition(
            touch.id,
            Self::winit_pos_to_vec2(touch.location),
        ));
    }

    fn send_moved_touch(&mut self, touch: Touch) {
        self.send_touch_event(TouchEvent::UpdatedPosition(
            touch.id,
            Self::winit_pos_to_vec2(touch.location),
        ));
    }

    fn send_ended_touch(&mut self, touch: Touch) {
        self.send_touch_event(TouchEvent::Ended(touch.id));
    }

    fn send_pressed_gamepad_button(&mut self, gamepad_id: u64, button: gilrs::Button) {
        if let Some(button) = inputs::convert_gamepad_button(button) {
            self.send_gamepad_event(GamepadEvent::PressedButton(gamepad_id, button));
        }
    }

    fn send_released_gamepad_button(&mut self, gamepad_id: u64, button: gilrs::Button) {
        if let Some(button) = inputs::convert_gamepad_button(button) {
            self.send_gamepad_event(GamepadEvent::ReleasedButton(gamepad_id, button));
        }
    }

    fn send_changed_gamepad_button(&mut self, gamepad_id: u64, button: gilrs::Button, value: f32) {
        if let Some(button) = inputs::convert_gamepad_button(button) {
            self.send_gamepad_event(GamepadEvent::UpdatedButtonValue(gamepad_id, button, value));
        }
    }

    fn send_changed_gamepad_axis(&mut self, gamepad_id: u64, axis: gilrs::Axis, value: f32) {
        if let Some(axis) = inputs::convert_gamepad_axis(axis) {
            self.send_gamepad_event(GamepadEvent::UpdatedAxisValue(gamepad_id, axis, value));
        }
    }

    fn send_keyboard_event(&mut self, event: KeyboardEvent) {
        self.send_event(InputEvent::Keyboard(event));
    }

    fn send_mouse_event(&mut self, event: MouseEvent) {
        self.send_event(InputEvent::Mouse(event));
    }

    fn send_touch_event(&mut self, event: TouchEvent) {
        self.send_event(InputEvent::Touch(event));
    }

    fn send_gamepad_event(&mut self, event: GamepadEvent) {
        self.send_event(InputEvent::Gamepad(event));
    }

    fn send_event(&mut self, event: InputEvent) {
        self.app
            .update_singleton(|e: &mut InputEventCollector| e.push(event));
    }

    #[allow(clippy::cast_possible_truncation)]
    fn winit_pos_to_vec2(position: PhysicalPosition<f64>) -> Vec2 {
        Vec2::new(position.x as f32, position.y as f32)
    }
}
