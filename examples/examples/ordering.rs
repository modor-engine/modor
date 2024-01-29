#![allow(missing_docs)]

use modor::log::LevelFilter;
use modor::{App, Context, Object, Role, RoleConstraint};

#[modor::main]
fn main() {
    App::new()
        .set_log_level(LevelFilter::Trace)
        .create(|_| A)
        .create(|_| B)
        .create(|_| C)
        .update();
}

struct First;

impl Role for First {
    fn constraints() -> Vec<RoleConstraint> {
        vec![RoleConstraint::before::<Second>()]
    }
}

struct Second;

impl Role for Second {
    fn constraints() -> Vec<RoleConstraint> {
        vec![RoleConstraint::after::<First>()]
    }
}

struct A;

impl Object for A {
    type Role = First;

    fn update(&mut self, _ctx: &mut Context<'_, Self>) -> modor::Result<()> {
        Ok(())
    }
}

struct B;

impl Object for B {
    type Role = Second;

    fn update(&mut self, _ctx: &mut Context<'_, Self>) -> modor::Result<()> {
        Ok(())
    }
}

struct C;

impl Object for C {
    type Role = First;

    fn update(&mut self, _ctx: &mut Context<'_, Self>) -> modor::Result<()> {
        Ok(())
    }
}
