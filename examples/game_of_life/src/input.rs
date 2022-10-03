use crate::state::{CellPosition, Grid};
use crate::GRID_WIDTH;
use modor::{singleton, Built, EntityBuilder, Single, SingleMut};
use modor_graphics::Camera2D;
use modor_input::{Mouse, MouseButton};
use modor_math::Vec2;
use std::num::TryFromIntError;

pub(crate) struct Cursor {
    last_position: Option<CellPosition>,
}

#[singleton]
impl Cursor {
    pub(crate) fn build() -> impl Built<Self> {
        EntityBuilder::new(Self {
            last_position: None,
        })
    }

    #[run]
    fn update_grid(
        &mut self,
        mut grid: SingleMut<'_, Grid>,
        camera: Single<'_, Camera2D>,
        mouse: Single<'_, Mouse>,
    ) {
        let position = Self::mouse_to_cell_position(camera.mouse_position()).ok();
        if self.should_cell_be_toggled(&mouse, position) {
            if let Some((cursor_x, cursor_y)) = position {
                let state = grid.cell_state(cursor_x, cursor_y).toggled();
                grid.set_cell_state(cursor_x, cursor_y, state);
            }
        }
        self.last_position = position;
    }

    fn mouse_to_cell_position(mouse_position: Vec2) -> Result<CellPosition, TryFromIntError> {
        Ok((
            usize::try_from(((mouse_position.x + 0.5) * GRID_WIDTH as f32) as isize)?,
            usize::try_from(((0.5 - mouse_position.y) * GRID_WIDTH as f32) as isize)?,
        ))
    }

    fn should_cell_be_toggled(&mut self, mouse: &Mouse, position: Option<CellPosition>) -> bool {
        if mouse.button(MouseButton::Left).is_pressed {
            self.last_position.is_none() || position != self.last_position
        } else {
            mouse.button(MouseButton::Left).is_just_released
        }
    }
}
