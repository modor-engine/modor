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
    pub(crate) ignore_z: bool,
}

impl<G> CollisionLayer<G> {
    pub const fn new_2d(groups: Vec<G>) -> Self {
        Self {
            groups,
            ignore_z: true,
        }
    }

    pub const fn new_3d(groups: Vec<G>) -> Self {
        Self {
            groups,
            ignore_z: false,
        }
    }
}
