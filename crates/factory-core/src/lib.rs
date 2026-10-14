//! `factory-core` — игровая логика.
//!
//! Весь игровой код (ресурсы, постройки, рецепты) живёт здесь
//! и не зависит от интерфейса, поэтому его можно переиспользовать
//! в TUI, тестах или будущем GUI.

/// Простые ресурсы фабрики.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Resource {
    Iron,
    Copper,
    Coal,
}

/// Состояние игры: текущий тик и запасы ресурсов.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GameState {
    tick: u64,
    stock: std::collections::BTreeMap<Resource, u64>,
}

impl GameState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Продвигает игру на один тик.
    ///
    /// Пока что просто добывает по одной единице железа — заглушка,
    /// которую позже заменит настоящая игровая логика.
    pub fn step(&mut self) {
        self.tick += 1;
        *self.stock.entry(Resource::Iron).or_default() += 1;
    }

    /// Текущий номер тика.
    pub fn tick(&self) -> u64 {
        self.tick
    }

    /// Количество ресурса на складе.
    pub fn stock(&self, resource: Resource) -> u64 {
        self.stock.get(&resource).copied().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step_adds_iron() {
        let mut game = GameState::new();
        game.step();
        assert_eq!(game.tick(), 1);
        assert_eq!(game.stock(Resource::Iron), 1);
    }

    #[test]
    fn new_game_starts_empty() {
        let game = GameState::new();
        assert_eq!(game.tick(), 0);
        assert_eq!(game.stock(Resource::Copper), 0);
    }
}
