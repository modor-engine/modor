use crate::{utils, FrameRate, FrameRateLimit, SurfaceSize, Window, WindowInit};
use instant::Instant;
use modor::App;
use modor_input::{
    GamepadAxis, GamepadButton, GamepadEvent, InputDelta, InputEventCollector, Key, KeyboardEvent,
    MouseButton, MouseEvent, MouseScrollUnit, TouchEvent, WindowPosition,
};
use modor_physics::DeltaTime;
use winit::event;
use winit::event::{
    DeviceEvent, ElementState, Event, MouseScrollDelta, TouchPhase, VirtualKeyCode, WindowEvent,
};
use winit::event_loop::{ControlFlow, EventLoop};

// coverage: off (window cannot be tested)

/// Run application update for each frame rendered in a window.
///
/// [`DeltaTime`](modor_physics::DeltaTime) is automatically updated.<br>
/// Frame rate is limited depending on [`FrameRateLimit`](crate::FrameRateLimit).
///
/// Input events are automatically sent to the [`InputModule`](modor_input::InputModule).
///
/// This runner must be used instead of a call to [`App::update`](modor::App::update)
/// inside a loop to ensure a correct window update.
///
/// # Panics
///
/// This will panic if [`GraphicsModule`](crate::GraphicsModule) does not exist or has been created
/// in windowless mode.
///
/// # Platform-specific
///
/// - Web: a canvas with id `modor` is automatically added to the HTML body.
///
/// # Examples
///
/// ```rust
/// # use modor::App;
/// # use modor_graphics::{GraphicsModule, SurfaceSize, WindowSettings};
/// #
/// # fn no_run() {
/// App::new()
///      .with_entity(GraphicsModule::build(
///          WindowSettings::default()
///              .size(SurfaceSize::new(640, 480))
///              .title("title"),
///      ))
///     .run(modor_graphics::runner);
/// # }
/// ```
#[allow(clippy::wildcard_enum_match_arm, clippy::cast_possible_truncation)]
pub fn runner(mut app: App) {
    configure_logging();
    #[cfg(not(target_os = "android"))]
    let mut gilrs = gilrs::Gilrs::new().expect("cannot retrieve gamepad information");
    init_gamepads(&mut app, &gilrs);
    let event_loop = EventLoop::new();
    let mut window = None;
    app.run_for_singleton(|i: &mut WindowInit| window = Some(i.create_window(&event_loop)));
    let window = window.expect("`GraphicsModule` entity not found or created in windowless mode");
    let mut previous_update_end = Instant::now();
    event_loop.run(move |event, _, control_flow| match event {
        Event::Resumed => {
            app.run_for_singleton(|w: &mut WindowInit| w.create_renderer(&window));
            app.run_for_singleton(|w: &mut Window| w.update_renderer(&window));
        }
        Event::MainEventsCleared => window.request_redraw(),
        Event::RedrawRequested(window_id) if window_id == window.id() => {
            let mut frame_rate = FrameRate::Unlimited;
            app.run_for_singleton(|i: &mut FrameRateLimit| frame_rate = i.get());
            app.run_for_singleton(|w: &mut Window| {
                let size = window.inner_size();
                w.set_size(SurfaceSize {
                    width: size.width,
                    height: size.height,
                });
            });
            #[cfg(not(target_os = "android"))]
            update_gamepads(&mut app, &mut gilrs);
            utils::run_with_frame_rate(previous_update_end, frame_rate, || app.update());
            let update_end = Instant::now();
            app.run_for_singleton(|t: &mut DeltaTime| t.set(update_end - previous_update_end));
            previous_update_end = update_end;
        }
        Event::DeviceEvent {
            event: DeviceEvent::MouseMotion { delta },
            ..
        } => {
            let delta = InputDelta::xy(delta.0 as f32, -delta.1 as f32);
            send_mouse_event(&mut app, MouseEvent::Moved(delta));
        }
        Event::WindowEvent { event, window_id } if window_id == window.id() => {
            treat_window_event(&mut app, event, control_flow);
        }
        _ => {}
    });
}

#[allow(clippy::wildcard_enum_match_arm, clippy::cast_possible_truncation)]
fn treat_window_event(app: &mut App, event: WindowEvent<'_>, control_flow: &mut ControlFlow) {
    match event {
        WindowEvent::Resized(size)
        | WindowEvent::ScaleFactorChanged {
            new_inner_size: &mut size,
            ..
        } => {
            app.run_for_singleton(|w: &mut Window| {
                w.set_size(SurfaceSize {
                    width: size.width,
                    height: size.height,
                });
            });
        }
        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
        WindowEvent::MouseInput { button, state, .. } => {
            send_mouse_event(app, convert_mouse_button_event(button, state));
        }
        WindowEvent::MouseWheel { delta, .. } => {
            send_mouse_event(
                app,
                match delta {
                    MouseScrollDelta::LineDelta(columns, rows) => {
                        MouseEvent::Scroll(InputDelta::xy(columns, -rows), MouseScrollUnit::Line)
                    }
                    MouseScrollDelta::PixelDelta(delta) => MouseEvent::Scroll(
                        InputDelta::xy(delta.x as f32, -delta.y as f32),
                        MouseScrollUnit::Pixel,
                    ),
                },
            );
        }
        WindowEvent::CursorMoved { position, .. } => {
            send_mouse_event(
                app,
                MouseEvent::UpdatedPosition(WindowPosition::xy(
                    position.x as f32,
                    position.y as f32,
                )),
            );
        }
        WindowEvent::KeyboardInput { input, .. } => {
            if let Some(code) = input.virtual_keycode {
                send_keyboard_event(app, convert_keyboard_key_event(code, input.state));
            }
        }
        WindowEvent::ReceivedCharacter(character) => {
            send_keyboard_event(app, KeyboardEvent::EnteredText(character.into()));
        }
        WindowEvent::Touch(touch) => match touch.phase {
            TouchPhase::Started => {
                send_touch_event(app, TouchEvent::Started(touch.id));
                send_touch_event(
                    app,
                    TouchEvent::UpdatedPosition(
                        touch.id,
                        WindowPosition::xy(touch.location.x as f32, touch.location.y as f32),
                    ),
                );
            }
            TouchPhase::Moved => send_touch_event(
                app,
                TouchEvent::UpdatedPosition(
                    touch.id,
                    WindowPosition::xy(touch.location.x as f32, touch.location.y as f32),
                ),
            ),
            TouchPhase::Ended | TouchPhase::Cancelled => {
                send_touch_event(app, TouchEvent::Ended(touch.id));
            }
        },
        _ => {}
    }
}

fn configure_logging() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
    }
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init().expect("cannot initialize logger");
    }
}

fn send_keyboard_event(app: &mut App, event: KeyboardEvent) {
    app.run_for_singleton(|c: &mut InputEventCollector| c.push(event.into()));
}

fn send_mouse_event(app: &mut App, event: MouseEvent) {
    app.run_for_singleton(|c: &mut InputEventCollector| c.push(event.into()));
}

fn send_touch_event(app: &mut App, event: TouchEvent) {
    app.run_for_singleton(|c: &mut InputEventCollector| c.push(event.into()));
}

fn send_gamepad_event(app: &mut App, event: GamepadEvent) {
    app.run_for_singleton(|c: &mut InputEventCollector| c.push(event.into()));
}

#[cfg(not(target_os = "android"))]
fn init_gamepads(app: &mut App, gilrs: &gilrs::Gilrs) {
    for (gamepad_id, _) in gilrs.gamepads() {
        let gamepad_id = <_ as Into<usize>>::into(gamepad_id) as u64;
        send_gamepad_event(app, GamepadEvent::Plugged(gamepad_id));
    }
}

#[cfg(not(target_os = "android"))]
fn update_gamepads(app: &mut App, gilrs: &mut gilrs::Gilrs) {
    while let Some(gilrs::Event { id, event, .. }) = gilrs.next_event() {
        let gamepad_id = <_ as Into<usize>>::into(id) as u64;
        match event {
            gilrs::EventType::Connected => {
                send_gamepad_event(app, GamepadEvent::Plugged(gamepad_id));
            }
            gilrs::EventType::Disconnected => {
                send_gamepad_event(app, GamepadEvent::Unplugged(gamepad_id));
            }
            gilrs::EventType::ButtonPressed(button, _) => {
                if let Some(button) = convert_gamepad_button(button) {
                    send_gamepad_event(app, GamepadEvent::PressedButton(gamepad_id, button));
                }
            }
            gilrs::EventType::ButtonReleased(button, _) => {
                if let Some(button) = convert_gamepad_button(button) {
                    send_gamepad_event(app, GamepadEvent::ReleasedButton(gamepad_id, button));
                }
            }
            gilrs::EventType::ButtonChanged(button, value, _) => {
                if let Some(button) = convert_gamepad_button(button) {
                    send_gamepad_event(
                        app,
                        GamepadEvent::UpdatedButtonValue(gamepad_id, button, value),
                    );
                }
            }
            gilrs::EventType::AxisChanged(axis, value, _) => {
                if let Some(axis) = convert_gamepad_axis(axis) {
                    send_gamepad_event(
                        app,
                        GamepadEvent::UpdatedAxisValue(gamepad_id, axis, value),
                    );
                }
            }
            gilrs::EventType::Dropped | gilrs::EventType::ButtonRepeated(_, _) => {}
        }
    }
}

#[cfg(not(target_os = "android"))]
fn convert_gamepad_button(button: gilrs::Button) -> Option<GamepadButton> {
    match button {
        gilrs::Button::South => Some(GamepadButton::South),
        gilrs::Button::East => Some(GamepadButton::East),
        gilrs::Button::North => Some(GamepadButton::North),
        gilrs::Button::West => Some(GamepadButton::West),
        gilrs::Button::C => Some(GamepadButton::C),
        gilrs::Button::Z => Some(GamepadButton::Z),
        gilrs::Button::LeftTrigger => Some(GamepadButton::FrontLeftTrigger),
        gilrs::Button::LeftTrigger2 => Some(GamepadButton::BackLeftTrigger),
        gilrs::Button::RightTrigger => Some(GamepadButton::FrontRightTrigger),
        gilrs::Button::RightTrigger2 => Some(GamepadButton::BackRightTrigger),
        gilrs::Button::Select => Some(GamepadButton::Select),
        gilrs::Button::Start => Some(GamepadButton::Start),
        gilrs::Button::Mode => Some(GamepadButton::Mode),
        gilrs::Button::LeftThumb => Some(GamepadButton::LeftStick),
        gilrs::Button::RightThumb => Some(GamepadButton::RightStick),
        gilrs::Button::DPadUp => Some(GamepadButton::DPadUp),
        gilrs::Button::DPadDown => Some(GamepadButton::DPadDown),
        gilrs::Button::DPadLeft => Some(GamepadButton::DPadLeft),
        gilrs::Button::DPadRight => Some(GamepadButton::DPadRight),
        gilrs::Button::Unknown => None,
    }
}

#[cfg(not(target_os = "android"))]
fn convert_gamepad_axis(button: gilrs::Axis) -> Option<GamepadAxis> {
    match button {
        gilrs::Axis::LeftStickX => Some(GamepadAxis::LeftStickX),
        gilrs::Axis::LeftStickY => Some(GamepadAxis::LeftStickY),
        gilrs::Axis::RightStickX => Some(GamepadAxis::RightStickX),
        gilrs::Axis::RightStickY => Some(GamepadAxis::RightStickY),
        gilrs::Axis::DPadX => Some(GamepadAxis::DPadX),
        gilrs::Axis::DPadY => Some(GamepadAxis::DPadY),
        gilrs::Axis::LeftZ => Some(GamepadAxis::LeftZ),
        gilrs::Axis::RightZ => Some(GamepadAxis::RightZ),
        gilrs::Axis::Unknown => None,
    }
}

fn convert_mouse_button_event(button: event::MouseButton, state: ElementState) -> MouseEvent {
    let button = match button {
        event::MouseButton::Left => MouseButton::Left,
        event::MouseButton::Right => MouseButton::Right,
        event::MouseButton::Middle => MouseButton::Middle,
        event::MouseButton::Other(id) => MouseButton::Other(id),
    };
    match state {
        ElementState::Pressed => MouseEvent::PressedButton(button),
        ElementState::Released => MouseEvent::ReleasedButton(button),
    }
}

#[allow(clippy::too_many_lines)]
fn convert_keyboard_key_event(code: VirtualKeyCode, state: ElementState) -> KeyboardEvent {
    let key = match code {
        VirtualKeyCode::Key1 => Key::Key1,
        VirtualKeyCode::Key2 => Key::Key2,
        VirtualKeyCode::Key3 => Key::Key3,
        VirtualKeyCode::Key4 => Key::Key4,
        VirtualKeyCode::Key5 => Key::Key5,
        VirtualKeyCode::Key6 => Key::Key6,
        VirtualKeyCode::Key7 => Key::Key7,
        VirtualKeyCode::Key8 => Key::Key8,
        VirtualKeyCode::Key9 => Key::Key9,
        VirtualKeyCode::Key0 => Key::Key0,
        VirtualKeyCode::A => Key::A,
        VirtualKeyCode::B => Key::B,
        VirtualKeyCode::C => Key::C,
        VirtualKeyCode::D => Key::D,
        VirtualKeyCode::E => Key::E,
        VirtualKeyCode::F => Key::F,
        VirtualKeyCode::G => Key::G,
        VirtualKeyCode::H => Key::H,
        VirtualKeyCode::I => Key::I,
        VirtualKeyCode::J => Key::J,
        VirtualKeyCode::K => Key::K,
        VirtualKeyCode::L => Key::L,
        VirtualKeyCode::M => Key::M,
        VirtualKeyCode::N => Key::N,
        VirtualKeyCode::O => Key::O,
        VirtualKeyCode::P => Key::P,
        VirtualKeyCode::Q => Key::Q,
        VirtualKeyCode::R => Key::R,
        VirtualKeyCode::S => Key::S,
        VirtualKeyCode::T => Key::T,
        VirtualKeyCode::U => Key::U,
        VirtualKeyCode::V => Key::V,
        VirtualKeyCode::W => Key::W,
        VirtualKeyCode::X => Key::X,
        VirtualKeyCode::Y => Key::Y,
        VirtualKeyCode::Z => Key::Z,
        VirtualKeyCode::Escape => Key::Escape,
        VirtualKeyCode::F1 => Key::F1,
        VirtualKeyCode::F2 => Key::F2,
        VirtualKeyCode::F3 => Key::F3,
        VirtualKeyCode::F4 => Key::F4,
        VirtualKeyCode::F5 => Key::F5,
        VirtualKeyCode::F6 => Key::F6,
        VirtualKeyCode::F7 => Key::F7,
        VirtualKeyCode::F8 => Key::F8,
        VirtualKeyCode::F9 => Key::F9,
        VirtualKeyCode::F10 => Key::F10,
        VirtualKeyCode::F11 => Key::F11,
        VirtualKeyCode::F12 => Key::F12,
        VirtualKeyCode::F13 => Key::F13,
        VirtualKeyCode::F14 => Key::F14,
        VirtualKeyCode::F15 => Key::F15,
        VirtualKeyCode::F16 => Key::F16,
        VirtualKeyCode::F17 => Key::F17,
        VirtualKeyCode::F18 => Key::F18,
        VirtualKeyCode::F19 => Key::F19,
        VirtualKeyCode::F20 => Key::F20,
        VirtualKeyCode::F21 => Key::F21,
        VirtualKeyCode::F22 => Key::F22,
        VirtualKeyCode::F23 => Key::F23,
        VirtualKeyCode::F24 => Key::F24,
        VirtualKeyCode::Snapshot => Key::Snapshot,
        VirtualKeyCode::Scroll => Key::Scroll,
        VirtualKeyCode::Pause => Key::Pause,
        VirtualKeyCode::Insert => Key::Insert,
        VirtualKeyCode::Home => Key::Home,
        VirtualKeyCode::Delete => Key::Delete,
        VirtualKeyCode::End => Key::End,
        VirtualKeyCode::PageDown => Key::PageDown,
        VirtualKeyCode::PageUp => Key::PageUp,
        VirtualKeyCode::Left => Key::Left,
        VirtualKeyCode::Up => Key::Up,
        VirtualKeyCode::Right => Key::Right,
        VirtualKeyCode::Down => Key::Down,
        VirtualKeyCode::Back => Key::Back,
        VirtualKeyCode::Return => Key::Return,
        VirtualKeyCode::Space => Key::Space,
        VirtualKeyCode::Compose => Key::Compose,
        VirtualKeyCode::Caret => Key::Caret,
        VirtualKeyCode::Numlock => Key::Numlock,
        VirtualKeyCode::Numpad0 => Key::Numpad0,
        VirtualKeyCode::Numpad1 => Key::Numpad1,
        VirtualKeyCode::Numpad2 => Key::Numpad2,
        VirtualKeyCode::Numpad3 => Key::Numpad3,
        VirtualKeyCode::Numpad4 => Key::Numpad4,
        VirtualKeyCode::Numpad5 => Key::Numpad5,
        VirtualKeyCode::Numpad6 => Key::Numpad6,
        VirtualKeyCode::Numpad7 => Key::Numpad7,
        VirtualKeyCode::Numpad8 => Key::Numpad8,
        VirtualKeyCode::Numpad9 => Key::Numpad9,
        VirtualKeyCode::NumpadAdd => Key::NumpadAdd,
        VirtualKeyCode::NumpadDivide => Key::NumpadDivide,
        VirtualKeyCode::NumpadDecimal => Key::NumpadDecimal,
        VirtualKeyCode::NumpadComma => Key::NumpadComma,
        VirtualKeyCode::NumpadEnter => Key::NumpadEnter,
        VirtualKeyCode::NumpadEquals => Key::NumpadEquals,
        VirtualKeyCode::NumpadMultiply => Key::NumpadMultiply,
        VirtualKeyCode::NumpadSubtract => Key::NumpadSubtract,
        VirtualKeyCode::AbntC1 => Key::AbntC1,
        VirtualKeyCode::AbntC2 => Key::AbntC2,
        VirtualKeyCode::Apostrophe => Key::Apostrophe,
        VirtualKeyCode::Apps => Key::Apps,
        VirtualKeyCode::Asterisk => Key::Asterisk,
        VirtualKeyCode::At => Key::At,
        VirtualKeyCode::Ax => Key::Ax,
        VirtualKeyCode::Backslash => Key::Backslash,
        VirtualKeyCode::Calculator => Key::Calculator,
        VirtualKeyCode::Capital => Key::Capital,
        VirtualKeyCode::Colon => Key::Colon,
        VirtualKeyCode::Comma => Key::Comma,
        VirtualKeyCode::Convert => Key::Convert,
        VirtualKeyCode::Equals => Key::Equals,
        VirtualKeyCode::Grave => Key::Grave,
        VirtualKeyCode::Kana => Key::Kana,
        VirtualKeyCode::Kanji => Key::Kanji,
        VirtualKeyCode::LAlt => Key::LAlt,
        VirtualKeyCode::LBracket => Key::LBracket,
        VirtualKeyCode::LControl => Key::LControl,
        VirtualKeyCode::LShift => Key::LShift,
        VirtualKeyCode::LWin => Key::LWin,
        VirtualKeyCode::Mail => Key::Mail,
        VirtualKeyCode::MediaSelect => Key::MediaSelect,
        VirtualKeyCode::MediaStop => Key::MediaStop,
        VirtualKeyCode::Minus => Key::Minus,
        VirtualKeyCode::Mute => Key::Mute,
        VirtualKeyCode::MyComputer => Key::MyComputer,
        VirtualKeyCode::NavigateForward => Key::NavigateForward,
        VirtualKeyCode::NavigateBackward => Key::NavigateBackward,
        VirtualKeyCode::NextTrack => Key::NextTrack,
        VirtualKeyCode::NoConvert => Key::NoConvert,
        VirtualKeyCode::OEM102 => Key::OEM102,
        VirtualKeyCode::Period => Key::Period,
        VirtualKeyCode::PlayPause => Key::PlayPause,
        VirtualKeyCode::Plus => Key::Plus,
        VirtualKeyCode::Power => Key::Power,
        VirtualKeyCode::PrevTrack => Key::PrevTrack,
        VirtualKeyCode::RAlt => Key::RAlt,
        VirtualKeyCode::RBracket => Key::RBracket,
        VirtualKeyCode::RControl => Key::RControl,
        VirtualKeyCode::RShift => Key::RShift,
        VirtualKeyCode::RWin => Key::RWin,
        VirtualKeyCode::Semicolon => Key::Semicolon,
        VirtualKeyCode::Slash => Key::Slash,
        VirtualKeyCode::Sleep => Key::Sleep,
        VirtualKeyCode::Stop => Key::Stop,
        VirtualKeyCode::Sysrq => Key::Sysrq,
        VirtualKeyCode::Tab => Key::Tab,
        VirtualKeyCode::Underline => Key::Underline,
        VirtualKeyCode::Unlabeled => Key::Unlabeled,
        VirtualKeyCode::VolumeDown => Key::VolumeDown,
        VirtualKeyCode::VolumeUp => Key::VolumeUp,
        VirtualKeyCode::Wake => Key::Wake,
        VirtualKeyCode::WebBack => Key::WebBack,
        VirtualKeyCode::WebFavorites => Key::WebFavorites,
        VirtualKeyCode::WebForward => Key::WebForward,
        VirtualKeyCode::WebHome => Key::WebHome,
        VirtualKeyCode::WebRefresh => Key::WebRefresh,
        VirtualKeyCode::WebSearch => Key::WebSearch,
        VirtualKeyCode::WebStop => Key::WebStop,
        VirtualKeyCode::Yen => Key::Yen,
        VirtualKeyCode::Copy => Key::Copy,
        VirtualKeyCode::Paste => Key::Paste,
        VirtualKeyCode::Cut => Key::Cut,
    };
    match state {
        ElementState::Pressed => KeyboardEvent::PressedKey(key),
        ElementState::Released => KeyboardEvent::ReleasedKey(key),
    }
}
