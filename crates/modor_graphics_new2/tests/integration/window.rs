use modor::{App, With};
use modor_graphics_new2::testing::TestRunnerContext;
use modor_graphics_new2::{testing, Window};

#[test]
pub fn test_window() {
    let mut context = TestRunnerContext::default();
    test_default_window(&mut context);
}

fn test_default_window(context: &mut TestRunnerContext) {
    App::new().with_entity(Window::default()).run(|a| {
        testing::test_runner(a, context, 1, |a, _, _| {
            a.assert::<With<Window>>(1, |e| {
                e.has(|w: &Window| assert_eq!(w.title, ""))
                    .has(|w: &Window| assert_eq!(w.size.width, 800))
                    .has(|w: &Window| assert_eq!(w.size.height, 600))
            })
        });
    });
}
