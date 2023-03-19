use crate::input::Gamepads;
use crate::{input, FrameRate, Window};
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
use winit::event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget};
use winit::window::WindowId;

pub fn runner(app: App) {
    let event_loop = EventLoop::new();
    let mut state = RunnerState::new(app);
    state.init();
    event_loop.run(move |event, event_loop, control_flow| {
        state.treat_event(event, event_loop, control_flow);
    });
}

struct RunnerState {
    app: App,
    gamepads: Gamepads,
    previous_update_end: Instant,
    is_suspended: bool,
    is_updated: bool,
}

impl RunnerState {
    fn new(app: App) -> Self {
        Self {
            app,
            gamepads: Gamepads::new(),
            previous_update_end: Instant::now(),
            is_suspended: false,
            is_updated: false,
        }
    }

    fn init(&mut self) {
        for gamepad_id in self.gamepads.plugged_gamepads_ids().collect::<Vec<_>>() {
            self.send_gamepad_event(GamepadEvent::Plugged(gamepad_id));
        }
    }

    fn treat_event(
        &mut self,
        event: Event<'_, ()>,
        event_loop: &EventLoopWindowTarget<()>,
        control_flow: &mut ControlFlow,
    ) {
        match event {
            Event::Suspended => self.is_suspended = true,
            Event::Resumed => self.invalidate_surface(),
            Event::MainEventsCleared => self.request_redraw(event_loop),
            Event::RedrawRequested(_) => self.update(),
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => self.send_mouse_motion(delta),
            Event::WindowEvent { event, window_id } => match event {
                WindowEvent::CloseRequested => self.close_window(window_id, control_flow),
                WindowEvent::Resized(size)
                | WindowEvent::ScaleFactorChanged {
                    new_inner_size: &mut size,
                    ..
                } => self.update_window_size(size, window_id),
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
            | Event::RedrawEventsCleared
            | Event::LoopDestroyed => (),
        }
    }

    fn gamepad_events(&mut self) -> Vec<gilrs::ev::Event> {
        let mut events = Vec::new();
        while let Some(event) = self.gamepads.next_event() {
            events.push(event);
        }
        events
    }

    fn invalidate_surface(&mut self) {
        self.app.update_components(|w: &mut Window| {
            w.is_surface_invalid = true;
        });
    }

    fn request_redraw(&mut self, event_loop: &EventLoopWindowTarget<()>) {
        self.app.update_components(|w: &mut Window| {
            w.request_redraw(event_loop);
        });
        self.is_updated = false;
    }

    fn update(&mut self) {
        if self.is_updated {
            return;
        }
        for event in self.gamepad_events() {
            self.treat_gamepad_event(event);
        }
        self.app.update_components(Window::update);
        self.app.update();
        self.frame_rate()
            .sleep(self.previous_update_end, self.window_frame_rate_mhz());
        let update_end = Instant::now();
        self.update_delta_time(update_end);
        self.previous_update_end = update_end;
        self.is_updated = true;
    }

    fn frame_rate(&mut self) -> FrameRate {
        let mut frame_rate = FrameRate::default();
        self.app.update_components(|r: &mut FrameRate| {
            frame_rate = *r;
        });
        frame_rate
    }

    fn window_frame_rate_mhz(&mut self) -> Option<u32> {
        let mut min_frame_rate: Option<u32> = None;
        self.app.update_components(|w: &mut Window| {
            if let Some(frame_rate) = w.min_frame_rate_mhz() {
                if let Some(min_frame_rate) = &mut min_frame_rate {
                    *min_frame_rate = frame_rate.min(*min_frame_rate);
                } else {
                    min_frame_rate = Some(frame_rate);
                }
            }
        });
        min_frame_rate
    }

    fn update_delta_time(&mut self, update_end: Instant) {
        let delta_time = if self.is_suspended {
            self.is_suspended = false;
            Duration::ZERO
        } else {
            update_end - self.previous_update_end
        };
        self.app.update_components(|t: &mut DeltaTime| {
            t.set(delta_time);
        });
    }

    fn close_window(&mut self, window_id: WindowId, control_flow: &mut ControlFlow) {
        self.app.update_components(|w: &mut Window| {
            w.close_window(window_id, control_flow);
        });
    }

    fn update_window_size(&mut self, size: PhysicalSize<u32>, window_id: WindowId) {
        let PhysicalSize { width, height } = size;
        self.app
            .update_components(|w: &mut Window| w.update_size(width, height, window_id));
    }

    #[allow(clippy::cast_possible_truncation)]
    fn send_mouse_motion(&mut self, winit_delta: (f64, f64)) {
        let delta = Vec2::new(winit_delta.0 as f32, -winit_delta.1 as f32);
        self.send_mouse_event(MouseEvent::Moved(delta));
    }

    fn send_mouse_button(&mut self, button: MouseButton, state: ElementState) {
        let button = input::convert_mouse_button(button);
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
            let key = input::convert_keyboard_key(code);
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

    fn treat_gamepad_event(&mut self, event: gilrs::ev::Event) {
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

    fn send_pressed_gamepad_button(&mut self, gamepad_id: u64, button: gilrs::Button) {
        if let Some(button) = input::convert_gamepad_button(button) {
            self.send_gamepad_event(GamepadEvent::PressedButton(gamepad_id, button));
        }
    }

    fn send_released_gamepad_button(&mut self, gamepad_id: u64, button: gilrs::Button) {
        if let Some(button) = input::convert_gamepad_button(button) {
            self.send_gamepad_event(GamepadEvent::ReleasedButton(gamepad_id, button));
        }
    }

    fn send_changed_gamepad_button(&mut self, gamepad_id: u64, button: gilrs::Button, value: f32) {
        if let Some(button) = input::convert_gamepad_button(button) {
            self.send_gamepad_event(GamepadEvent::UpdatedButtonValue(gamepad_id, button, value));
        }
    }

    fn send_changed_gamepad_axis(&mut self, gamepad_id: u64, axis: gilrs::Axis, value: f32) {
        if let Some(axis) = input::convert_gamepad_axis(axis) {
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
        self.app.update_components(|e: &mut InputEventCollector| {
            e.push(event.clone());
        });
    }

    #[allow(clippy::cast_possible_truncation)]
    fn winit_pos_to_vec2(position: PhysicalPosition<f64>) -> Vec2 {
        Vec2::new(position.x as f32, position.y as f32)
    }
}
