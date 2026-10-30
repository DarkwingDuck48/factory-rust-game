use crate::Position;

pub trait MapObjectMethods {
    fn get_position(&self) -> Position;
}
