use crate::{MapObjectMethods, Position};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct City {
    name: String,
    map_position: Position,
}

impl City {
    pub fn new(name: String, position: Position) -> Self {
        Self {
            name,
            map_position: position,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl MapObjectMethods for City {
    fn get_position(&self) -> Position {
        self.map_position
    }
}
