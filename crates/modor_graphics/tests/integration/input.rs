use modor::{App, SingleRef, With};
use modor_graphics::testing;
use modor_graphics::testing::{TestRunnerContext, UpdateState};
use modor_input::{Fingers, Mouse, MouseButton};
use modor_internal::assert_approx_eq;
use modor_math::Vec2;
use modor_physics::DeltaTime;
use winit::dpi::PhysicalPosition;
use winit::event::{
    DeviceEvent, DeviceId, ElementState, Event, MouseScrollDelta, Touch, TouchPhase, WindowEvent,
};

#[allow(unsafe_code)] // safe because never passed to a winit function
const DEVICE_ID: DeviceId = unsafe { DeviceId::dummy() };

pub fn run_window_tests(context: &mut TestRunnerContext) {
    press_mouse_button(context);
    move_mouse_wheel_in_lines(context);
    move_mouse_wheel_in_pixels(context);
    move_mouse(context);
    detect_mouse_motion(context);
    touch_screen(context);
    suspend_app(context);
}

fn press_mouse_button(context: &mut TestRunnerContext) {
    App::new().with_entity(modor_input::module()).run(|a| {
        testing::test_runner(a, context, 3, |s| {
            let is_pressed = s.update_id == 0;
            s.next_events.push(left_mouse_button_event(&s, is_pressed));
            s.app.assert::<With<Mouse>>(1, |e| {
                e.has(|m: &Mouse| {
                    assert_eq!(m[MouseButton::Left].is_pressed(), s.update_id == 1);
                })
            })
        });
    });
}

fn move_mouse_wheel_in_lines(context: &mut TestRunnerContext) {
    App::new()
        .with_entity(modor_input::module())
        .with_entity(MouseTest::new(1, |m| {
            assert_approx_eq!(m.scroll_delta.as_lines(100., 200.), Vec2::new(2., -3.));
        }))
        .run(|a| {
            testing::test_runner(a, context, 2, |s| {
                if s.update_id == 0 {
                    let delta = MouseScrollDelta::LineDelta(2., 3.);
                    s.next_events.push(mouse_wheel_event(&s, delta));
                    s.app
                } else {
                    s.app.assert::<With<Mouse>>(1, |e| {
                        e.has(|m: &Mouse| {
                            assert_approx_eq!(m.scroll_delta.as_lines(100., 200.), Vec2::ZERO);
                        })
                    })
                }
            });
        });
}

fn move_mouse_wheel_in_pixels(context: &mut TestRunnerContext) {
    App::new()
        .with_entity(modor_input::module())
        .with_entity(MouseTest::new(1, |m| {
            assert_approx_eq!(m.scroll_delta.as_pixels(100., 200.), Vec2::new(2., -3.));
        }))
        .run(|a| {
            testing::test_runner(a, context, 2, |s| {
                if s.update_id == 0 {
                    let delta = MouseScrollDelta::PixelDelta(PhysicalPosition::new(2., 3.));
                    s.next_events.push(mouse_wheel_event(&s, delta));
                    s.app
                } else {
                    s.app.assert::<With<Mouse>>(1, |e| {
                        e.has(|m: &Mouse| {
                            assert_approx_eq!(m.scroll_delta.as_pixels(100., 200.), Vec2::ZERO);
                        })
                    })
                }
            });
        });
}

fn move_mouse(context: &mut TestRunnerContext) {
    App::new().with_entity(modor_input::module()).run(|a| {
        testing::test_runner(a, context, 2, |s| {
            if s.update_id == 0 {
                let position = PhysicalPosition::new(5., 9.);
                s.next_events.push(cursor_moved_event(&s, position));
                s.app
            } else {
                s.app.assert::<With<Mouse>>(1, |e| {
                    e.has(|m: &Mouse| assert_eq!(m.position, Vec2::new(5., 9.)))
                })
            }
        });
    });
}

fn detect_mouse_motion(context: &mut TestRunnerContext) {
    App::new()
        .with_entity(modor_input::module())
        .with_entity(MouseTest::new(1, |m| {
            assert_approx_eq!(m.delta, Vec2::new(4., -3.));
        }))
        .run(|a| {
            testing::test_runner(a, context, 2, |s| {
                if s.update_id == 0 {
                    s.next_events.push(mouse_motion_event((1., 2.)));
                    s.next_events.push(mouse_motion_event((3., 1.)));
                    s.app
                } else {
                    s.app.assert::<With<Mouse>>(1, |e| {
                        e.has(|m: &Mouse| assert_approx_eq!(m.delta, Vec2::ZERO))
                    })
                }
            });
        });
}

fn touch_screen(context: &mut TestRunnerContext) {
    App::new()
        .with_entity(modor_input::module())
        .with_entity(FingersTest::new(1, |f| {
            assert_approx_eq!(f[42].delta, Vec2::ZERO);
        }))
        .with_entity(FingersTest::new(2, |f| {
            assert_approx_eq!(f[42].delta, Vec2::new(1., 0.));
        }))
        .run(|a| {
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
                    s.app.assert::<With<Fingers>>(1, |e| {
                        e.has(|f: &Fingers| {
                            assert!(f[42].state.is_pressed());
                            assert_approx_eq!(f[42].position, Vec2::new(4., 8.));
                            assert_approx_eq!(f[42].delta, Vec2::ZERO);
                        })
                    })
                }
                2 => {
                    let position = PhysicalPosition::new(5., 8.);
                    s.next_events
                        .push(touch_event(&s, 42, position, TouchPhase::Ended));
                    s.app.assert::<With<Fingers>>(1, |e| {
                        e.has(|f: &Fingers| {
                            assert!(f[42].state.is_pressed());
                            assert_approx_eq!(f[42].position, Vec2::new(5., 8.));
                            assert_approx_eq!(f[42].delta, Vec2::ZERO);
                        })
                    })
                }
                _ => s.app.assert::<With<Fingers>>(1, |e| {
                    e.has(|f: &Fingers| {
                        assert!(!f[42].state.is_pressed());
                    })
                }),
            });
        });
}

fn suspend_app(context: &mut TestRunnerContext) {
    App::new().with_entity(modor_graphics::module()).run(|a| {
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

fn left_mouse_button_event(state: &UpdateState<'_>, is_pressed: bool) -> Event<()> {
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
        },
    }
}

fn mouse_wheel_event(state: &UpdateState<'_>, delta: MouseScrollDelta) -> Event<()> {
    Event::WindowEvent {
        window_id: state.window.id(),
        event: WindowEvent::MouseWheel {
            device_id: DEVICE_ID,
            delta,
            phase: TouchPhase::Started,
        },
    }
}

fn cursor_moved_event(state: &UpdateState<'_>, position: PhysicalPosition<f64>) -> Event<()> {
    Event::WindowEvent {
        window_id: state.window.id(),
        event: WindowEvent::CursorMoved {
            device_id: DEVICE_ID,
            position,
        },
    }
}

fn mouse_motion_event(delta: (f64, f64)) -> Event<()> {
    Event::DeviceEvent {
        device_id: DEVICE_ID,
        event: DeviceEvent::MouseMotion { delta },
    }
}

fn touch_event(
    state: &UpdateState<'_>,
    id: u64,
    position: PhysicalPosition<f64>,
    phase: TouchPhase,
) -> Event<()> {
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

#[derive(Component)]
struct MouseTest {
    current_update_id: usize,
    test_update_id: usize,
    test: fn(&Mouse),
}

#[systems]
impl MouseTest {
    fn new(test_update_id: usize, test: fn(&Mouse)) -> Self {
        Self {
            current_update_id: 0,
            test_update_id,
            test,
        }
    }

    #[run]
    fn update(&mut self, mouse: SingleRef<'_, '_, Mouse>) {
        if self.current_update_id == self.test_update_id {
            (self.test)(mouse.get());
        }
        self.current_update_id += 1;
    }
}

#[derive(Component)]
struct FingersTest {
    current_update_id: usize,
    test_update_id: usize,
    test: fn(&Fingers),
}

#[systems]
impl FingersTest {
    fn new(test_update_id: usize, test: fn(&Fingers)) -> Self {
        Self {
            current_update_id: 0,
            test_update_id,
            test,
        }
    }

    #[run]
    fn update(&mut self, fingers: SingleRef<'_, '_, Fingers>) {
        if self.current_update_id == self.test_update_id {
            (self.test)(fingers.get());
        }
        self.current_update_id += 1;
    }
}
