#![allow(clippy::print_stdout)]

use modor::*;

fn main() {
    let mut app = Application::default()
        .with_group(|b| build_main_group(b, 0))
        .with_group(|b| build_main_group(b, 1))
        .with_group(|b| build_main_group(b, 2))
        .on_update(system!(print_id));
    println!("Update 1:");
    app.update();
    println!("Update 2:");
    app.update();
}

fn build_main_group(builder: &mut GroupBuilder<'_>, group_id: u32) {
    builder
        .with_entity::<Text>((group_id * 2, group_id))
        .with_entity::<Text>((group_id * 2 + 1, group_id))
        .on_update(system!(print_text));
}

struct Id;

impl EntityMainComponent for Id {
    type Params = u32;

    fn build(builder: &mut EntityBuilder<'_, Self>, params: Self::Params) -> Built {
        builder.with(params).with_self(Self)
    }
}

struct Text {
    update_id: u64,
    group_id: u32,
}

impl EntityMainComponent for Text {
    type Params = (u32, u32);

    fn build(builder: &mut EntityBuilder<'_, Self>, params: Self::Params) -> Built {
        builder
            .inherit_from::<Id>(params.0)
            .with(format!("Update 0 for text entity with ID {}", params.0))
            .with_self(Self {
                update_id: 0,
                group_id: params.1,
            })
    }

    fn on_update(runner: &mut EntityRunner<'_, Self>) {
        runner
            .run(system!(Self::update_text))
            .run(system!(Self::replace_group))
            .run(system!(Self::delete_group))
            .run(system!(Self::add_entity))
            .run(system!(Self::delete_entity));
    }
}

impl Text {
    fn update_text(&mut self, text: &mut String, id: &u32) {
        self.update_id += 1;
        *text = format!("Update {} for text entity with ID {}", self.update_id, id);
    }

    fn delete_group(&self, id: &u32, mut group: Group<'_>) {
        if self.group_id == 1 && id == &2 {
            group.delete();
            println!("Group {} marked as deleted", self.group_id);
        }
    }

    fn replace_group(&self, id: &u32, mut group: Group<'_>) {
        if self.group_id == 2 && id == &4 {
            group.replace(|b| build_main_group(b, 2));
            println!("Group {} marked as replaced", self.group_id);
        }
    }

    fn add_entity(&self, id: &u32, mut group: Group<'_>) {
        if self.group_id == 0 && id == &0 && self.update_id == 1 {
            group.create_entity::<Self>((9999, self.group_id));
            println!("Entity 9999 created in group {}", self.group_id);
        }
    }

    fn delete_entity(id: &u32, mut entity: Entity<'_>) {
        if id == &1 {
            entity.delete();
            println!("Entity 1 deleted");
        }
    }
}

fn print_id(id: &u32) {
    println!("ID: {}", id);
}

fn print_text(text: &String) {
    println!("Text: {}", text);
}
