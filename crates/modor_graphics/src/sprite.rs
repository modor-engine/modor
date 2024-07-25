use crate::{DefaultMaterial2D, IntoMat, Mat, Model2D};
use modor::{App, Builder, Node};

/// A rendered 2D object that can be colored or textured.
///
/// This `struct` is used to simplify the creation of a [`Model2D`] with a dedicated material.
///
/// # Examples
///
/// See [`Texture`](crate::Texture).
#[non_exhaustive]
#[derive(Debug, Builder)]
pub struct Sprite2D {
    /// Material of the sprite, i.e. the aspect.
    #[builder(form(closure))]
    pub material: Mat<DefaultMaterial2D>,
    /// Model of the sprite, i.e. where the sprite is rendered.
    #[builder(form(closure))]
    pub model: Model2D<DefaultMaterial2D>,
}

impl Node for Sprite2D {
    fn update(&mut self, app: &mut App) {
        self.material.update(app);
        self.model.update(app);
    }
}

impl Sprite2D {
    /// Creates a new sprite.
    pub fn new(app: &mut App) -> Self {
        let material = DefaultMaterial2D::new(app).into_mat(app);
        let model = Model2D::new(app, material.glob());
        Self { material, model }
    }
}
