#![allow(clippy::missing_panics_doc, clippy::unwrap_used)]

#[macro_use]
extern crate modor;

use modor::{EntityAssertions, EntityFilter};
use modor_graphics_new2::testing::{assert_texture, MaxTextureDiff};
use modor_graphics_new2::TextureBuffer;

#[modor_test(disabled(macos, android, wasm))]
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
    |e| e.has(|b: &TextureBuffer| assert_texture(b, key, MaxTextureDiff::Zero))
}

pub mod color;
pub mod input;
pub mod material;
pub mod model;
pub mod testing;
pub mod texture_buffer;
pub mod window;

/*
TODO: add missing tests
    - material
    - texture
    - z_index
    - camera
    - render_target (target in target)
 */
