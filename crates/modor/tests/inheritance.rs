use modor::testing::TestApp;
use modor::{system, Built, EntityBuilder, EntityMainComponent, Query, SystemRunner};

struct ButtonSelection {
    label: String,
}

impl ButtonSelection {
    fn build(label: String) -> impl Built<Self> {
        EntityBuilder::new(Self { label })
    }
}

impl EntityMainComponent for ButtonSelection {
    type Type = ();
}

#[derive(PartialEq, Debug)]
struct Button {
    is_pressed: bool,
}

impl Button {
    fn build(label: String) -> impl Built<Self> {
        EntityBuilder::new(Self { is_pressed: false }).with(label)
    }

    #[allow(clippy::ptr_arg)]
    fn update(&mut self, label: &String, selections: Query<'_, &ButtonSelection>) {
        for selection in selections.iter() {
            if &selection.label == label {
                self.is_pressed = true;
            }
        }
    }
}

impl EntityMainComponent for Button {
    type Type = ();

    fn on_update(runner: SystemRunner<'_>) -> SystemRunner<'_> {
        runner.run(system!(Self::update))
    }
}

#[derive(PartialEq, Debug)]
struct ExitButton;

impl ExitButton {
    fn build(label: String) -> impl Built<Self> {
        EntityBuilder::new(Self)
            .inherit_from(Button::build(label))
            .with(ExitState(false))
    }

    fn update_state(state: &mut ExitState, button: &Button) {
        state.0 = button.is_pressed;
    }

    fn update_label(label: &mut String, button: &Button) {
        if button.is_pressed {
            *label = format!("{} (selected)", label);
        }
    }
}

impl EntityMainComponent for ExitButton {
    type Type = ();

    fn on_update(runner: SystemRunner<'_>) -> SystemRunner<'_> {
        runner
            .run(system!(Self::update_state))
            .run(system!(Self::update_label))
    }
}

struct ExitState(bool);

#[test]
fn run_entity_systems() {
    let mut app = TestApp::new();
    let other_button_id = app.create_entity(Button::build("New game".into()));
    let exit_button_id = app.create_entity(ExitButton::build("Exit".into()));
    app.create_entity(ButtonSelection::build("New game".into()));
    app.update();
    app.assert_entity(other_button_id)
        .has::<Button, _>(|c| assert!(c.is_pressed))
        .has::<String, _>(|c| assert_eq!(c, "New game"));
    app.assert_entity(exit_button_id)
        .has::<ExitButton, _>(|_| ())
        .has::<ExitState, _>(|c| assert!(!c.0))
        .has::<Button, _>(|c| assert!(!c.is_pressed))
        .has::<String, _>(|c| assert_eq!(c, "Exit"));
}

#[test]
fn run_inherited_systems() {
    let mut app = TestApp::new();
    let other_button_id = app.create_entity(Button::build("New game".into()));
    let exit_button_id = app.create_entity(ExitButton::build("Exit".into()));
    app.create_entity(ButtonSelection::build("Exit".into()));
    app.update();
    app.assert_entity(other_button_id)
        .has::<Button, _>(|c| assert!(!c.is_pressed))
        .has::<String, _>(|c| assert_eq!(c, "New game"));
    app.assert_entity(exit_button_id)
        .has::<ExitButton, _>(|_| ())
        .has::<ExitState, _>(|c| assert!(c.0))
        .has::<Button, _>(|c| assert!(c.is_pressed))
        .has::<String, _>(|c| assert_eq!(c, "Exit (selected)"));
}
