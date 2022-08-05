use std::any::Any;

pub trait CollisionGroup: Sized + Any {
    fn index(self) -> usize;

    fn layers() -> Vec<CollisionLayer<Self>>;
}

impl CollisionGroup for () {
    fn index(self) -> usize {
        0
    }

    fn layers() -> Vec<CollisionLayer<Self>> {
        vec![]
    }
}

pub struct CollisionLayer<G> {
    pub(crate) groups: Vec<G>,
}

impl<G> CollisionLayer<G> {
    // If 2D and 3D shapes in same layer, then they will not collide
    pub const fn new(groups: Vec<G>) -> Self {
        Self { groups }
    }
}
