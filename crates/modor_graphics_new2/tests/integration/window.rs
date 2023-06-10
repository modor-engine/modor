use modor::{App, BuiltEntity, Entity, EntityBuilder, With, World};
use modor_graphics_new2::testing::TestRunnerContext;
use modor_graphics_new2::{testing, RenderTarget, Size, Window, WindowCloseBehavior};
use std::thread;
use std::time::Duration;
use winit::dpi::PhysicalSize;
use winit::event::{Event, WindowEvent};

pub fn run_window_tests(context: &mut TestRunnerContext) {
    create_default_window(context);
    create_customized_window(context);
    create_target_window(context);
    #[cfg(any(target_os = "windows"))] // Window::is_visible not well supported on other platforms
    create_window_after_start(context);
    #[cfg(target_os = "windows")] // Window::is_visible not well supported on other platforms
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
    App::new()
        .with_entity(
            Window::default()
                .with_title("title")
                .with_close_behavior(WindowCloseBehavior::None)
                .with_cursor_shown(false),
        )
        .run(|a| {
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
    App::new()
        .with_entity(
            EntityBuilder::new()
                .with(Window::default())
                .with(RenderTarget::new(TargetKey)),
        )
        .run(|a| {
            testing::test_runner(a, context, 2, |s| {
                s.app.assert::<With<Window>>(1, |e| {
                    e.has(|w: &Window| assert_eq!(w.size(), Size::new(800, 600)))
                })
            });
        });
}

#[allow(dead_code)]
fn create_window_after_start(context: &mut TestRunnerContext) {
    App::new().run(|a| {
        testing::test_runner(a, context, 2, |s| {
            assert_eq!(s.window.inner_size(), PhysicalSize::new(800, 600));
            if s.update_id == 0 {
                assert!(!s.window.is_visible().unwrap());
                s.app.with_entity(Window::default())
            } else {
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
                .with(Window::default())
                .with(AutoRemove),
        )
        .run(|a| {
            testing::test_runner(a, context, 2, |s| {
                assert_eq!(s.window.inner_size(), PhysicalSize::new(800, 600));
                assert_eq!(s.window.is_visible().unwrap(), s.update_id == 0);
                s.app
            });
        });
}

fn set_window_properties(context: &mut TestRunnerContext) {
    App::new().with_entity(Window::default()).run(|a| {
        testing::test_runner(a, context, 2, |s| {
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
                s.window.set_inner_size(PhysicalSize::new(400, 300));
                thread::sleep(Duration::from_millis(100));
                s.app
            } else {
                s.app.assert::<With<Window>>(1, |e| {
                    e.has(|w: &Window| assert_eq!(w.size(), Size::new(400, 300)))
                })
            }
        });
    });
}

// fn close_window_with_exit_behavior(context: &mut TestRunnerContext) {
//     let mut update_count = 0;
//     App::new()
//         .with_entity(Window::default().with_close_behavior(WindowCloseBehavior::Exit))
//         .run(|a| {
//             testing::test_runner(a, context, 3, |s| {
//                 s.next_events.push(Event::WindowEvent {
//                     window_id: s.window.id(),
//                     event: WindowEvent::CloseRequested,
//                 });
//                 update_count += 1;
//                 s.app
//             });
//         });
//     assert_eq!(update_count, 1);
// }
//
// fn close_window_with_none_behavior(context: &mut TestRunnerContext) {
//     App::new()
//         .with_entity(Window::default().with_close_behavior(WindowCloseBehavior::None))
//         .run(|a| {
//             testing::test_runner(a, context, 3, |s| {
//                 if s.update_id == 0 {
//                     s.next_events.push(Event::WindowEvent {
//                         window_id: s.window.id(),
//                         event: WindowEvent::CloseRequested,
//                     });
//                     s.app.assert::<With<Window>>(1, |e| {
//                         e.has(|w: &Window| assert!(!w.is_closing_requested()))
//                     })
//                 } else {
//                     s.app.assert::<With<Window>>(1, |e| {
//                         e.has(|w: &Window| assert!(w.is_closing_requested()))
//                     })
//                 }
//             });
//         });
// }

#[derive(Component)]
struct AutoRemove;

#[systems]
impl AutoRemove {
    #[run]
    fn update(entity: Entity<'_>, mut world: World<'_>) {
        world.delete_entity(entity.id());
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TargetKey;
