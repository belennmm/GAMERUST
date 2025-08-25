#[derive(Debug, Clone, Copy, PartialEq, Eq)]


pub enum LookDirection {
    Up,
    Down,
    Left,
    Right,
}

impl LookDirection {
    pub fn position_from(&self, position: &[i32; 2]) -> [i32; 2] {
        let [x, y] = *position;

        match self {
            LookDirection::Up => [x, y - 1],
            LookDirection::Down => [x, y + 1],
            LookDirection::Left => [x - 1, y],
            LookDirection::Right => [x + 1, y],
        }
    }
}

// transform.rs
use glam::Vec3;
use crate::constants::{FLOOR_Y, UNITS_PER_TILE};

#[inline]
pub fn tile_to_world(tile: [i32; 2]) -> Vec3 {
    Vec3::new(tile[0] as f32 * UNITS_PER_TILE, FLOOR_Y, tile[1] as f32 * UNITS_PER_TILE)
}

#[inline]
pub fn world_from_xy(x: f32, y: f32) -> Vec3 {
    Vec3::new(x * UNITS_PER_TILE, FLOOR_Y, y * UNITS_PER_TILE)
}
