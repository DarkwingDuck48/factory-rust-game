use crate::MapObjectMethods;
use crate::Position;
use crate::Resource;

pub type FactoryId = u64;

/// Пока не понятно, будет ли у нас возможность создавать несколько разных фабрик на карте
/// С разными точками. Добавим на всякий случай ID фабрики к блоку, чтобы можно было в дальнейшем
/// это развить

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerFactoryBlock {
    factory: FactoryId,
    name: String,
    map_position: Position,
}

impl PlayerFactoryBlock {
    pub fn new(factoryid: FactoryId, name: String, position: Position) -> Self {
        Self {
            factory: factoryid,
            name,
            map_position: position,
        }
    }
}

impl MapObjectMethods for PlayerFactoryBlock {
    fn get_position(&self) -> Position {
        self.map_position
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExtractionFactory {
    resource: Resource,
    map_position: Position,
}

impl ExtractionFactory {
    pub fn new(resource: Resource, position: Position) -> Self {
        Self {
            resource,
            map_position: position,
        }
    }
}

impl MapObjectMethods for ExtractionFactory {
    fn get_position(&self) -> Position {
        self.map_position
    }
}
#[derive(Debug, Clone)]
pub struct PlayerFactoryMeta {}
