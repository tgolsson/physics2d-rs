mod circle;
mod polygon;

use crate::{math::Bounds, world::Transform};
pub use circle::Circle;
pub use polygon::Polygon;

#[derive(Clone)]
pub enum Shape {
    Circle(Circle),
    Polygon(Polygon),
}

#[macro_export]
macro_rules! match_fn_to_shape {
    ($val:expr, $fn_name:ident$args:tt) => {
        match $val {
            Shape::Circle(c) => c.$fn_name$args,
            Shape::Polygon(p) => p.$fn_name$args,
        }
    };
}

pub trait Matter {
    fn mass_and_inertia(&self, density: f32) -> (f32, f32);
    fn bounds(&self, transform: Option<&Transform>) -> Bounds;
}

impl Matter for Shape {
    fn mass_and_inertia(&self, density: f32) -> (f32, f32) {
        match_fn_to_shape!(self, mass_and_inertia(density))
    }

    fn bounds(&self, transform: Option<&Transform>) -> Bounds {
        match_fn_to_shape!(self, bounds(transform))
    }
}
