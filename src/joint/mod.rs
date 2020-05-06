mod spring;

use crate::{constraint::Constraint, world::Body};
pub use spring::SpringJoint;

#[derive(Clone)]
pub enum Joint {
    Spring(SpringJoint),
}

#[macro_export]
macro_rules! match_fn_to_joint {
    ($val:expr, $fn_name:ident$args:tt) => {
        match $val {
            Joint::Spring(sj) => sj.$fn_name$args,
        }
    };
}

impl Constraint for Joint {
    fn initialize_velocity(&mut self, a: &Body, b: &Body, dt: f32) {
        match_fn_to_joint!(self, initialize_velocity(a, b, dt))
    }

    fn warm_start_velocity(&mut self, a: &mut Body, b: &mut Body, dt: f32) {
        match_fn_to_joint!(self, warm_start_velocity(a, b, dt))
    }

    fn warm_start_position(&mut self, a: &mut Body, b: &mut Body, dt: f32) {
        match_fn_to_joint!(self, warm_start_position(a, b, dt))
    }

    fn solve_velocity(&mut self, a: &mut Body, b: &mut Body, dt: f32) {
        match_fn_to_joint!(self, solve_velocity(a, b, dt))
    }

    fn solve_position(&mut self, a: &mut Body, b: &mut Body, dt: f32) {
        match_fn_to_joint!(self, solve_position(a, b, dt))
    }
}
