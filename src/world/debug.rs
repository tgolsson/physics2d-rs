use crate::{collision::Contact, world::World};

pub trait DebugCollision {
    fn contacts(&self) -> Vec<&Contact>;
}

impl DebugCollision for World {
    fn contacts(&self) -> Vec<&Contact> {
        self.contact_constraints
            .values()
            .flat_map(|constraints| constraints.iter().map(|constraint| &constraint.contact))
            .collect()
    }
}
