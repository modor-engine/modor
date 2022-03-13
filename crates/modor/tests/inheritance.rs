#[macro_use]
extern crate modor;

use modor::testing::TestApp;
use modor::{Built, EntityBuilder, Query};

struct ButtonSelection {
    label: String,
}

#[entity]
impl ButtonSelection {
    fn build(label: String) -> impl Built<Self> {
        EntityBuilder::new(Self { label })
    }
}

#[derive(PartialEq, Debug)]
struct Button {
    is_pressed: bool,
}

#[entity]
impl Button {
    fn build(label: String) -> impl Built<Self> {
        EntityBuilder::new(Self { is_pressed: false }).with(label)
    }

    #[allow(clippy::ptr_arg)]
    #[run]
    fn update(&mut self, label: &String, selections: Query<'_, &ButtonSelection>) {
        for selection in selections.iter() {
            if &selection.label == label {
                self.is_pressed = true;
            }
        }
    }
}

#[derive(PartialEq, Debug)]
struct ExitButton;

#[entity]
impl ExitButton {
    fn build(label: String) -> impl Built<Self> {
        EntityBuilder::new(Self)
            .inherit_from(Button::build(label))
            .with(ExitState(false))
    }

    #[run]
    fn update_state(state: &mut ExitState, button: &Button) {
        state.0 = button.is_pressed;
    }

    #[run]
    fn update_label(label: &mut String, button: &Button) {
        if button.is_pressed {
            *label = format!("{} (selected)", label);
        }
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
