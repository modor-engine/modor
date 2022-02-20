use modor::testing::TestApp;
use modor::{system, Built, EntityBuilder, EntityMainComponent, Query, SystemRunner};

struct ButtonSelection {
    label: String,
}

impl EntityMainComponent for ButtonSelection {
    type Data = String;

    fn build(builder: EntityBuilder<'_, Self>, label: Self::Data) -> Built {
        builder.with_self(Self { label })
    }
}

#[derive(PartialEq, Debug)]
struct Button {
    is_pressed: bool,
}

impl EntityMainComponent for Button {
    type Data = String;

    fn build(builder: EntityBuilder<'_, Self>, label: Self::Data) -> Built {
        builder.with(label).with_self(Self { is_pressed: false })
    }

    fn on_update(runner: SystemRunner<'_>) -> SystemRunner<'_> {
        runner.run(system!(Self::update))
    }
}

impl Button {
    #[allow(clippy::ptr_arg)]
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

impl EntityMainComponent for ExitButton {
    type Data = String;

    fn build(builder: EntityBuilder<'_, Self>, label: Self::Data) -> Built {
        builder
            .inherit_from::<Button>(label)
            .with(ExitState(false))
            .with_self(Self)
    }

    fn on_update(runner: SystemRunner<'_>) -> SystemRunner<'_> {
        runner
            .run(system!(Self::update_state))
            .run(system!(Self::update_label))
    }
}

impl ExitButton {
    fn update_state(state: &mut ExitState, button: &Button) {
        state.0 = button.is_pressed;
    }

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
    let other_button_id = app.create_entity::<Button>("New game".into());
    let exit_button_id = app.create_entity::<ExitButton>("Exit".into());
    app.create_entity::<ButtonSelection>("New game".into());
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
    let other_button_id = app.create_entity::<Button>("New game".into());
    let exit_button_id = app.create_entity::<ExitButton>("Exit".into());
    app.create_entity::<ButtonSelection>("Exit".into());
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
