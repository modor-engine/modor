use crate::Context;
use std::collections::HashMap;
use std::ops::DerefMut;

pub trait RootNode: 'static + Node {
    fn on_create(ctx: &mut Context<'_>) -> Self;
}

pub trait Node: Visit {
    #[inline]
    #[allow(unused_variables)]
    fn on_enter(&mut self, ctx: &mut Context<'_>) {}

    #[inline]
    #[allow(unused_variables)]
    fn on_exit(&mut self, ctx: &mut Context<'_>) {}

    #[inline]
    fn update(&mut self, ctx: &mut Context<'_>) {
        self.on_enter(ctx);
        self.visit(ctx);
        self.on_exit(ctx);
    }
}

pub trait Visit {
    fn visit(&mut self, ctx: &mut Context<'_>);
}

impl Node for Box<dyn Node> {}

impl Visit for Box<dyn Node> {
    #[inline]
    fn visit(&mut self, ctx: &mut Context<'_>) {
        self.deref_mut().update(ctx);
    }
}

impl<T> Node for Option<T> where T: Node {}

impl<T> Visit for Option<T>
where
    T: Node,
{
    #[inline]
    fn visit(&mut self, ctx: &mut Context<'_>) {
        if let Some(node) = self {
            node.update(ctx);
        }
    }
}

impl<T> Node for Vec<T> where T: Node {}

impl<T> Visit for Vec<T>
where
    T: Node,
{
    fn visit(&mut self, ctx: &mut Context<'_>) {
        for node in self {
            node.update(ctx);
        }
    }
}

impl<K, V, S> Node for HashMap<K, V, S> where V: Node {}

impl<K, V, S> Visit for HashMap<K, V, S>
where
    V: Node,
{
    #[inline]
    fn visit(&mut self, ctx: &mut Context<'_>) {
        for node in self.values_mut() {
            node.update(ctx);
        }
    }
}
