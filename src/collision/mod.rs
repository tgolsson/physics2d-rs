pub mod broad_phase;
mod narrow_phase;
mod solver;

pub use narrow_phase::{collide, Collide};
pub use solver::ContactConstraint;

use crate::math::{Cross, Vec2};

#[derive(Copy, Clone)]
pub struct Contact {
    pub position: Vec2,
    pub penetration: f32,

    pub normal: Vec2,
    pub tangent: Vec2,
}

impl Contact {
    pub fn new(position: Vec2, penetration: f32, normal: Vec2) -> Contact {
        Contact {
            position,
            penetration,
            normal,
            tangent: normal.cross(1.0),
        }
    }
}
