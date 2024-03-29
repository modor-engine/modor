use modor::{App, BuiltEntity, EntityBuilder, EntityMut, With};
use modor_graphics::testing::TestRunnerContext;
use modor_graphics::{
    instance_2d, testing, AntiAliasing, AntiAliasingMode, Camera2D, Color, Default2DMaterial,
    FrameRate, RenderTarget, Size, Window, WindowCloseBehavior,
};
use modor_resources::ResKey;
use std::thread;
use std::time::Duration;
use winit::dpi::PhysicalSize;
use winit::event::{Event, WindowEvent};

pub fn run_window_tests(context: &mut TestRunnerContext) {
    create_default_window(context);
    create_customized_window(context);
    create_target_window(context);
    create_window_after_start(context);
    delete_window(context);
    set_window_properties(context);
    resize_window(context);
    close_window_with_exit_behavior(context);
    close_window_with_none_behavior(context);
}

fn create_default_window(context: &mut TestRunnerContext) {
    App::new().with_entity(Window::default()).run(|a| {
        testing::test_runner(a, context, 2, |s| {
            s.app.assert::<With<Window>>(1, |e| {
                e.has(|w: &Window| assert_eq!(w.title, ""))
                    .has(|w: &Window| assert_eq!(w.close_behavior, WindowCloseBehavior::Exit))
                    .has(|w: &Window| assert!(w.is_cursor_shown))
                    .has(|w: &Window| assert_eq!(w.size(), Size::new(800, 600)))
                    .has(|w: &Window| assert!(!w.is_closing_requested()))
            })
        });
    });
}

fn create_customized_window(context: &mut TestRunnerContext) {
    let mut window = Window::default();
    window.title = "title".into();
    window.close_behavior = WindowCloseBehavior::None;
    window.is_cursor_shown = false;
    App::new().with_entity(window).run(|a| {
        testing::test_runner(a, context, 2, |s| {
            s.app.assert::<With<Window>>(1, |e| {
                e.has(|w: &Window| assert_eq!(w.title, "title"))
                    .has(|w: &Window| assert_eq!(w.close_behavior, WindowCloseBehavior::None))
                    .has(|w: &Window| assert!(!w.is_cursor_shown))
                    .has(|w: &Window| assert_eq!(w.size(), Size::new(800, 600)))
                    .has(|w: &Window| assert!(!w.is_closing_requested()))
            })
        });
    });
}

fn create_target_window(context: &mut TestRunnerContext) {
    let target_key = ResKey::new("main");
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(
            EntityBuilder::new()
                .component(Window::default())
                .component(RenderTarget::new(target_key)),
        )
        .with_entity(AntiAliasing::from(AntiAliasingMode::MsaaX4))
        .with_entity(FrameRate::Unlimited)
        .with_entity(Camera2D::new(CAMERA, target_key))
        .with_entity(opaque_rectangle())
        .with_entity(transparent_rectangle())
        .run(|a| {
            testing::test_runner(a, context, 4, |s| {
                let size = match s.update_id {
                    0 => Size::new(800, 600),
                    1 => {
                        let _ = s.window.request_inner_size(PhysicalSize::new(400, 300));
                        thread::sleep(Duration::from_millis(100));
                        Size::new(800, 600)
                    }
                    _ => Size::new(400, 300),
                };
                s.app
                    .assert::<With<Window>>(1, |e| e.has(|w: &Window| assert_eq!(w.size(), size)))
            });
        });
}

#[allow(dead_code)]
fn create_window_after_start(context: &mut TestRunnerContext) {
    App::new().run(|a| {
        testing::test_runner(a, context, 3, |s| {
            assert_eq!(s.window.inner_size(), PhysicalSize::new(800, 600));
            if s.update_id == 0 {
                #[cfg(target_os = "windows")] // Window::is_visible not well supported on other platforms
                assert!(!s.window.is_visible().unwrap());
                s.app.with_entity(Window::default())
            } else {
                #[cfg(target_os = "windows")] // Window::is_visible not well supported on other platforms
                assert!(s.window.is_visible().unwrap());
                s.app
            }
        });
    });
}

#[allow(dead_code)]
fn delete_window(context: &mut TestRunnerContext) {
    App::new()
        .with_entity(
            EntityBuilder::new()
                .component(Window::default())
                .component(AutoRemove(2)),
        )
        .run(|a| {
            testing::test_runner(a, context, 4, |s| {
                assert_eq!(s.window.inner_size(), PhysicalSize::new(800, 600));
                #[cfg(target_os = "windows")] // Window::is_visible not well supported on other platforms
                assert_eq!(s.window.is_visible().unwrap(), s.update_id == 0);
                s.app
            });
        });
}

fn set_window_properties(context: &mut TestRunnerContext) {
    App::new().with_entity(Window::default()).run(|a| {
        testing::test_runner(a, context, 3, |s| {
            if s.update_id == 0 {
                s.app.with_update::<(), _>(|w: &mut Window| {
                    w.title = "new title".into();
                    w.is_cursor_shown = false;
                })
            } else {
                s.app.assert::<With<Window>>(1, |e| {
                    e.has(|w: &Window| assert_eq!(w.title, "new title"))
                        .has(|w: &Window| assert!(!w.is_cursor_shown))
                })
            }
        });
    });
}

fn resize_window(context: &mut TestRunnerContext) {
    App::new().with_entity(Window::default()).run(|a| {
        testing::test_runner(a, context, 10, |s| {
            if s.update_id == 0 {
                let _ = s.window.request_inner_size(PhysicalSize::new(400, 300));
                s.app
            } else if s.update_id == 1 {
                s.app.assert::<With<Window>>(1, |e| {
                    e.has(|w: &Window| assert_eq!(w.size(), Size::new(800, 600)))
                })
            } else {
                s.app.assert::<With<Window>>(1, |e| {
                    e.has(|w: &Window| assert_eq!(w.size(), Size::new(400, 300)))
                })
            }
        });
    });
}

fn close_window_with_exit_behavior(context: &mut TestRunnerContext) {
    let mut update_count = 0;
    let mut window = Window::default();
    window.close_behavior = WindowCloseBehavior::Exit;
    App::new().with_entity(window).run(|a| {
        testing::test_runner(a, context, 10, |s| {
            if s.update_id == 0 {
                s.next_events.push(Event::WindowEvent {
                    window_id: s.window.id(),
                    event: WindowEvent::CloseRequested,
                });
            } else {
                *s.is_exit_requested = true;
            }
            update_count += 1;
            s.app
        });
    });
    assert_eq!(update_count, 2);
}

fn close_window_with_none_behavior(context: &mut TestRunnerContext) {
    let mut window = Window::default();
    window.close_behavior = WindowCloseBehavior::None;
    App::new().with_entity(window).run(|a| {
        testing::test_runner(a, context, 4, |s| {
            if s.update_id == 0 {
                s.next_events.push(Event::WindowEvent {
                    window_id: s.window.id(),
                    event: WindowEvent::CloseRequested,
                });
                s.app.assert::<With<Window>>(1, |e| {
                    e.has(|w: &Window| assert!(!w.is_closing_requested()))
                })
            } else {
                s.app.assert::<With<Window>>(1, |e| {
                    e.has(|w: &Window| assert!(w.is_closing_requested()))
                })
            }
        });
    });
}

fn opaque_rectangle() -> impl BuiltEntity {
    instance_2d(CAMERA, Default2DMaterial::new())
}

fn transparent_rectangle() -> impl BuiltEntity {
    instance_2d(CAMERA, Default2DMaterial::new())
        .updated(|m: &mut Default2DMaterial| m.color = Color::WHITE.with_alpha(0.5))
}

#[derive(Component)]
struct AutoRemove(u32);

#[systems]
impl AutoRemove {
    #[run]
    fn update(&mut self, mut entity: EntityMut<'_>) {
        if self.0 == 0 {
            entity.delete();
        } else {
            self.0 -= 1;
        }
    }
}

const CAMERA: ResKey<Camera2D> = ResKey::new("main");
