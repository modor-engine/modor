use crate::{DefaultMaterial2D, MatGlob, Model2D};
use modor::{App, Builder, FromApp};

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
    pub material: MatGlob<DefaultMaterial2D>,
    /// Model of the sprite, i.e. where the sprite is rendered.
    #[builder(form(closure))]
    pub model: Model2D,
}

impl FromApp for Sprite2D {
    fn from_app(app: &mut App) -> Self {
        let material = MatGlob::from_app(app);
        Self {
            model: Model2D::new(app).with_material(material.to_ref()),
            material,
        }
    }
}

impl Sprite2D {
    /// Updates the sprite.
    pub fn update(&mut self, app: &mut App) {
        self.model.update(app);
    }
}
