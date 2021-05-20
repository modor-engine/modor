#![allow(clippy::print_stdout)]

use modor::*;

fn main() {
    let mut app = Application::new()
        .with_group(|b| build_main_group(b, 0))
        .with_group(|b| build_main_group(b, 1))
        .with_group(|b| build_main_group(b, 2))
        .with_group(|b| build_main_group(b, 3))
        .on_update(system!(print_id))
        .on_update(system!(print_id_for_entity_with_additional_component));
    println!("##### Update 1 #####");
    app.update();
    println!("##### Update 2 #####");
    app.update();
}

fn build_main_group(builder: &mut GroupBuilder<'_>, group_id: u32) {
    builder
        .with_entity::<Text>((group_id * 2, group_id))
        .with_entity::<Text>((group_id * 2 + 1, group_id))
        .with_entity::<Other>(())
        .on_update(system!(print_text));
}

struct AdditionalComponent;

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
            .with(String::from("Wrong text"))
            .with(format!("Update 0 for text entity with ID {}", params.0)) // erase previous
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
            .run(system!(Self::delete_entity))
            .run(system!(Self::add_component))
            .run(system!(Self::delete_existing_component))
            .run(system!(Self::delete_component_with_missing_type))
            .run(system!(Self::delete_missing_component));
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

    fn add_component(id: &u32, mut entity: Entity<'_>) {
        if id == &0 || id == &4 {
            entity.add_component(AdditionalComponent);
            println!("Component AdditionalComponent added for entity {}", id);
        }
    }

    fn delete_existing_component(id: &u32, mut entity: Entity<'_>) {
        if id == &6 {
            entity.delete_component::<Self>();
            println!("Component Text deleted for entity {}", id);
        }
    }

    fn delete_component_with_missing_type(id: &u32, mut entity: Entity<'_>) {
        if id == &6 {
            entity.delete_component::<i16>();
            println!("Nonexistent component i16 deleted for entity {}", id);
        }
    }

    fn delete_missing_component(id: &u32, mut entity: Entity<'_>) {
        if id == &6 {
            entity.delete_component::<Other>();
            println!("Not assigned component Other deleted for entity {}", id);
        }
    }
}

struct Other;

impl EntityMainComponent for Other {
    type Params = ();

    fn build(builder: &mut EntityBuilder<'_, Self>, _params: Self::Params) -> Built {
        builder.with_self(Self)
    }
}

fn print_id(id: &u32) {
    println!(">>> ID: {}", id);
}

fn print_text(_: &Text, id: &u32, text: &String) {
    println!(">>> Text with ID {}: {}", id, text);
}

fn print_id_for_entity_with_additional_component(
    id: &u32,
    _additional_component: &AdditionalComponent,
) {
    println!(">>> Entity with additional component: {}", id);
}
