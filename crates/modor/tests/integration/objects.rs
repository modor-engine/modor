use modor::rayon::prelude::*;
use modor::{App, BuildContext, Error, Id};
use modor_derive::{Object, SingletonObject};

#[modor::test]
fn retrieve_immutable_object_by_id() {
    setup_app().for_each(1, |root: &mut IntegerRoot, ctx| {
        let objects = ctx.objects_mut::<Integer>()?;
        assert!(matches!(
            objects.get(root.deleted_integer),
            Err(Error::ObjectNotFound(_))
        ));
        assert_eq!(objects.get(root.integer1)?.0, 1);
        assert_eq!(objects.get(root.integer2)?.0, 2);
        assert_eq!(objects.get(root.replaced_integer)?.0, 3);
        Ok(())
    });
}

#[modor::test]
fn retrieve_mutable_object_by_id() {
    setup_app().for_each(1, |root: &mut IntegerRoot, ctx| {
        let objects = ctx.objects_mut::<Integer>()?;
        assert!(matches!(
            objects.get_mut(root.deleted_integer),
            Err(Error::ObjectNotFound(_))
        ));
        assert_eq!(objects.get_mut(root.integer1)?.0, 1);
        assert_eq!(objects.get_mut(root.integer2)?.0, 2);
        assert_eq!(objects.get_mut(root.replaced_integer)?.0, 3);
        Ok(())
    });
}

#[modor::test]
fn retrieve_immutable_singleton() {
    setup_app().for_each(3, |_: &mut Integer, ctx| {
        let objects = ctx.objects_mut::<IntegerRoot>()?;
        assert!(objects.singleton().is_ok());
        let objects = ctx.objects_mut::<MissingSingleton>()?;
        assert!(matches!(
            objects.singleton(),
            Err(Error::SingletonObjectNotFound(_))
        ));
        Ok(())
    });
}

#[modor::test]
fn retrieve_mutable_singleton() {
    setup_app().for_each(3, |_: &mut Integer, ctx| {
        let objects = ctx.objects_mut::<IntegerRoot>()?;
        assert!(objects.singleton_mut().is_ok());
        let objects = ctx.objects_mut::<MissingSingleton>()?;
        assert!(matches!(
            objects.singleton_mut(),
            Err(Error::SingletonObjectNotFound(_))
        ));
        Ok(())
    });
}

#[modor::test]
fn iterate_on_immutable_objects() {
    setup_app().for_each(1, |_root: &mut IntegerRoot, ctx| {
        let objects = ctx.objects_mut::<Integer>()?;
        let result: Vec<_> = objects.iter().map(|int| int.0).collect();
        assert_eq!(result.len(), 3);
        assert!(result.contains(&1));
        assert!(result.contains(&2));
        assert!(result.contains(&3));
        Ok(())
    });
}

#[modor::test]
fn iterate_on_mutable_objects() {
    setup_app().for_each(1, |_root: &mut IntegerRoot, ctx| {
        let objects = ctx.objects_mut::<Integer>()?;
        let result: Vec<_> = objects.iter_mut().map(|int| int.0).collect();
        assert_eq!(result.len(), 3);
        assert!(result.contains(&1));
        assert!(result.contains(&2));
        assert!(result.contains(&3));
        Ok(())
    });
}

#[modor::test]
fn iterate_in_parallel_on_immutable_objects() {
    setup_app().for_each(1, |_root: &mut IntegerRoot, ctx| {
        let objects = ctx.objects_mut::<Integer>()?;
        let result: Vec<_> = objects.par_iter().map(|int| int.0).collect();
        assert_eq!(result.len(), 3);
        assert!(result.contains(&1));
        assert!(result.contains(&2));
        assert!(result.contains(&3));
        Ok(())
    });
}

#[modor::test]
fn iterate_in_parallel_on_mutable_objects() {
    setup_app().for_each(1, |_root: &mut IntegerRoot, ctx| {
        let objects = ctx.objects_mut::<Integer>()?;
        let result: Vec<_> = objects.par_iter_mut().map(|int| int.0).collect();
        assert_eq!(result.len(), 3);
        assert!(result.contains(&1));
        assert!(result.contains(&2));
        assert!(result.contains(&3));
        Ok(())
    });
}

#[modor::test]
fn iterate_on_immutable_objects_and_their_ids() {
    setup_app().for_each(1, |_root: &mut IntegerRoot, ctx| {
        let objects = ctx.objects_mut::<Integer>()?;
        let result: Vec<_> = objects
            .iter_enumerated()
            .map(|(id, int)| (usize::from(id), int.0))
            .collect();
        assert_eq!(result.len(), 3);
        assert!(result.contains(&(1, 1)));
        assert!(result.contains(&(2, 2)));
        assert!(result.contains(&(0, 3)));
        Ok(())
    });
}

#[modor::test]
fn iterate_on_mutable_objects_and_their_ids() {
    setup_app().for_each(1, |_root: &mut IntegerRoot, ctx| {
        let objects = ctx.objects_mut::<Integer>()?;
        let result: Vec<_> = objects
            .iter_mut_enumerated()
            .map(|(id, int)| (usize::from(id), int.0))
            .collect();
        assert_eq!(result.len(), 3);
        assert!(result.contains(&(1, 1)));
        assert!(result.contains(&(2, 2)));
        assert!(result.contains(&(0, 3)));
        Ok(())
    });
}

#[modor::test]
fn iterate_in_parallel_on_immutable_objects_and_their_ids() {
    setup_app().for_each(1, |_root: &mut IntegerRoot, ctx| {
        let objects = ctx.objects_mut::<Integer>()?;
        let result: Vec<_> = objects
            .par_iter_enumerated()
            .map(|(id, int)| (usize::from(id), int.0))
            .collect();
        assert_eq!(result.len(), 3);
        assert!(result.contains(&(1, 1)));
        assert!(result.contains(&(2, 2)));
        assert!(result.contains(&(0, 3)));
        Ok(())
    });
}

#[modor::test]
fn iterate_in_parallel_on_mutable_objects_and_their_ids() {
    setup_app().for_each(1, |_root: &mut IntegerRoot, ctx| {
        let objects = ctx.objects_mut::<Integer>()?;
        let result: Vec<_> = objects
            .par_iter_mut_enumerated()
            .map(|(id, int)| (usize::from(id), int.0))
            .collect();
        assert_eq!(result.len(), 3);
        assert!(result.contains(&(1, 1)));
        assert!(result.contains(&(2, 2)));
        assert!(result.contains(&(0, 3)));
        Ok(())
    });
}

fn setup_app() -> App {
    let mut app = App::new();
    app.create(IntegerRoot::new)
        .for_each(1, |root: &mut IntegerRoot, ctx| {
            ctx.delete(root.deleted_integer);
        })
        .for_each(1, |root: &mut IntegerRoot, ctx| {
            root.replaced_integer = ctx.create(|_| Integer(3));
        });
    app
}

#[derive(SingletonObject)]
struct MissingSingleton;

#[derive(SingletonObject)]
struct IntegerRoot {
    deleted_integer: Id<Integer>,
    integer1: Id<Integer>,
    integer2: Id<Integer>,
    replaced_integer: Id<Integer>,
}

impl IntegerRoot {
    fn new(ctx: &mut BuildContext<'_>) -> Self {
        let deleted_integer = ctx.create(|_| Integer(0));
        Self {
            deleted_integer,
            integer1: ctx.create(|_| Integer(1)),
            integer2: ctx.create(|_| Integer(2)),
            replaced_integer: deleted_integer,
        }
    }
}

#[derive(Object)]
struct Integer(u32);
