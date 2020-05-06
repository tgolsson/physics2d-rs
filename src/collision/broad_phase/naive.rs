use super::{BroadPhase, ProxyId};
use crate::collision::ContactConstraint;
use crate::world::{Bodies, Body, BodyPair, ConstraintsMap};

#[derive(Clone)]
pub struct NaiveBroadPhase;

impl BroadPhase for NaiveBroadPhase {
    fn new_potential_pairs(
        &self,
        bodies: &Bodies,
        constraints: &mut ConstraintsMap<ContactConstraint>,
    ) {
        for body_a in bodies.iter() {
            for body_b in bodies.iter() {
                let body_a_id = body_a.id;
                let body_b_id = body_b.id;

                if body_b_id <= body_a_id {
                    continue;
                }

                let body_a = &body_a;
                let body_b = &body_b;

                if body_a.bounds.intersects(&body_b.bounds)
                    && (!body_a.is_static() || !body_b.is_static())
                {
                    let body_pair = BodyPair(body_a_id, body_b_id);
                    constraints.entry(body_pair).or_insert_with(Vec::new);
                }
            }
        }
    }

    fn create_proxy(&mut self, _body: &Body) -> ProxyId {
        ProxyId::default()
    }

    fn destroy_proxy(&mut self, _proxy_id: ProxyId) {}

    fn update_proxy(&mut self, _proxy_id: ProxyId, _body: &Body) {}
}
