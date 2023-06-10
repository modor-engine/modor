//! Testing utilities.

use crate::{Texture, TextureBuffer};
use image::ColorType;
use modor_resources::{Resource, ResourceState};
use std::path::PathBuf;
use std::time::Duration;
use std::{env, fs, thread};

pub use crate::platform::testing::*;
pub use crate::runner::testing::*;

/// The maximum allowed difference between two textures.
///
/// # Examples
///
/// See [`assert_texture`](assert_texture()).
pub enum MaxTextureDiff {
    /// No difference is allowed.
    Zero,
    /// The components of each RGBA pixel must not have a difference higher than the passed value.
    Component(u8),
    /// The number of different pixels must not be higher than the passed value.
    PixelCount(usize),
}

/// Asserts the [`TextureBuffer`](TextureBuffer) is similar to the expected texture.
///
/// If the expected texture is not yet generated, it is saved in
/// `$CARGO_MANIFEST_DIR/tests/expected/{key}.png` and the function panics. At the next function run,
/// the function shouldn't panic if the actual texture is similar to the expected one.
///
/// If the difference between expected and actual texture is higher than `max_diff`, then they are
/// not considered as similar. In this case, a diff texture is saved in a temporary folder and
/// the function panics with a message containing the path to the diff texture.
///
/// The generated diff texture is a black texture, with white color for pixels that are different.
///
/// # Panics
///
/// This will panic if:
/// - the [`TextureBuffer`](TextureBuffer) buffer is empty.
/// - the actual texture found in the [`TextureBuffer`](TextureBuffer) is not similar to the
/// expected one saved in `$CARGO_MANIFEST_DIR/tests/expected/{key}.png`.
/// - there is an I/O error while reading or writing the expected or the diff texture.
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// # use modor_graphics_new2::*;
/// # use modor_graphics_new2::testing::*;
/// #
/// # fn f() {
/// #[derive(Clone, Debug, PartialEq, Eq, Hash)]
/// struct TextureKey;
///
/// fn texture() -> impl BuiltEntity {
///     EntityBuilder::new()
///         .with(Texture::from_path(TextureKey, "image.png"))
///         .with(TextureBuffer::default())
/// }
///
/// App::new()
///     .with_entity(texture())
///     .updated_until_all::<With<Texture>, _>(Some(100), wait_texture_loading)
///     .assert::<With<TextureBuffer>>(1, |e| {
///         e.has(|b| assert_texture(b, "texture", MaxTextureDiff::Zero))
///     });
/// # }
/// ```
pub fn assert_texture(buffer: &TextureBuffer, key: &str, max_diff: MaxTextureDiff) {
    let data = buffer.get();
    let size = buffer.size();
    assert_ne!(size.width * size.height, 0, "texture buffer is empty");
    let expected_folder = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/expected");
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

/// Returns whether the texture is loaded, and sleeps 10ms if not yet loaded.
///
/// The texture is considered as loaded if the state is [`ResourceState::Loaded`] or
/// [`ResourceState::Error`].
///
/// # Platform-specific
///
/// - Web: sleep is not supported, so the function panics.
///
/// # Examples
///
/// See [`assert_texture`](assert_texture()).
pub fn wait_texture_loading(texture: &Texture) -> bool {
    if matches!(
        texture.state(),
        ResourceState::Loaded | ResourceState::Error(_)
    ) {
        true
    } else {
        thread::sleep(Duration::from_micros(10));
        false
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
