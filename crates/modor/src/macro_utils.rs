use crate::{Context, Node};

pub struct MaybeNode<'a, T>(pub &'a mut T);

impl<T> MaybeNode<'_, T>
where
    T: Node,
{
    #[inline]
    pub fn update(&mut self, ctx: &mut Context<'_>) {
        Node::update(self.0, ctx);
    }
}

pub trait NotNode {
    fn update(&mut self, ctx: &mut Context<'_>);
}

impl<T> NotNode for MaybeNode<'_, T> {
    #[inline]
    fn update(&mut self, _ctx: &mut Context<'_>) {
        // do nothing
    }
}
