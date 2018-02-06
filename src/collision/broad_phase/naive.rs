use collision::broad_phase::BroadPhase;
use world::{BodyMap, BodyPair};

pub struct NaiveBroadPhase;

impl BroadPhase for NaiveBroadPhase {
    fn potential_pairs(&self, bodies: &BodyMap) -> Vec<BodyPair> {
        let mut pairs = Vec::new();
        
        for (body_a_id, body_a) in bodies.iter() {
            for (body_b_id, body_b) in bodies.iter() {
                if body_b_id <= body_a_id {
                    continue;
                }
        
                let body_pair = BodyPair(*body_a_id, *body_b_id);
                pairs.push(body_pair);
            }
        }
        
        pairs
    }
}