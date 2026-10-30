mod mapobjects;
pub mod position;

use rand::RngExt;
use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::seq::IndexedRandom;
use std::collections::BTreeMap;
use std::collections::HashSet;

use crate::Position;
use crate::exceptions::MapGenError;
pub use mapobjects::MapObject;
pub use mapobjects::MapObjectMethods;
pub use mapobjects::{
    City, ExtractionFactory, FactoryId, PlayerFactoryBlock, PlayerFactoryMeta, Resource,
    ResourceArea,
};

#[derive(Debug)]
pub enum MapSize {
    Small,
    Medium,
    Large,
}

const CITY_PREFIXES: &[&str] = &[
    "Ново",
    "Старо",
    "Верхне",
    "Нижне",
    "Бело",
    "Черно",
    "Красно",
];
const CITY_STEMS: &[&str] = &["камен", "озер", "речен", "лес", "гор", "степ"];
const CITY_SUFFIX: &str = "ск";

fn generate_city_name(rng: &mut StdRng, used: &mut HashSet<String>) -> String {
    loop {
        let name = format!(
            "{}{}{}",
            CITY_PREFIXES.choose(rng).copied().unwrap(),
            CITY_STEMS.choose(rng).copied().unwrap(),
            CITY_SUFFIX,
        );
        if used.insert(name.clone()) {
            return name;
        }
    }
}

#[derive(Debug)]
pub struct MapConfig {
    pub size: MapSize,
    pub city_count: usize,
    pub extraction_count: usize,
    pub factory_block_count: usize,
    pub seed: u64,
}
impl MapConfig {
    pub fn random_seed() -> u64 {
        rand::random()
    }
    pub fn small_config() -> Self {
        Self {
            size: MapSize::Small,
            city_count: 5,
            extraction_count: 5,
            factory_block_count: 1,
            seed: Self::random_seed(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Map {
    width: usize,
    height: usize,
    objects: BTreeMap<Position, MapObject>,
}

impl Map {
    pub fn width(&self) -> usize {
        self.width
    }
    pub fn height(&self) -> usize {
        self.height
    }
    pub fn object_at(&self, pos: Position) -> Option<&MapObject> {
        self.objects.get(&pos)
    }

    /// Итерация по всем объектам карты: (позиция, объект)
    pub fn objects(&self) -> impl Iterator<Item = (&Position, &MapObject)> {
        self.objects.iter()
    }
    fn place_obj(&mut self, obj: MapObject, position: Position) -> Result<(), MapGenError> {
        if position.x() >= self.width || position.y() >= self.height {
            return Err(MapGenError::OutOfBounds { position });
        }
        if let Some(existing) = self.objects.get(&position) {
            return Err(MapGenError::Occupied {
                position,
                new_obj: obj,
                existing_obj: existing.clone(),
            });
        }
        self.objects.insert(position, obj);
        Ok(())
    }

    fn random_free_position(
        &self,
        rng: &mut StdRng,
        margin: usize,
        tries: usize,
    ) -> Result<Position, MapGenError> {
        // margin - отступ от края карты.
        for _ in 0..tries {
            let pos = Position::new(
                rng.random_range(0..self.width - margin),
                rng.random_range(0..self.height - margin),
            );
            if !self.objects.contains_key(&pos) {
                return Ok(pos);
            }
        }
        Err(MapGenError::FreePositionNotFound)
    }

    pub fn generate(config: &MapConfig) -> Result<Self, MapGenError> {
        let (width, height) = match config.size {
            MapSize::Small => (64usize, 64usize),
            MapSize::Medium => (128usize, 128usize),
            MapSize::Large => (256usize, 256usize),
        };
        let mut rng = StdRng::seed_from_u64(config.seed);
        let mut map = Self {
            width,
            height,
            objects: BTreeMap::new(),
        };

        // Фабрика игрока - всегда в центре карты?
        let center_pos = Position::new(width / 2, height / 2);
        let player_factory = MapObject::PlayerFactoryBlock(PlayerFactoryBlock::new(
            1,
            "TestFactory".to_string(),
            center_pos,
        ));
        // размещение фабрики игрока
        map.place_obj(player_factory, center_pos)?;

        // размещаем города
        let mut used_names = HashSet::new();
        for _ in 0..config.city_count {
            let pos = map.random_free_position(&mut rng, 5, 5)?;
            let name = generate_city_name(&mut rng, &mut used_names);
            map.place_obj(MapObject::City(City::new(name, pos)), pos)?
        }

        Ok(map)
    }
}
