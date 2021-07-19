#![allow(clippy::print_stdout, missing_docs)]

use modor::*;

fn main() {
    let mut app = Application::new().with_group(build_main_menu_group);
    app.update();
    println!("----------");
    app.run(system_once!(print_button_labels));
    app.run(system_once!(click_on_settings_button));
    app.update();
    println!("----------");
    app.run(system_once!(print_button_labels));
}

#[allow(clippy::ptr_arg)]
fn print_button_labels(_button: &mut Button, label: &String) {
    println!("{}", label)
}

#[allow(clippy::ptr_arg)]
fn click_on_settings_button(button: &mut Button, label: &String) {
    if label == "Settings" {
        button.0 = true;
    }
}

fn build_main_menu_group(builder: &mut GroupBuilder<'_>) {
    builder
        .with_entity::<Button>("New game")
        .with_entity::<SettingsButton>("Settings")
        .with_entity::<Button>("Exit");
}

fn build_settings_menu_group(builder: &mut GroupBuilder<'_>) {
    builder
        .with_entity::<Button>("Enable fullscreen mode")
        .with_entity::<Button>("Ok")
        .with_entity::<SettingsBackButton>("Back");
}

struct Button(bool);

impl EntityMainComponent for Button {
    type Data = &'static str;

    fn build(builder: &mut EntityBuilder<'_, Self>, data: Self::Data) -> Built {
        builder.with(String::from(data)).with_self(Self(false))
    }
}

struct SettingsButton;

impl EntityMainComponent for SettingsButton {
    type Data = &'static str;

    fn build(builder: &mut EntityBuilder<'_, Self>, data: Self::Data) -> Built {
        builder.inherit_from::<Button>(data).with_self(Self)
    }

    fn on_update(runner: &mut EntityRunner<'_, Self>) {
        runner.run(entity_system!(Self::update));
    }
}

impl SettingsButton {
    fn update(button: &Button, mut group: Group<'_>) {
        if button.0 {
            group.replace(build_settings_menu_group);
        }
    }
}

struct SettingsBackButton;

impl EntityMainComponent for SettingsBackButton {
    type Data = &'static str;

    fn build(builder: &mut EntityBuilder<'_, Self>, data: Self::Data) -> Built {
        builder.inherit_from::<Button>(data).with_self(Self)
    }

    fn on_update(runner: &mut EntityRunner<'_, Self>) {
        runner.run(entity_system!(Self::update));
    }
}

impl SettingsBackButton {
    fn update(button: &Button, mut group: Group<'_>) {
        if button.0 {
            group.replace(build_main_menu_group);
        }
    }
}
