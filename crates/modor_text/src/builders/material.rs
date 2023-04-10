use crate::Text;
use modor::{BuiltEntity, EntityBuilder};
use modor_graphics_new2::{Material, Size, Texture, TextureSource};
use modor_resources::IntoResourceKey;

pub struct TextMaterialBuilder<K> {
    material_key: K,
    text: String,
    font_height: f32,
    material_fn: Option<Box<dyn FnOnce(Material) -> Material>>,
    texture_fn: Option<Box<dyn FnOnce(Texture) -> Texture>>,
    text_fn: Option<Box<dyn FnOnce(Text) -> Text>>,
}

impl<K> TextMaterialBuilder<K>
where
    K: IntoResourceKey + Clone,
{
    pub fn new(material_key: K, text: impl Into<String>, font_height: f32) -> Self {
        Self {
            material_key,
            text: text.into(),
            font_height,
            material_fn: None,
            texture_fn: None,
            text_fn: None,
        }
    }

    pub fn with_material(mut self, f: impl FnOnce(Material) -> Material + 'static) -> Self {
        self.material_fn = Some(Box::new(f));
        self
    }

    pub fn with_texture(mut self, f: impl FnOnce(Texture) -> Texture + 'static) -> Self {
        self.texture_fn = Some(Box::new(f));
        self
    }

    pub fn with_text(mut self, f: impl FnOnce(Text) -> Text + 'static) -> Self {
        self.text_fn = Some(Box::new(f));
        self
    }

    pub fn build(self) -> impl BuiltEntity {
        let material =
            Material::new(self.material_key.clone()).with_front_texture(self.material_key.clone());
        let texture = Texture::new(self.material_key, TextureSource::Size(Size::ONE));
        let text = Text::new(self.text, self.font_height);
        EntityBuilder::new()
            .with(if let Some(material_fn) = self.material_fn {
                material_fn(material)
            } else {
                material
            })
            .with(if let Some(texture_fn) = self.texture_fn {
                texture_fn(texture)
            } else {
                texture
            })
            .with(if let Some(text_fn) = self.text_fn {
                text_fn(text)
            } else {
                text
            })
    }
}
