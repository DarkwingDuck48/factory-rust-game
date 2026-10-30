//! `factory-tui` — терминальный интерфейс игры «Factory».
//!
//! Запуск: `cargo run -p factory-tui` (выход — клавиша `q`).
//!
//! Экран глобальной карты: карта генерируется один раз со случайным
//! seed (показан в заголовке — им же расстановка воспроизводится).
//! Слева карта, справа — список городов: Tab переключает фокус,
//! ↑↓ выбирают город, камера сразу следует за выбором.
//! Курсор карты — hjkl/стрелки (его координаты — в правой части
//! заголовка, камера следит, чтобы он не ушёл за край экрана),
//! пан камеры — HJKL/Shift+стрелки. Внизу — легенда и подсказка
//! клавиш текущего режима.

use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use factory_core::{Direction, Map, MapConfig, MapObject, Position};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::{Frame, Terminal};

/// Ширина сайдбара: рамка + маркер + глиф + самое длинное имя (13 клеток)
const SIDEBAR_WIDTH: u16 = 22;

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run(&mut terminal);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

/// Фокус ввода: навигация по карте или выбор города в списке.
#[derive(Clone, Copy, PartialEq)]
enum Focus {
    Map,
    Sidebar,
}

/// Состояние экрана карты.
struct App {
    map: Map,
    seed: u64,
    /// Города в порядке обхода карты: (позиция, имя)
    cities: Vec<(Position, String)>,
    /// Сдвиг левого-верхнего угла видимой области
    offset: (u16, u16),
    /// Курсор на карте: его шагает игрок, его координаты — в заголовке
    cursor: Position,
    selected: usize,
    focus: Focus,
}

/// Главный цикл: генерирует карту и рисует кадры, обрабатывает клавиши.
fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    let config = MapConfig::small_config();
    let seed = config.seed;
    // Обработка ошибки генерации будет подключена к экранам UI позже;
    // на этом этапе падение с понятным сообщением достаточно.
    let map = Map::generate(&config).expect("не удалось сгенерировать карту");
    let cities = collect_cities(&map);
    let cursor = Position::new(map.width() / 2, map.height() / 2);
    let mut app = App {
        map,
        seed,
        cities,
        offset: (0, 0),
        cursor,
        selected: 0,
        focus: Focus::Map,
    };

    app.offset = center_offset(&app.map, map_viewport(terminal.size()?.into()));

    loop {
        // Клампим каждый кадр: границы зависят и от камеры, и от размера окна
        let view = map_viewport(terminal.size()?.into());
        clamp_offset(&mut app.offset, &app.map, view);
        terminal.draw(|frame| ui(frame, &app))?;

        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            match app.focus {
                Focus::Map => {
                    let shift = key.modifiers.contains(KeyModifiers::SHIFT);
                    match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Tab => app.focus = Focus::Sidebar,
                        // Курсор: hjkl / стрелки (без Shift) — камера следует
                        KeyCode::Char('h') | KeyCode::Left if !shift => {
                            move_cursor(&mut app, view, Direction::Left);
                        }
                        KeyCode::Char('l') | KeyCode::Right if !shift => {
                            move_cursor(&mut app, view, Direction::Right);
                        }
                        KeyCode::Char('k') | KeyCode::Up if !shift => {
                            move_cursor(&mut app, view, Direction::Up);
                        }
                        KeyCode::Char('j') | KeyCode::Down if !shift => {
                            move_cursor(&mut app, view, Direction::Down);
                        }
                        // Пан камеры: HJKL / Shift+стрелки
                        KeyCode::Char('H') | KeyCode::Left => {
                            app.offset.0 = app.offset.0.saturating_sub(1);
                        }
                        KeyCode::Char('L') | KeyCode::Right => {
                            app.offset.0 = app.offset.0.saturating_add(1);
                        }
                        KeyCode::Char('K') | KeyCode::Up => {
                            app.offset.1 = app.offset.1.saturating_sub(1);
                        }
                        KeyCode::Char('J') | KeyCode::Down => {
                            app.offset.1 = app.offset.1.saturating_add(1);
                        }
                        _ => {}
                    }
                }
                Focus::Sidebar => match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Tab | KeyCode::Esc => app.focus = Focus::Map,
                    KeyCode::Up | KeyCode::Char('k') => {
                        app.selected = app.selected.saturating_sub(1);
                        goto_selected(&mut app, view);
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        app.selected = (app.selected + 1).min(app.cities.len().saturating_sub(1));
                        goto_selected(&mut app, view);
                    }
                    _ => {}
                },
            }
        }
    }
}

/// Города карты в детерминированном порядке обхода (по позиции).
fn collect_cities(map: &Map) -> Vec<(Position, String)> {
    map.objects()
        .filter_map(|(pos, obj)| match obj {
            MapObject::City(city) => Some((*pos, city.name().to_string())),
            _ => None,
        })
        .collect()
}

/// Компоновка кадра: карта, сайдбар городов, нижняя строка-легенда.
fn split(area: Rect) -> (Rect, Rect, Rect) {
    let [full, legend_area] =
        Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(area);
    let [map_area, sidebar_area] =
        Layout::horizontal([Constraint::Min(0), Constraint::Length(SIDEBAR_WIDTH)]).areas(full);
    (map_area, sidebar_area, legend_area)
}

/// Видимая область карты: внутренность рамки минус сайдбар и легенда.
/// Используется и для отрисовки, и для клампа камеры в `run`.
fn map_viewport(area: Rect) -> Rect {
    Block::default().borders(Borders::ALL).inner(split(area).0)
}

/// Ограничение камеры границами карты. Если карта меньше окна,
/// камера прижимается к левому-верхнему углу (сдвиг 0).
fn clamp_offset(offset: &mut (u16, u16), map: &Map, view: Rect) {
    let max_x = map.width().saturating_sub(view.width as usize) as u16;
    let max_y = map.height().saturating_sub(view.height as usize) as u16;
    offset.0 = offset.0.min(max_x);
    offset.1 = offset.1.min(max_y);
}

/// Центрировать камеру на точке карты.
fn center_on(offset: &mut (u16, u16), map: &Map, view: Rect, target: Position) {
    *offset = (
        (target.x() as u16).saturating_sub(view.width / 2),
        (target.y() as u16).saturating_sub(view.height / 2),
    );
    clamp_offset(offset, map, view);
}

/// Стартовая камера: центр карты в центре экрана (там фабрика игрока).
fn center_offset(map: &Map, view: Rect) -> (u16, u16) {
    let mut offset = (0u16, 0u16);
    center_on(
        &mut offset,
        map,
        view,
        Position::new(map.width() / 2, map.height() / 2),
    );
    offset
}

/// Камера прыгает к выбранному в списке городу, курсор — на клетку города.
fn goto_selected(app: &mut App, view: Rect) {
    if let Some((pos, _)) = app.cities.get(app.selected) {
        app.cursor = *pos;
        center_on(&mut app.offset, &app.map, view, *pos);
    }
}

/// Шаг курсора в направлении (`Position::step` — общий механизм движения
/// по сетке); шаг за границу карты игнорируется.
fn move_cursor(app: &mut App, view: Rect, direction: Direction) {
    let Some(next) = app.cursor.step(direction) else {
        return;
    };
    if next.x() >= app.map.width() || next.y() >= app.map.height() {
        return;
    }
    app.cursor = next;
    ensure_visible(&mut app.offset, &app.map, view, next);
}

/// Сдвинуть камеру минимально, чтобы курсор остался в видимой области
/// (vim-подобный скролл: экран дёргается только у края).
fn ensure_visible(offset: &mut (u16, u16), map: &Map, view: Rect, cursor: Position) {
    let cx = cursor.x() as u16;
    let cy = cursor.y() as u16;
    if cx < offset.0 {
        offset.0 = cx;
    } else if view.width > 0 && cx >= offset.0 + view.width {
        offset.0 = (cx + 1).saturating_sub(view.width);
    }
    if cy < offset.1 {
        offset.1 = cy;
    } else if view.height > 0 && cy >= offset.1 + view.height {
        offset.1 = (cy + 1).saturating_sub(view.height);
    }
    clamp_offset(offset, map, view);
}

/// Стиль рамки активной зоны.
fn border_style(focused: bool) -> Style {
    if focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    }
}

/// Отрисовка одного кадра глобальной карты.
fn ui(frame: &mut Frame, app: &App) {
    let (map_area, sidebar_area, legend_area) = split(frame.area());

    // --- Карта ---
    // Координаты курсора — правый сегмент заголовка, видны всегда
    let coords = Line::styled(
        format!(" x: {}  y: {} ", app.cursor.x(), app.cursor.y()),
        Style::default().fg(Color::Gray).bold(),
    )
    .right_aligned();
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Фабрика — seed {} ", app.seed))
        .title(coords)
        .border_style(border_style(app.focus == Focus::Map));
    let inner = block.inner(map_area);
    frame.render_widget(block, map_area);

    // Выбранный город подсвечивается на карте
    let selected_pos = app.cities.get(app.selected).map(|(pos, _)| *pos);
    let buf = frame.buffer_mut();
    for dy in 0..inner.height {
        for dx in 0..inner.width {
            let mx = app.offset.0 + dx;
            let my = app.offset.1 + dy;
            // Окно может быть больше карты — вне её границ пусто
            if mx as usize >= app.map.width() || my as usize >= app.map.height() {
                continue;
            }
            let pos = Position::new(mx as usize, my as usize);
            let (symbol, color) = match app.map.object_at(pos) {
                Some(MapObject::City(_)) => ("⌂", Color::Cyan),
                Some(MapObject::ExtractionFactory(_)) => ("E", Color::Yellow),
                Some(MapObject::PlayerFactoryBlock(_)) => ("P", Color::Green),
                Some(MapObject::ResourceArea(_)) => ("▲", Color::Blue),
                None => ("·", Color::DarkGray),
            };
            let mut style = Style::default().fg(color);
            if pos == app.cursor {
                // Курсор поверх всего: белая ячейка читается на любом фоне
                style = Style::default().fg(Color::Black).bg(Color::White);
            } else if selected_pos == Some(pos) {
                style = style.reversed();
            }
            if let Some(cell) = buf.cell_mut((inner.x + dx, inner.y + dy)) {
                cell.set_symbol(symbol);
                cell.set_style(style);
            }
        }
    }

    // --- Сайдбар со списком городов ---
    let sidebar_block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Города ({}) ", app.cities.len()))
        .border_style(border_style(app.focus == Focus::Sidebar));
    let lines: Vec<Line> = app
        .cities
        .iter()
        .enumerate()
        .map(|(i, (_, name))| {
            let selected = i == app.selected;
            let base = if selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };
            Line::from(vec![
                Span::styled(if selected { "▸ " } else { "  " }, base),
                Span::styled("⌂ ", base.fg(Color::Cyan)),
                Span::styled(name.as_str(), base),
            ])
        })
        .collect();
    frame.render_widget(Paragraph::new(lines).block(sidebar_block), sidebar_area);

    // --- Легенда: символы + подсказка клавиш текущего режима ---
    let mut legend = vec![
        Span::styled("⌂", Style::default().fg(Color::Cyan)),
        Span::raw(" город  "),
        Span::styled("E", Style::default().fg(Color::Yellow)),
        Span::raw(" добыча  "),
        Span::styled("P", Style::default().fg(Color::Green)),
        Span::raw(" фабрика  "),
        Span::styled("▲", Style::default().fg(Color::Blue)),
        Span::raw(" ресурсы  "),
    ];
    match app.focus {
        Focus::Map => {
            legend.push(Span::styled("hjkl", Style::default().bold()));
            legend.push(Span::raw(" курсор  "));
            legend.push(Span::styled("HJKL", Style::default().bold()));
            legend.push(Span::raw(" камера  "));
            legend.push(Span::styled("Tab", Style::default().bold()));
            legend.push(Span::raw(" список  "));
        }
        Focus::Sidebar => {
            legend.push(Span::styled("↑↓/JK", Style::default().bold()));
            legend.push(Span::raw(" выбор  "));
            legend.push(Span::styled("Tab", Style::default().bold()));
            legend.push(Span::raw(" карта  "));
        }
    }
    legend.push(Span::styled("Q", Style::default().bold()));
    legend.push(Span::raw(" выход"));
    frame.render_widget(Paragraph::new(Line::from(legend)), legend_area);
}
