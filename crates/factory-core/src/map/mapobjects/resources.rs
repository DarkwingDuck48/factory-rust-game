use crate::{MapObjectMethods, Position};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resource {
    Wood,
    Iron,
    Stone,
    Sand,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceArea {
    resource: Resource,
    map_position: Position,
}

impl MapObjectMethods for ResourceArea {
    fn get_position(&self) -> Position {
        self.map_position
    }
}
