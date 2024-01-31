use modor::{App, BuildContext, DynId, Error, Id};
use modor_derive::Object;

#[modor::test]
fn retrieve_immutable_object_from_id() {
    setup_app().for_each(1, |parent: &mut Parent, ctx| {
        assert!(parent.existing.get(ctx).is_ok());
        assert!(matches!(
            parent.missing.get(ctx),
            Err(Error::ObjectNotFound(_))
        ));
        ctx.lock_objects::<Child>(|ctx, _| {
            assert!(matches!(
                parent.existing.get(ctx),
                Err(Error::ObjectTypeAlreadyLocked(_))
            ));
            assert!(matches!(
                parent.missing.get(ctx),
                Err(Error::ObjectTypeAlreadyLocked(_))
            ));
            Ok(())
        })?;
        Ok(())
    });
}

#[modor::test]
fn retrieve_mutable_object_from_id() {
    setup_app().for_each(1, |parent: &mut Parent, ctx| {
        assert!(parent.existing.get_mut(ctx).is_ok());
        assert!(matches!(
            parent.missing.get_mut(ctx),
            Err(Error::ObjectNotFound(_))
        ));
        ctx.lock_objects::<Child>(|ctx, _| {
            assert!(matches!(
                parent.existing.get_mut(ctx),
                Err(Error::ObjectTypeAlreadyLocked(_))
            ));
            assert!(matches!(
                parent.missing.get_mut(ctx),
                Err(Error::ObjectTypeAlreadyLocked(_))
            ));
            Ok(())
        })?;
        Ok(())
    });
}

#[modor::test]
fn retrieve_object_index() {
    setup_app().for_each(1, |parent: &mut Parent, _ctx| {
        assert_eq!(usize::from(parent.existing), 0);
        assert_eq!(usize::from(parent.missing), 1);
        Ok(())
    });
}

#[modor::test]
fn obtain_dynamic_id() {
    setup_app().for_each(1, |parent: &mut Parent, _ctx| {
        let dyn_id = DynId::from(parent.existing);
        assert_eq!(usize::from(dyn_id), usize::from(parent.existing));
        assert!(dyn_id.typed::<Child>().is_ok());
        assert!(matches!(
            dyn_id.typed::<Parent>(),
            Err(Error::InvalidIdType(_))
        ));
        assert_eq!(format!("{dyn_id}"), format!("{}", parent.existing));
        Ok(())
    });
}

#[modor::test]
fn display_id() {
    setup_app().for_each(1, |parent: &mut Parent, _ctx| {
        assert_eq!(format!("{}", parent.existing), "0|0");
        assert_eq!(format!("{}", parent.missing), "1|0");
        Ok(())
    });
}

fn setup_app() -> App {
    let mut app = App::new();
    app.create(Parent::new)
        .for_each(1, |parent: &mut Parent, ctx| ctx.delete(parent.missing));
    app
}

#[derive(Object)]
struct Parent {
    existing: Id<Child>,
    missing: Id<Child>,
}

impl Parent {
    fn new(ctx: &mut BuildContext<'_>) -> Self {
        Self {
            existing: ctx.create(|_| Child),
            missing: ctx.create(|_| Child),
        }
    }
}

#[derive(Object)]
struct Child;
