mod city;
mod factory;
mod objects_traits;
mod resources;

pub use city::City;
pub use factory::{ExtractionFactory, FactoryId, PlayerFactoryBlock, PlayerFactoryMeta};

pub use objects_traits::MapObjectMethods;
pub use resources::{Resource, ResourceArea};

#[derive(Debug, Clone, PartialEq, Eq)]
// Объекты, которые могут быть на карте
pub enum MapObject {
    City(city::City),                                // Город
    PlayerFactoryBlock(factory::PlayerFactoryBlock), // Фабрика игрока
    ExtractionFactory(factory::ExtractionFactory),   // Добывающая фабрика
    ResourceArea(resources::ResourceArea),           // область добычи ресурсов
}
