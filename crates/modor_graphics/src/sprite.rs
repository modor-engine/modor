use crate::{DefaultMaterial2D, IntoMat, Mat, Model2D};
use modor::{Builder, Context, Node, Visit};

/// A rendered 2D object that can be colored or textured.
///
/// This `struct` is used to simplify the creation of a [`Model2D`] with a dedicated material.
///
/// # Examples
///
/// See [`Texture`](crate::Texture).
#[non_exhaustive]
#[derive(Debug, Node, Visit, Builder)]
pub struct Sprite2D {
    /// Material of the sprite, i.e. the aspect.
    #[builder(form(closure))]
    pub material: Mat<DefaultMaterial2D>,
    /// Model of the sprite, i.e. where the sprite is rendered.
    #[builder(form(closure))]
    pub model: Model2D<DefaultMaterial2D>,
}

impl Sprite2D {
    /// Creates a new sprite.
    ///
    /// The `label` is used to identity the material in logs.
    pub fn new(ctx: &mut Context<'_>, label: impl Into<String>) -> Self {
        let material = DefaultMaterial2D::new(ctx).into_mat(ctx, label);
        let model = Model2D::new(ctx, material.glob());
        Self { material, model }
    }
}
