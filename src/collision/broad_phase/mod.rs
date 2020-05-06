mod bounds_tree;
mod naive;

pub use self::bounds_tree::BoundsTreeBroadPhase;
pub use self::naive::NaiveBroadPhase;

use crate::collision::ContactConstraint;
use crate::world::{Bodies, Body, ConstraintsMap};

pub type ProxyId = usize;

pub trait BroadPhase {
    fn new_potential_pairs(
        &self,
        bodies: &Bodies,
        constraints: &mut ConstraintsMap<ContactConstraint>,
    );

    fn create_proxy(&mut self, body: &Body) -> ProxyId;
    fn destroy_proxy(&mut self, proxy_id: ProxyId);
    fn update_proxy(&mut self, proxy_id: ProxyId, body: &Body);
}
