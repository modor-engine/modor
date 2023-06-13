#![allow(clippy::missing_panics_doc, clippy::unwrap_used)]

#[macro_use]
extern crate modor;

use modor::{EntityAssertions, EntityFilter};
use modor_graphics_new2::testing::{assert_texture, MaxTextureDiff};
use modor_graphics_new2::TextureBuffer;

#[modor_test(disabled(windows, macos, android, wasm))]
pub fn run_window_tests() {
    let mut context = modor_graphics_new2::testing::TestRunnerContext::default();
    window::run_window_tests(&mut context);
    input::run_window_tests(&mut context);
}

fn assert_exact_texture<F>(
    key: &str,
) -> impl FnMut(EntityAssertions<'_, F>) -> EntityAssertions<'_, F> + '_
where
    F: EntityFilter,
{
    |e| e.has(|b: &TextureBuffer| assert_texture(b, key, MaxTextureDiff::Component(1)))
}

fn assert_approx_texture<F>(
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

pub mod camera;
pub mod color;
pub mod input;
pub mod material;
pub mod model;
pub mod render_target;
pub mod testing;
pub mod texture;
pub mod texture_buffer;
pub mod window;
pub mod z_index;