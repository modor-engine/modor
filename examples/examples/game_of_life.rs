#![allow(missing_docs, clippy::cast_precision_loss)]

use instant::Instant;
use modor::{
    systems, App, BuiltEntity, Component, Entity, EntityBuilder, Filter, NoSystem, Query,
    SingletonComponent, With, World,
};
use modor_graphics::{Camera2D, Color, Material, Model, RenderTarget, Window, ZIndex2D};
use modor_math::Vec2;
use modor_physics::Transform2D;
use std::fmt::Debug;
use std::hash::Hash;
use std::time::Duration;

const GRID_SIZE: usize = 150;
const REFRESH_PERIOD: Duration = Duration::from_millis(100);

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
pub fn main() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(window())
        .with_entity(Material::new(MaterialKey::Background).with_color(Color::WHITE))
        .with_entity(Material::new(MaterialKey::AliveCell).with_color(Color::BLACK))
        .with_entity(Grid::load())
        .with_entity(background())
        .run(modor_graphics::runner);
}

fn window() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(RenderTarget::new(TargetKey))
        .with(Window::default())
        .with(Camera2D::new(CameraKey, TargetKey))
}

fn background() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(Transform2D::new())
        .with(Model::rectangle(MaterialKey::Background, CameraKey))
}

fn alive_cell(x: usize, y: usize) -> impl BuiltEntity {
    let position = to_word_position(x, y);
    let size = Vec2::ONE / GRID_SIZE as f32;
    EntityBuilder::new()
        .with(Transform2D::new().with_position(position).with_size(size))
        .with(Model::rectangle(MaterialKey::AliveCell, CameraKey))
        .with(ZIndex2D::from(1))
        .with(AliveCell)
}

fn to_word_position(x: usize, y: usize) -> Vec2 {
    Vec2::new(
        (x as f32 + 0.5) / GRID_SIZE as f32 - 0.5,
        0.5 - (y as f32 + 0.5) / GRID_SIZE as f32,
    )
}

#[derive(SingletonComponent)]
struct Grid {
    are_cells_alive: Vec<Vec<bool>>,
    last_update: Instant,
}

#[systems]
impl Grid {
    fn load() -> Self {
        let mut are_cells_alive = vec![vec![false; GRID_SIZE]; GRID_SIZE];
        for (x, line) in include_str!("../res/game-of-life-grid").lines().enumerate() {
            for (y, character) in line.chars().enumerate() {
                if character == 'X' {
                    are_cells_alive[x][y] = true;
                }
            }
        }
        Self {
            are_cells_alive,
            last_update: Instant::now(),
        }
    }

    #[run]
    fn update(&mut self) {
        if self.last_update.elapsed() > REFRESH_PERIOD {
            self.last_update = Instant::now();
            let are_cells_alive = self.are_cells_alive.clone();
            for x in 0..GRID_SIZE {
                for y in 0..GRID_SIZE {
                    let neighbor_count = Self::neighbor_count(&are_cells_alive, x, y);
                    let is_cell_alive = &mut self.are_cells_alive[x][y];
                    if *is_cell_alive && !(neighbor_count == 2 || neighbor_count == 3) {
                        *is_cell_alive = false;
                    } else if !*is_cell_alive && neighbor_count == 3 {
                        *is_cell_alive = true;
                    }
                }
            }
        }
    }

    #[run_after_previous]
    fn update_display(
        &self,
        entity: Entity<'_>,
        mut cells: Query<'_, (&mut Transform2D, Entity<'_>, Filter<With<AliveCell>>)>,
        mut world: World<'_>,
    ) {
        let mut cells = cells.iter_mut();
        for x in 0..GRID_SIZE {
            for y in 0..GRID_SIZE {
                if self.are_cells_alive[x][y] {
                    if let Some((cell_transform, _, _)) = cells.next() {
                        *cell_transform.position = to_word_position(x, y);
                    } else {
                        world.create_child_entity(entity.id(), alive_cell(x, y));
                    }
                }
            }
        }
        for (_, cell_entity, _) in cells {
            world.delete_entity(cell_entity.id());
        }
    }

    fn neighbor_count(are_cells_alive: &[Vec<bool>], x: usize, y: usize) -> u8 {
        let first_x = x.saturating_sub(1);
        let first_y = y.saturating_sub(1);
        let mut neighbor_count = 0;
        for (neighbor_x, line) in are_cells_alive.iter().enumerate().skip(first_x).take(3) {
            for (neighbor_y, &is_alive) in line.iter().enumerate().skip(first_y).take(3) {
                if (neighbor_x != x || neighbor_y != y) && is_alive {
                    neighbor_count += 1;
                }
            }
        }
        neighbor_count
    }
}

#[derive(Component, NoSystem)]
struct AliveCell;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TargetKey;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CameraKey;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum MaterialKey {
    Background,
    AliveCell,
}
