use modor::BuildContext;
use modor_derive::{Object, SingletonObject};

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
        ctx.create(|_| Level3(0));
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
struct Level3(u32);

impl Level3 {
    fn new_failed(ctx: &mut BuildContext<'_>) -> modor::Result<Self> {
        ctx.singleton::<MissingSingleton>()?;
        Ok(Self(0))
    }
}

#[derive(SingletonObject)]
struct MissingSingleton;

pub mod app;
pub mod context;
pub mod id;
pub mod objects;
pub mod ranges;
pub mod roles;
