use modor::{App, Object, Role, RoleConstraint, UpdateContext};
use modor_derive::SingletonObject;
use std::any::TypeId;
use std::marker::PhantomData;

#[modor::test]
fn run_ordered_objects() {
    App::new()
        .create(|_| RoleTester::<Third>(PhantomData))
        .create(|_| RoleTester::<First>(PhantomData))
        .create(|_| RoleTester::<Second>(PhantomData))
        .create(|_| RoleCollector::default())
        .update()
        .for_each(1, |collector: &mut RoleCollector, _| {
            assert_eq!(
                collector.0,
                [
                    TypeId::of::<First>(),
                    TypeId::of::<Second>(),
                    TypeId::of::<Third>()
                ]
            );
            Ok(())
        });
}

#[modor::test]
fn run_ordered_objects_with_circular_dependency() {
    App::new()
        .create(|_| RoleTester::<ThirdCircular>(PhantomData))
        .create(|_| RoleTester::<FirstCircular>(PhantomData))
        .create(|_| RoleTester::<SecondCircular>(PhantomData))
        .create(|_| RoleCollector::default())
        .update()
        .for_each(1, |collector: &mut RoleCollector, _| {
            assert_eq!(collector.0.len(), 3);
            assert!(collector.0.contains(&TypeId::of::<FirstCircular>()));
            assert!(collector.0.contains(&TypeId::of::<SecondCircular>()));
            assert!(collector.0.contains(&TypeId::of::<ThirdCircular>()));
            Ok(())
        });
}

struct RoleTester<R>(PhantomData<R>);

impl<R> Object for RoleTester<R>
where
    R: Role,
{
    type Role = R;

    fn update(&mut self, ctx: &mut UpdateContext<'_>) -> modor::Result<()> {
        ctx.singleton_mut::<RoleCollector>()?
            .0
            .push(TypeId::of::<R>());
        Ok(())
    }
}

#[derive(Default, SingletonObject)]
struct RoleCollector(Vec<TypeId>);

struct First;

impl Role for First {
    fn constraints() -> Vec<RoleConstraint> {
        vec![]
    }
}

struct Second;

impl Role for Second {
    fn constraints() -> Vec<RoleConstraint> {
        vec![
            RoleConstraint::after::<First>(),
            RoleConstraint::before::<Third>(),
        ]
    }
}

struct Third;

impl Role for Third {
    fn constraints() -> Vec<RoleConstraint> {
        vec![]
    }
}

struct FirstCircular;

impl Role for FirstCircular {
    fn constraints() -> Vec<RoleConstraint> {
        vec![RoleConstraint::after::<ThirdCircular>()]
    }
}

struct SecondCircular;

impl Role for SecondCircular {
    fn constraints() -> Vec<RoleConstraint> {
        vec![RoleConstraint::after::<FirstCircular>()]
    }
}

struct ThirdCircular;

impl Role for ThirdCircular {
    fn constraints() -> Vec<RoleConstraint> {
        vec![RoleConstraint::after::<SecondCircular>()]
    }
}
