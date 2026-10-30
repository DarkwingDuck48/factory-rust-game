#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

// Структура для работы с координатами
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
    x: usize,
    y: usize,
}

impl Position {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    pub fn x(self) -> usize {
        self.x
    }

    pub fn y(self) -> usize {
        self.y
    }

    // Соседа за границей координатной сетки не существует, поэтому шаг в
    // отрицательные координаты (или переполнение) возвращает None
    pub fn step(self, direction: Direction) -> Option<Self> {
        //  Весь проект — терминальный, карта — сетка, дороги и конвейеры рисуются клетка за клеткой в том же пространстве, что и рендер.
        //  Для терминала - движение вверх это смещение по строке назад, поэтому мы реализуем это движение так
        match direction {
            Direction::Up => self.y.checked_sub(1).map(|y| Self { x: self.x, y }),
            Direction::Down => self.y.checked_add(1).map(|y| Self { x: self.x, y }),
            Direction::Left => self.x.checked_sub(1).map(|x| Self { x, y: self.y }),
            Direction::Right => self.x.checked_add(1).map(|x| Self { x, y: self.y }),
        }
    }
}
