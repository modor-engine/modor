use crate::{App, Node};

pub struct MaybeNode<'a, T>(pub &'a mut T);

impl<T> MaybeNode<'_, T>
where
    T: Node,
{
    #[inline]
    pub fn update(&mut self, app: &mut App) {
        Node::update(self.0, app);
    }
}

pub trait NotNode {
    fn update(&mut self, app: &mut App);
}

impl<T> NotNode for MaybeNode<'_, T> {
    #[inline]
    fn update(&mut self, _app: &mut App) {
        // do nothing
    }
}
