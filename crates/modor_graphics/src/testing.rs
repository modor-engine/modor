//! Testing utilities.

use crate::TextureGlob;
use image::imageops::FilterType;
use image::{ColorType, ImageBuffer, Rgba};
use modor::{App, GlobRef};
use std::path::PathBuf;
use std::{env, fs};

/// Asserts a [`Texture`](crate::Texture) buffer is the same as the expected texture.
///
/// If the expected texture is not yet generated, it is saved in
/// `$CARGO_MANIFEST_DIR/tests/expected/{key}.png` and the function panics. At the next function
/// run, the function shouldn't panic if the actual texture is similar to the expected one.
///
/// If there is a difference between expected and actual texture, a diff texture is saved in a
/// temporary folder and the function panics with a message containing the path to the diff
/// texture.
///
/// The generated diff texture is a black texture, with white color for pixels that are
/// different.
///
/// # Panics
///
/// This will panic if:
/// - the expected and actual textures are different.
/// - the [`Texture`](crate::Texture) buffer is empty.
/// - the actual texture found in the [`Texture`](crate::Texture) doesn't match the
/// expected one saved in `$CARGO_MANIFEST_DIR/tests/expected/{key}.png`.
/// - there is an I/O error while reading or writing the expected or the diff texture.
///
/// # Examples
///
/// ```rust
/// # use log::*;
/// # use modor::*;
/// # use modor_graphics::*;
/// # use modor_graphics::testing::*;
/// # use modor_resources::*;
/// #
/// # fn no_run() {
/// let mut app = App::new::<Root>(Level::Info);
/// let texture = app.get_mut::<Root>().texture.glob().clone();
/// assert_same(&mut app, &texture, "expected_texture");
///
/// #[derive(Node, Visit)]
/// struct Root {
///     texture: Res<Texture>,
/// }
///
/// impl RootNode for Root {
///     fn on_create(ctx: &mut Context<'_>) -> Self {
///         let mut texture = 
///             Res::<Texture>::from_source(ctx, "texture", TextureSource::Size(Size::new(10, 10)));
///         texture.is_buffer_enabled = true;
///         Self { texture }
///     }
/// }
/// # }
/// ```
pub fn assert_same(app: &mut App, texture: &GlobRef<TextureGlob>, key: impl AsRef<str>) {
    assert_texture(app, texture, key.as_ref(), MaxTextureDiff::Zero);
}

/// Asserts a [`Texture`](crate::Texture) buffer is similar to the expected texture
/// at component level.
///
/// If the expected texture is not yet generated, it is saved in
/// `$CARGO_MANIFEST_DIR/tests/expected/{key}.png` and the function panics. At the next function
/// run, the function shouldn't panic if the actual texture is similar to the expected one.
///
/// If at least one of the pixel components has a difference greater than `max_component_diff`
/// between expected and actual texture, a diff texture is saved in a temporary folder and the
/// function panics with a message containing the path to the diff texture.
///
/// The generated diff texture is a black texture, with white color for pixels that are
/// different.
///
/// The images are downscaled at a factor of `downscale_factor` using linear filtering
/// before being compared.
///
/// # Panics
///
/// This will panic if:
/// - the expected and actual textures are not similar.
/// - the [`Texture`](crate::Texture) buffer is empty.
/// - the actual texture found in the [`Texture`](crate::Texture) doesn't match the
/// expected one saved in `$CARGO_MANIFEST_DIR/tests/expected/{key}.png`.
/// - there is an I/O error while reading or writing the expected or the diff texture.
///
/// # Examples
///
/// ```rust
/// # use log::*;
/// # use modor::*;
/// # use modor_graphics::*;
/// # use modor_graphics::testing::*;
/// # use modor_resources::*;
/// #
/// # fn no_run() {
/// let mut app = App::new::<Root>(Level::Info);
/// let texture = app.get_mut::<Root>().texture.glob().clone();
/// assert_max_component_diff(&mut app, &texture, "expected_texture", 1, 1);
///
/// #[derive(Node, Visit)]
/// struct Root {
///     texture: Res<Texture>,
/// }
///
/// impl RootNode for Root {
///     fn on_create(ctx: &mut Context<'_>) -> Self {
///         let mut texture = 
///             Res::<Texture>::from_source(ctx, "texture", TextureSource::Size(Size::new(10, 10)));
///         texture.is_buffer_enabled = true;
///         Self { texture }
///     }
/// }
/// # }
/// ```
pub fn assert_max_component_diff(
    app: &mut App,
    texture: &GlobRef<TextureGlob>,
    key: impl AsRef<str>,
    max_component_diff: u8,
    downscale_factor: u8,
) {
    assert_texture(
        app,
        texture,
        key.as_ref(),
        MaxTextureDiff::Component(max_component_diff, downscale_factor),
    );
}

/// Asserts a [`Texture`](crate::Texture) buffer is similar to the expected texture
/// at pixel level.
///
/// If the expected texture is not yet generated, it is saved in
/// `$CARGO_MANIFEST_DIR/tests/expected/{key}.png` and the function panics. At the next function
/// run, the function shouldn't panic if the actual texture is similar to the expected one.
///
/// If more than `max_pixel_count_diff` pixels are different between expected and actual
/// textures, a diff texture is saved in a temporary folder and the function panics with a
/// message containing the path to the diff texture.
///
/// The generated diff texture is a black texture, with white color for pixels that are
/// different.
///
/// # Panics
///
/// This will panic if:
/// - the expected and actual textures are not similar.
/// - the [`Texture`](crate::Texture) buffer is empty.
/// - the actual texture found in the [`Texture`](crate::Texture) doesn't match the
/// expected one saved in `$CARGO_MANIFEST_DIR/tests/expected/{key}.png`.
/// - there is an I/O error while reading or writing the expected or the diff texture.
///
/// # Examples
///
/// ```rust
/// # use log::*;
/// # use modor::*;
/// # use modor_graphics::*;
/// # use modor_graphics::testing::*;
/// # use modor_resources::*;
/// #
/// # fn no_run() {
/// let mut app = App::new::<Root>(Level::Info);
/// let texture = app.get_mut::<Root>().texture.glob().clone();
/// assert_max_pixel_diff(&mut app, &texture, "expected_texture", 10);
///
/// #[derive(Node, Visit)]
/// struct Root {
///     texture: Res<Texture>,
/// }
///
/// impl RootNode for Root {
///     fn on_create(ctx: &mut Context<'_>) -> Self {
///         let mut texture = 
///             Res::<Texture>::from_source(ctx, "texture", TextureSource::Size(Size::new(10, 10)));
///         texture.is_buffer_enabled = true;
///         Self { texture }
///     }
/// }
/// # }
/// ```
pub fn assert_max_pixel_diff(
    app: &mut App,
    texture: &GlobRef<TextureGlob>,
    key: impl AsRef<str>,
    max_pixel_count_diff: usize,
) {
    assert_texture(
        app,
        texture,
        key.as_ref(),
        MaxTextureDiff::PixelCount(max_pixel_count_diff),
    );
}

fn assert_texture(
    app: &mut App,
    texture: &GlobRef<TextureGlob>,
    key: &str,
    max_diff: MaxTextureDiff,
) {
    let ctx = app.ctx();
    let glob = texture.get(&ctx);
    let data = glob.buffer(&ctx);
    let size = glob.size;
    assert!(!data.is_empty(), "texture buffer is empty");
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
        if !are_texture_similar(&data, &expected_data, expected_width, max_diff) {
            let diff_data = texture_diff(&data, &expected_data);
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
            &data,
            size.width,
            size.height,
            ColorType::Rgba8,
        )
        .expect("cannot save expected texture");
        panic!("expected texture saved, need to rerun the test");
    }
}

fn are_texture_similar(
    texture1: &[u8],
    texture2: &[u8],
    width: u32,
    max_diff: MaxTextureDiff,
) -> bool {
    match max_diff {
        MaxTextureDiff::Zero => texture1 == texture2,
        MaxTextureDiff::Component(epsilon, factor) => !downscaled_texture(texture1, width, factor)
            .into_iter()
            .zip(downscaled_texture(texture2, width, factor))
            .any(|(a, b)| a.abs_diff(b) > epsilon),
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

#[allow(clippy::cast_possible_truncation)]
fn downscaled_texture(texture: &[u8], width: u32, factor: u8) -> Vec<u8> {
    let height = (texture.len() as u32).div_euclid(4 * width);
    let buffer: ImageBuffer<Rgba<u8>, _> =
        ImageBuffer::from_raw(width, height, texture).expect("cannot downscale image");
    image::imageops::resize(
        &buffer,
        width.div_euclid(u32::from(factor)),
        height.div_euclid(u32::from(factor)),
        FilterType::Triangle,
    )
    .into_raw()
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
    Component(u8, u8), // component diff, downscale factor
    PixelCount(usize),
}
