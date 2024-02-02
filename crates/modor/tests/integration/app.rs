use crate::{Level1, Level2, Level3, OtherLevel2};
use modor::{App, NoRole, Object, SingletonObject, UpdateContext};

#[modor::test]
fn create_object() {
    App::new()
        .create(Level1::new)
        .for_each::<Level1, _>(1, |_, _| ())
        .for_each::<Level2, _>(1, |_, _| ())
        .for_each::<OtherLevel2, _>(1, |_, _| ())
        .for_each::<Level3, _>(1, |_, _| ());
}

#[modor::test]
fn create_object_with_error() {
    App::new()
        .create(Level1::new_failed)
        .for_each::<Level1, _>(0, |_, _| ())
        .for_each::<Level2, _>(0, |_, _| ())
        .for_each::<OtherLevel2, _>(0, |_, _| ())
        .for_each::<Level3, _>(0, |_, _| ());
}

#[modor::test]
fn create_existing_singleton() {
    App::new()
        .create(|_| Singleton(0))
        .create(|_| Singleton(1))
        .for_each(1, |singleton: &mut Singleton, _| assert_eq!(singleton.0, 0));
}

#[modor::test]
fn assert_correct_object_count_with_for_each() {
    App::new()
        .create(|_| Level3(0))
        .create(|_| Level3(0))
        .create(|_| Level3(0))
        .for_each(3, |_: &mut Level3, _| ())
        .for_each(2..4, |_: &mut Level3, _| ())
        .for_each(.., |_: &mut Level3, _| ());
}

#[modor::test]
#[should_panic = "assertion failed: expected 4 objects of type integration::Level3, \
    3 objects found"]
fn assert_incorrect_object_count_with_for_each() {
    App::new()
        .create(|_| Level3(0))
        .create(|_| Level3(0))
        .create(|_| Level3(0))
        .for_each(4, |_: &mut Level3, _| ());
}

#[modor::test]
fn run_for_each() {
    let mut values = Vec::new();
    App::new()
        .create(|_| Level3(1))
        .create(|_| Level3(2))
        .create(|_| Level3(3))
        .for_each(3, |object: &mut Level3, ctx| {
            let value = object.0;
            ctx.create(Level3::new_failed);
            ctx.create(move |_| Level3(value + 10));
        })
        .for_each(6, |object: &mut Level3, _| values.push(object.0));
    assert_eq!(values, [1, 2, 3, 11, 12, 13]);
}

#[modor::test]
fn run_update() {
    let mut values = Vec::new();
    App::new()
        .create(|_| UpdatedObject(1))
        .create(|_| UpdatedObject(2))
        .create(|_| UpdatedObject(3))
        .create(|_| UpdatedSingletonObject(4))
        .update()
        .for_each(4, |object: &mut Level3, _| values.push(object.0));
    assert_eq!(values, [11, 12, 13, 14]);
}

#[derive(SingletonObject)]
struct MissingSingleton;

#[derive(SingletonObject)]
struct Singleton(u32);

struct UpdatedObject(u32);

impl Object for UpdatedObject {
    type Role = NoRole;

    fn update(&mut self, ctx: &mut UpdateContext<'_>) -> modor::Result<()> {
        let value = self.0;
        ctx.create(Level3::new_failed);
        ctx.create(move |_| Level3(value + 10));
        Ok(())
    }
}

struct UpdatedSingletonObject(u32);

impl SingletonObject for UpdatedSingletonObject {
    type Role = NoRole;

    fn update(&mut self, ctx: &mut UpdateContext<'_>) -> modor::Result<()> {
        let value = self.0;
        ctx.create(Level3::new_failed);
        ctx.create(move |_| Level3(value + 10));
        Ok(())
    }
}
