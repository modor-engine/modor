use modor::{App, With};
use modor_graphics::testing;
use modor_graphics::testing::{TestRunnerContext, UpdateState};
use modor_input::{Finger, InputModule, Key, Keyboard, Mouse, MouseButton};
use modor_internal::assert_approx_eq;
use modor_math::Vec2;
use modor_physics::{DeltaTime, PhysicsModule};
use winit::dpi::PhysicalPosition;
use winit::event::{
    DeviceEvent, DeviceId, ElementState, Event, KeyboardInput, ModifiersState, MouseScrollDelta,
    Touch, TouchPhase, VirtualKeyCode, WindowEvent,
};

#[allow(unsafe_code)] // safe because never passed to a winit function
const DEVICE_ID: DeviceId = unsafe { DeviceId::dummy() };

pub fn run_window_tests(context: &mut TestRunnerContext) {
    press_mouse_button(context);
    move_mouse_wheel_in_lines(context);
    move_mouse_wheel_in_pixels(context);
    move_mouse(context);
    detect_mouse_motion(context);
    press_keyboard_key(context);
    enter_character(context);
    touch_screen(context);
    suspend_app(context);
}

fn press_mouse_button(context: &mut TestRunnerContext) {
    App::new().with_entity(InputModule::build()).run(|a| {
        testing::test_runner(a, context, 3, |s| {
            let is_pressed = s.update_id == 0;
            s.next_events.push(left_mouse_button_event(&s, is_pressed));
            s.app.assert::<With<Mouse>>(1, |e| {
                e.has(|m: &Mouse| {
                    assert_eq!(m.button(MouseButton::Left).is_pressed, s.update_id == 1);
                })
            })
        });
    });
}

fn move_mouse_wheel_in_lines(context: &mut TestRunnerContext) {
    App::new().with_entity(InputModule::build()).run(|a| {
        testing::test_runner(a, context, 2, |s| {
            if s.update_id == 0 {
                let delta = MouseScrollDelta::LineDelta(2., 3.);
                s.next_events.push(mouse_wheel_event(&s, delta));
                s.app
            } else {
                s.app.assert::<With<Mouse>>(1, |e| {
                    e.has(|m: &Mouse| {
                        assert_approx_eq!(m.scroll_delta_in_lines(100., 200.), Vec2::new(2., -3.));
                    })
                })
            }
        });
    });
}

fn move_mouse_wheel_in_pixels(context: &mut TestRunnerContext) {
    App::new().with_entity(InputModule::build()).run(|a| {
        testing::test_runner(a, context, 2, |s| {
            if s.update_id == 0 {
                let delta = MouseScrollDelta::PixelDelta(PhysicalPosition::new(2., 3.));
                s.next_events.push(mouse_wheel_event(&s, delta));
                s.app
            } else {
                s.app.assert::<With<Mouse>>(1, |e| {
                    e.has(|m: &Mouse| {
                        assert_approx_eq!(m.scroll_delta_in_pixels(100., 200.), Vec2::new(2., -3.));
                    })
                })
            }
        });
    });
}

fn move_mouse(context: &mut TestRunnerContext) {
    App::new().with_entity(InputModule::build()).run(|a| {
        testing::test_runner(a, context, 2, |s| {
            if s.update_id == 0 {
                let position = PhysicalPosition::new(5., 9.);
                s.next_events.push(cursor_moved_event(&s, position));
                s.app
            } else {
                s.app.assert::<With<Mouse>>(1, |e| {
                    e.has(|m: &Mouse| assert_eq!(m.position(), Vec2::new(5., 9.)))
                })
            }
        });
    });
}

fn detect_mouse_motion(context: &mut TestRunnerContext) {
    App::new().with_entity(InputModule::build()).run(|a| {
        testing::test_runner(a, context, 2, |s| {
            if s.update_id == 0 {
                s.next_events.push(mouse_motion_event((1., 2.)));
                s.next_events.push(mouse_motion_event((3., 1.)));
                s.app
            } else {
                s.app.assert::<With<Mouse>>(1, |e| {
                    e.has(|m: &Mouse| assert_eq!(m.delta(), Vec2::new(4., -3.)))
                })
            }
        });
    });
}

fn press_keyboard_key(context: &mut TestRunnerContext) {
    App::new().with_entity(InputModule::build()).run(|a| {
        testing::test_runner(a, context, 3, |s| {
            let is_pressed = s.update_id == 0;
            s.next_events.push(space_key_event(&s, is_pressed));
            s.app.assert::<With<Keyboard>>(1, |e| {
                e.has(|k: &Keyboard| {
                    assert_eq!(k.key(Key::Space).is_pressed, s.update_id == 1);
                })
            })
        });
    });
}

fn enter_character(context: &mut TestRunnerContext) {
    App::new().with_entity(InputModule::build()).run(|a| {
        testing::test_runner(a, context, 2, |s| {
            if s.update_id == 0 {
                s.next_events.push(received_character_event(&s, 'A'));
                s.next_events.push(received_character_event(&s, 'B'));
                s.app
            } else {
                s.app.assert::<With<Keyboard>>(1, |e| {
                    e.has(|k: &Keyboard| assert_eq!(k.text(), "AB"))
                })
            }
        });
    });
}

fn touch_screen(context: &mut TestRunnerContext) {
    App::new().with_entity(InputModule::build()).run(|a| {
        testing::test_runner(a, context, 4, |s| match s.update_id {
            0 => {
                let position = PhysicalPosition::new(4., 8.);
                s.next_events
                    .push(touch_event(&s, 42, position, TouchPhase::Started));
                s.app
            }
            1 => {
                let position = PhysicalPosition::new(5., 8.);
                s.next_events
                    .push(touch_event(&s, 42, position, TouchPhase::Moved));
                s.app.assert::<With<Finger>>(1, |e| {
                    e.has(|f: &Finger| {
                        assert_eq!(f.id(), 42);
                        assert!(f.state().is_pressed);
                        assert_approx_eq!(f.position(), Vec2::new(4., 8.));
                    })
                })
            }
            2 => {
                let position = PhysicalPosition::new(5., 8.);
                s.next_events
                    .push(touch_event(&s, 42, position, TouchPhase::Ended));
                s.app.assert::<With<Finger>>(1, |e| {
                    e.has(|f: &Finger| {
                        assert_eq!(f.id(), 42);
                        assert!(f.state().is_pressed);
                        assert_approx_eq!(f.position(), Vec2::new(5., 8.));
                    })
                })
            }
            _ => s.app.assert::<With<Finger>>(1, |e| {
                e.has(|f: &Finger| {
                    assert_eq!(f.id(), 42);
                    assert!(!f.state().is_pressed);
                })
            }),
        });
    });
}

fn suspend_app(context: &mut TestRunnerContext) {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(PhysicsModule::build())
        .run(|a| {
            testing::test_runner(a, context, 3, |s| match s.update_id {
                0 => {
                    s.next_events.push(Event::Suspended);
                    s.app.assert::<With<DeltaTime>>(1, |e| {
                        e.has(|d: &DeltaTime| assert_ne!(d.get().as_nanos(), 0))
                    })
                }
                1 => {
                    s.next_events.push(Event::Resumed);
                    s.app.assert::<With<DeltaTime>>(1, |e| {
                        e.has(|d: &DeltaTime| assert_eq!(d.get().as_nanos(), 0))
                    })
                }
                _ => s.app.assert::<With<DeltaTime>>(1, |e| {
                    e.has(|d: &DeltaTime| assert_ne!(d.get().as_nanos(), 0))
                }),
            });
        });
}

#[allow(deprecated)]
fn left_mouse_button_event(state: &UpdateState<'_>, is_pressed: bool) -> Event<'static, ()> {
    Event::WindowEvent {
        window_id: state.window.id(),
        event: WindowEvent::MouseInput {
            device_id: DEVICE_ID,
            state: if is_pressed {
                ElementState::Pressed
            } else {
                ElementState::Released
            },
            button: winit::event::MouseButton::Left,
            modifiers: ModifiersState::empty(),
        },
    }
}

#[allow(deprecated)]
fn mouse_wheel_event(state: &UpdateState<'_>, delta: MouseScrollDelta) -> Event<'static, ()> {
    Event::WindowEvent {
        window_id: state.window.id(),
        event: WindowEvent::MouseWheel {
            device_id: DEVICE_ID,
            delta,
            phase: TouchPhase::Started,
            modifiers: ModifiersState::empty(),
        },
    }
}

#[allow(deprecated)]
fn cursor_moved_event(
    state: &UpdateState<'_>,
    position: PhysicalPosition<f64>,
) -> Event<'static, ()> {
    Event::WindowEvent {
        window_id: state.window.id(),
        event: WindowEvent::CursorMoved {
            device_id: DEVICE_ID,
            position,
            modifiers: ModifiersState::empty(),
        },
    }
}

fn mouse_motion_event(delta: (f64, f64)) -> Event<'static, ()> {
    Event::DeviceEvent {
        device_id: DEVICE_ID,
        event: DeviceEvent::MouseMotion { delta },
    }
}

#[allow(deprecated)]
fn space_key_event(state: &UpdateState<'_>, is_pressed: bool) -> Event<'static, ()> {
    Event::WindowEvent {
        window_id: state.window.id(),
        event: WindowEvent::KeyboardInput {
            device_id: DEVICE_ID,
            input: KeyboardInput {
                scancode: 0,
                state: if is_pressed {
                    ElementState::Pressed
                } else {
                    ElementState::Released
                },
                virtual_keycode: Some(VirtualKeyCode::Space),
                modifiers: ModifiersState::empty(),
            },
            is_synthetic: false,
        },
    }
}

fn received_character_event(state: &UpdateState<'_>, character: char) -> Event<'static, ()> {
    Event::WindowEvent {
        window_id: state.window.id(),
        event: WindowEvent::ReceivedCharacter(character),
    }
}

fn touch_event(
    state: &UpdateState<'_>,
    id: u64,
    position: PhysicalPosition<f64>,
    phase: TouchPhase,
) -> Event<'static, ()> {
    Event::WindowEvent {
        window_id: state.window.id(),
        event: WindowEvent::Touch(Touch {
            device_id: DEVICE_ID,
            phase,
            location: position,
            force: None,
            id,
        }),
    }
}
