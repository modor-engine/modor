use crate::{Level1, Level2, Level3, OtherLevel2};
use modor::{App, BuildContext, Error, NoRole, Object, UpdateContext};
use modor_derive::SingletonObject;

#[modor::test]
fn access_self_id() {
    App::new()
        .create(|_| SelfIdTester(0))
        .create(|_| SelfIdTester(1))
        .create(|_| SelfIdTester(2))
        .for_each(3, |tester: &mut SelfIdTester, ctx| {
            assert_eq!(tester.0, usize::from(ctx.self_id()));
        })
        .update();
}

#[modor::test]
fn retrieve_immutable_singleton() {
    App::new()
        .create(|_| Root)
        .create(|_| ExistingSingleton)
        .for_each(1, |_: &mut Root, ctx| {
            assert!(ctx.singleton::<ExistingSingleton>().is_ok());
            assert!(matches!(
                ctx.singleton::<MissingSingleton>(),
                Err(Error::SingletonObjectNotFound(_))
            ));
            assert!(matches!(
                ctx.singleton::<Root>(),
                Err(Error::ObjectTypeAlreadyLocked(_))
            ));
            Ok(())
        });
}

#[modor::test]
fn retrieve_mutable_singleton() {
    App::new()
        .create(|_| Root)
        .create(|_| ExistingSingleton)
        .for_each(1, |_: &mut Root, ctx| {
            assert!(ctx.singleton_mut::<ExistingSingleton>().is_ok());
            assert!(matches!(
                ctx.singleton_mut::<MissingSingleton>(),
                Err(Error::SingletonObjectNotFound(_))
            ));
            assert!(matches!(
                ctx.singleton_mut::<Root>(),
                Err(Error::ObjectTypeAlreadyLocked(_))
            ));
            Ok(())
        });
}

#[modor::test]
fn retrieve_immutable_objects() {
    App::new()
        .create(|_| Root)
        .create(|_| SelfIdTester(0))
        .create(|_| SelfIdTester(1))
        .create(|_| SelfIdTester(2))
        .for_each(1, |_: &mut Root, ctx| {
            let values: Vec<_> = ctx.objects::<SelfIdTester>()?.iter().map(|t| t.0).collect();
            assert_eq!(values, [0, 1, 2]);
            assert!(matches!(
                ctx.objects::<Root>(),
                Err(Error::ObjectTypeAlreadyLocked(_))
            ));
            Ok(())
        });
}

#[modor::test]
fn retrieve_mutable_objects() {
    App::new()
        .create(|_| Root)
        .create(|_| SelfIdTester(0))
        .create(|_| SelfIdTester(1))
        .create(|_| SelfIdTester(2))
        .for_each(1, |_: &mut Root, ctx| {
            let values: Vec<_> = ctx
                .objects_mut::<SelfIdTester>()?
                .iter()
                .map(|t| t.0)
                .collect();
            assert_eq!(values, [0, 1, 2]);
            assert!(matches!(
                ctx.objects_mut::<Root>(),
                Err(Error::ObjectTypeAlreadyLocked(_))
            ));
            Ok(())
        });
}

#[modor::test]
fn lock_object_type() {
    App::new()
        .create(|_| Root)
        .create(|_| SelfIdTester(0))
        .create(|_| SelfIdTester(1))
        .create(|_| SelfIdTester(2))
        .for_each(1, |_: &mut Root, ctx| {
            let mut done = false;
            ctx.lock_objects::<SelfIdTester, _>(|ctx, testers| {
                let values: Vec<_> = testers.iter().map(|t| t.0).collect();
                assert_eq!(values, [0, 1, 2]);
                assert!(matches!(
                    ctx.objects_mut::<SelfIdTester>(),
                    Err(Error::ObjectTypeAlreadyLocked(_))
                ));
                done = true;
            })?;
            assert!(done);
            Ok(())
        });
}

#[modor::test]
fn create_object() {
    App::new()
        .create(|_| Root)
        .for_each(1, |_: &mut Root, ctx| {
            assert_eq!(usize::from(ctx.create(Level1::new)), 0);
            assert_eq!(usize::from(ctx.create(Level1::new)), 1);
        })
        .for_each::<Level1, _>(2, |_, _| ())
        .for_each::<Level2, _>(2, |_, _| ())
        .for_each::<OtherLevel2, _>(2, |_, _| ())
        .for_each::<Level3, _>(2, |_, _| ());
}

#[modor::test]
fn create_object_with_error() {
    App::new()
        .create(|_| Root)
        .for_each(1, |_: &mut Root, ctx| {
            assert_eq!(usize::from(ctx.create(Level1::new_failed)), 0);
            assert_eq!(usize::from(ctx.create(Level1::new_failed)), 1);
        })
        .for_each::<Level1, _>(0, |_, _| ())
        .for_each::<Level2, _>(0, |_, _| ())
        .for_each::<OtherLevel2, _>(0, |_, _| ())
        .for_each::<Level3, _>(0, |_, _| ());
}

#[modor::test]
fn create_existing_singleton() {
    App::new()
        .create(|_| Root)
        .for_each(1, |_: &mut Root, ctx| {
            assert_eq!(usize::from(ctx.create(|_| Singleton(0))), 0);
            assert_eq!(usize::from(ctx.create(|_| Singleton(1))), 0);
        })
        .for_each(1, |singleton: &mut Singleton, _| assert_eq!(singleton.0, 0));
}

#[modor::test]
fn delete_object() {
    App::new()
        .create(|_| Root)
        .create(Level1::new)
        .for_each(1, |_: &mut Level1, ctx| ctx.delete(ctx.self_id()))
        .for_each::<Level1, _>(0, |_, _| ())
        .for_each::<Level2, _>(0, |_, _| ())
        .for_each::<OtherLevel2, _>(0, |_, _| ())
        .for_each::<Level3, _>(0, |_, _| ());
}

#[modor::test]
fn delete_missing_object() {
    let mut id = None;
    App::new()
        .create(|_| Root)
        .for_each(1, |_: &mut Root, ctx| {
            id = Some(ctx.create(Level1::new_failed));
        })
        .for_each(1, |_: &mut Root, ctx| {
            if let Some(id) = id {
                ctx.delete(id);
            } else {
                panic!("id not initialized");
            }
        });
}

#[modor::test]
fn delete_missing_singleton() {
    let mut id = None;
    App::new()
        .create(|_| Root)
        .for_each(1, |_: &mut Root, ctx| {
            id = Some(ctx.create(|ctx| MissingSingleton::new_failed(ctx)));
        })
        .for_each(1, |_: &mut Root, ctx| {
            if let Some(id) = id {
                ctx.delete(id);
            } else {
                panic!("id not initialized");
            }
        });
}

#[modor::test]
fn delete_self_object() {
    App::new()
        .create(|_| Root)
        .create(Level1::new)
        .for_each(1, |_: &mut Level1, ctx| ctx.delete_self())
        .for_each::<Level1, _>(0, |_, _| ())
        .for_each::<Level2, _>(0, |_, _| ())
        .for_each::<OtherLevel2, _>(0, |_, _| ())
        .for_each::<Level3, _>(0, |_, _| ());
}

#[derive(SingletonObject)]
struct Root;

struct SelfIdTester(usize);

impl Object for SelfIdTester {
    type Role = NoRole;

    fn update(&mut self, ctx: &mut UpdateContext<'_>) -> modor::Result<()> {
        assert_eq!(self.0, usize::from(ctx.self_id()));
        Ok(())
    }
}

#[derive(SingletonObject)]
struct MissingSingleton;

impl MissingSingleton {
    fn new_failed(ctx: &BuildContext<'_>) -> modor::Result<Self> {
        ctx.singleton::<Self>()?;
        Ok(Self)
    }
}

#[derive(SingletonObject)]
struct ExistingSingleton;

#[derive(SingletonObject)]
struct Singleton(u32);
