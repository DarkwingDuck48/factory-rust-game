//! `factory-core` — игровая логика.
//!
//! Весь игровой код (ресурсы, постройки, рецепты) живёт здесь
//! и не зависит от интерфейса, поэтому его можно переиспользовать
//! в TUI, тестах или будущем GUI.

mod exceptions;
mod gamestate;
mod map;
pub use exceptions::MapGenError;
pub use gamestate::GameState;
pub use map::FactoryId;
pub use map::{
    City, ExtractionFactory, MapObject, MapObjectMethods, PlayerFactoryBlock, PlayerFactoryMeta,
    Resource, ResourceArea,
    position::{Direction, Position},
};
pub use map::{Map, MapConfig, MapSize};
