//! Testing utilities.

use crate::TextureBuffer;
use image::ColorType;
use modor::{EntityAssertions, EntityFilter};
use std::path::PathBuf;
use std::{env, fs};

pub use crate::platform::testing::*;
pub use crate::runner::testing::*;

/// Asserts the [`TextureBuffer`](TextureBuffer) is the same as the expected texture.
///
/// If the expected texture is not yet generated, it is saved in
/// `$CARGO_MANIFEST_DIR/tests/expected/{key}.png` and the function panics. At the next function run,
/// the function shouldn't panic if the actual texture is similar to the expected one.
///
/// If there is a difference between expected and actual texture, a diff texture is saved in a
/// temporary folder and the function panics with a message containing the path to the diff texture.
///
/// The generated diff texture is a black texture, with white color for pixels that are different.
///
/// # Panics
///
/// This will panic if:
/// - the expected and actual textures are different.
/// - the [`TextureBuffer`](TextureBuffer) buffer is empty.
/// - the actual texture found in the [`TextureBuffer`](TextureBuffer) is not similar to the
/// expected one saved in `$CARGO_MANIFEST_DIR/tests/expected/{key}.png`.
/// - there is an I/O error while reading or writing the expected or the diff texture.
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// # use modor_graphics::*;
/// # use modor_graphics::testing::*;
/// # use modor_resources::*;
/// #
/// # fn f() {
/// use modor_resources::testing::wait_resource_loading;
///
/// const TEXTURE: ResKey<Texture> = ResKey::new("texture");
///
/// fn texture() -> impl BuiltEntity {
///     EntityBuilder::new()
///         .with(Texture::from_path(TEXTURE, "image.png"))
///         .with(TextureBuffer::default())
/// }
///
/// App::new()
///     .with_entity(texture())
///     .updated_until_all::<With<Texture>, Texture>(Some(100), wait_resource_loading)
///     .assert::<With<TextureBuffer>>(1, is_same("texture"));
/// # }
/// ```
pub fn is_same<F>(key: &str) -> impl FnMut(EntityAssertions<'_, F>) -> EntityAssertions<'_, F> + '_
where
    F: EntityFilter,
{
    |e| e.has(|b: &TextureBuffer| assert_texture(b, key, MaxTextureDiff::Zero))
}

/// Asserts the [`TextureBuffer`](TextureBuffer) is similar to the expected texture
/// at component level.
///
/// If the expected texture is not yet generated, it is saved in
/// `$CARGO_MANIFEST_DIR/tests/expected/{key}.png` and the function panics. At the next function run,
/// the function shouldn't panic if the actual texture is similar to the expected one.
///
/// If at least one of the pixel components has a difference greater than `max_component_diff`
/// between expected and actual texture, a diff texture is saved in a temporary folder and the
/// function panics with a message containing the path to the diff texture.
///
/// The generated diff texture is a black texture, with white color for pixels that are different.
///
/// # Panics
///
/// This will panic if:
/// - the expected and actual textures are not similar.
/// - the [`TextureBuffer`](TextureBuffer) buffer is empty.
/// - the actual texture found in the [`TextureBuffer`](TextureBuffer) is not similar to the
/// expected one saved in `$CARGO_MANIFEST_DIR/tests/expected/{key}.png`.
/// - there is an I/O error while reading or writing the expected or the diff texture.
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// # use modor_graphics::*;
/// # use modor_graphics::testing::*;
/// # use modor_resources::*;
/// #
/// # fn f() {
/// use modor_resources::testing::wait_resource_loading;
///
/// const TEXTURE: ResKey<Texture> = ResKey::new("texture");
///
/// fn texture() -> impl BuiltEntity {
///     EntityBuilder::new()
///         .with(Texture::from_path(TEXTURE, "image.png"))
///         .with(TextureBuffer::default())
/// }
///
/// App::new()
///     .with_entity(texture())
///     .updated_until_all::<With<Texture>, Texture>(Some(100), wait_resource_loading)
///     .assert::<With<TextureBuffer>>(1, has_component_diff("texture", 1));
/// # }
/// ```
pub fn has_component_diff<F>(
    key: &str,
    max_component_diff: u8,
) -> impl FnMut(EntityAssertions<'_, F>) -> EntityAssertions<'_, F> + '_
where
    F: EntityFilter,
{
    move |e| {
        e.has(|b: &TextureBuffer| {
            assert_texture(b, key, MaxTextureDiff::Component(max_component_diff));
        })
    }
}

/// Asserts the [`TextureBuffer`](TextureBuffer) is similar to the expected texture
/// at pixel level.
///
/// If the expected texture is not yet generated, it is saved in
/// `$CARGO_MANIFEST_DIR/tests/expected/{key}.png` and the function panics. At the next function run,
/// the function shouldn't panic if the actual texture is similar to the expected one.
///
/// If more than `max_pixel_count_diff` pixels are different between expected and actual textures,
/// a diff texture is saved in a temporary folder and the function panics with a message
/// containing the path to the diff texture.
///
/// The generated diff texture is a black texture, with white color for pixels that are different.
///
/// # Panics
///
/// This will panic if:
/// - the expected and actual textures are not similar.
/// - the [`TextureBuffer`](TextureBuffer) buffer is empty.
/// - the actual texture found in the [`TextureBuffer`](TextureBuffer) is not similar to the
/// expected one saved in `$CARGO_MANIFEST_DIR/tests/expected/{key}.png`.
/// - there is an I/O error while reading or writing the expected or the diff texture.
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// # use modor_graphics::*;
/// # use modor_resources::*;
/// # use modor_graphics::testing::*;
/// #
/// # fn f() {
/// use modor_resources::testing::wait_resource_loading;
///
/// const TEXTURE: ResKey<Texture> = ResKey::new("texture");
///
/// #[derive(Clone, Debug, PartialEq, Eq, Hash)]
/// struct TextureKey;
///
/// fn texture() -> impl BuiltEntity {
///     EntityBuilder::new()
///         .with(Texture::from_path(TEXTURE, "image.png"))
///         .with(TextureBuffer::default())
/// }
///
/// App::new()
///     .with_entity(texture())
///     .updated_until_all::<With<Texture>, Texture>(Some(100), wait_resource_loading)
///     .assert::<With<TextureBuffer>>(1, has_pixel_diff("texture", 10));
/// # }
/// ```
pub fn has_pixel_diff<F>(
    key: &str,
    max_pixel_count_diff: usize,
) -> impl FnMut(EntityAssertions<'_, F>) -> EntityAssertions<'_, F> + '_
where
    F: EntityFilter,
{
    move |e| {
        e.has(|b: &TextureBuffer| {
            assert_texture(b, key, MaxTextureDiff::PixelCount(max_pixel_count_diff));
        })
    }
}

fn assert_texture(buffer: &TextureBuffer, key: &str, max_diff: MaxTextureDiff) {
    let data = buffer.get();
    let size = buffer.size();
    assert_ne!(size.width * size.height, 0, "texture buffer is empty");
    let expected_folder = env::var("CARGO_MANIFEST_DIR")
        .expect("`CARGO_MANIFEST_DIR` environment variable not set")
        + "/tests/expected";
    let expected_file: PathBuf = format!("{expected_folder}/{key}.png").into();
    if expected_file.exists() {
        let image = image::open(&expected_file).expect("cannot read expected texture from disk");
        let expected_width = image.width();
        let expected_height = image.height();
        let expected_data = image.to_rgba8().into_raw();
        assert_eq!(size.width, expected_width, "texture width is different");
        assert_eq!(size.height, expected_height, "texture height is different");
        if !are_texture_similar(data, &expected_data, max_diff) {
            let diff_data = texture_diff(data, &expected_data);
            let diff_file = env::temp_dir().join(format!("diff_{key}.png"));
            image::save_buffer(
                &diff_file,
                &diff_data,
                size.width,
                size.height,
                ColorType::Rgba8,
            )
            .expect("cannot save texture diff");
            panic!("texture is different (diff saved in {diff_file:?})")
        }
    } else {
        fs::create_dir_all(expected_folder).expect("cannot create folder for expected texture");
        image::save_buffer(
            &expected_file,
            data,
            size.width,
            size.height,
            ColorType::Rgba8,
        )
        .expect("cannot save expected texture");
        panic!("expected texture saved, need to rerun the test");
    }
}

fn are_texture_similar(texture1: &[u8], texture2: &[u8], max_diff: MaxTextureDiff) -> bool {
    match max_diff {
        MaxTextureDiff::Zero => texture1 == texture2,
        MaxTextureDiff::Component(epsilon) => !texture1
            .iter()
            .zip(texture2)
            .any(|(a, b)| a.abs_diff(*b) > epsilon),
        MaxTextureDiff::PixelCount(pixel_count) => {
            texture1
                .chunks(4)
                .zip(texture2.chunks(4))
                .filter(|(a, b)| a != b)
                .count()
                <= pixel_count
        }
    }
}

fn texture_diff(texture1: &[u8], texture2: &[u8]) -> Vec<u8> {
    texture1
        .chunks(4)
        .zip(texture2.chunks(4))
        .flat_map(|(e, a)| {
            if a == e {
                [0, 0, 0, 255]
            } else {
                [255, 255, 255, 255]
            }
        })
        .collect()
}

enum MaxTextureDiff {
    Zero,
    Component(u8),
    PixelCount(usize),
}
