#![allow(
    clippy::cast_precision_loss,
    clippy::print_stdout,
    missing_docs,
    clippy::cast_possible_truncation
)]

mod input;
mod state;
mod view;

use crate::input::Cursor;
use crate::state::{Grid, Simulation};
use crate::view::{AliveCell, GridBackground};
use modor::{systems, App, BuiltEntity, Component, EntityBuilder};
use modor_graphics::{GraphicsModule, WindowSettings};
use modor_input::Key;
use std::time::Duration;

const GRID_WIDTH: usize = 200;
const REFRESH_FREQUENCY: Duration = Duration::from_millis(50);
const START_STOP_KEY: Key = Key::S;
const RAW_SAVED_GRID: &str = include_str!("../resources/save");

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
pub fn main() {
    App::new()
        .with_entity(GraphicsModule::build(
            WindowSettings::default().title("Modor - game of life"),
        ))
        .with_entity(MainModule::build())
        .run(modor_graphics::runner);
}

#[derive(Component)]
struct MainModule;

#[systems]
impl MainModule {
    fn build() -> impl BuiltEntity {
        EntityBuilder::new()
            .with(Self)
            .with_child(Cursor::default())
            .with_child(Simulation::new())
            .with_child(Grid::new())
            .with_child(GridBackground::build())
    }
}