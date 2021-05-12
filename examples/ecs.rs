#![allow(clippy::print_stdout)]

use modor::*;

fn main() {
    let mut app = Application::default()
        .with_group(|b| build_main_group(b, 0))
        .with_group(|b| build_main_group(b, 100))
        .with_group(|b| build_main_group(b, 1000))
        .on_update(system!(print_strings_global));
    println!("Update 1:");
    app.update();
    println!("Update 2:");
    app.update();
}

fn build_main_group(builder: &mut GroupBuilder<'_>, offset: u32) {
    builder
        .with_entity::<ChildEntity>((42 + offset, offset))
        .with_entity::<ChildEntity>((13 + offset, offset))
        .with_entity::<ChildEntity>((15 + offset, offset))
        .on_update(system!(print_strings));
}

struct ChildEntity(u32, u32);

impl EntityMainComponent for ChildEntity {
    type Params = (u32, u32);

    fn build(builder: &mut EntityBuilder<'_, Self>, value: Self::Params) -> Built {
        builder
            .inherit_from::<ParentEntity>(value.0 + 10)
            .with(format!("string {:?}", value))
            .with_self(Self(value.0, value.1))
    }

    fn on_update(runner: &mut EntityRunner<'_, Self>) {
        runner.run(system!(Self::print_value));
        runner.run(system!(Self::delete_group));
    }
}

impl ChildEntity {
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn print_value(&self, parent_value: &u32) {
        println!("Value: {}, parent: {}", self.0, parent_value);
    }

    fn delete_group(&self, mut group: Group<'_>) {
        if self.1 == 100 {
            group.delete();
        }
    }
}

struct ParentEntity;

impl EntityMainComponent for ParentEntity {
    type Params = u32;

    fn build(builder: &mut EntityBuilder<'_, Self>, value: Self::Params) -> Built {
        builder.with(value).with_self(Self)
    }
}

#[allow(clippy::ptr_arg)]
fn print_strings(string: &String) {
    println!("String: {}", string);
}

#[allow(clippy::ptr_arg)]
fn print_strings_global(string: &String) {
    println!("String global: {}", string);
}
