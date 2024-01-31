use modor::{App, BuildContext};
use modor_derive::{Object, SingletonObject};

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

// TODO: continue

#[derive(Object)]
struct Level1;

impl Level1 {
    fn new(ctx: &mut BuildContext<'_>) -> Self {
        ctx.create(|_| OtherLevel2);
        ctx.create(Level2::new);
        Self
    }

    fn new_failed(ctx: &mut BuildContext<'_>) -> Self {
        ctx.create(|_| OtherLevel2);
        ctx.create(Level2::new_failed);
        Self
    }
}

#[derive(Object)]
struct Level2;

impl Level2 {
    fn new(ctx: &mut BuildContext<'_>) -> Self {
        ctx.create(|_| Level3);
        Self
    }

    fn new_failed(ctx: &mut BuildContext<'_>) -> Self {
        ctx.create(Level3::new_failed);
        Self
    }
}

#[derive(Object)]
struct OtherLevel2;

#[derive(Object)]
struct Level3;

impl Level3 {
    fn new_failed(ctx: &mut BuildContext<'_>) -> modor::Result<Self> {
        ctx.singleton::<MissingSingleton>()?;
        Ok(Self)
    }
}

#[derive(SingletonObject)]
struct MissingSingleton;

#[derive(SingletonObject)]
struct Singleton(u32);
