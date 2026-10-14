//! `factory-tui` — терминальный интерфейс игры «Factory».
//!
//! Запуск: `cargo run -p factory-tui` (выход — клавиша `q`).

use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::{Frame, Terminal};

use factory_core::{GameState, Resource};

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = GameState::new();
    let result = run(&mut terminal, &mut state);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

/// Главный цикл: рисует кадр и обрабатывает клавиши.
fn run(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    state: &mut GameState,
) -> io::Result<()> {
    loop {
        terminal.draw(|frame| ui(frame, state))?;

        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                // Пока тикаем вручную — позже заведём автотик по таймеру.
                KeyCode::Char(' ') => state.step(),
                _ => {}
            }
        }
    }
}

/// Отрисовка одного кадра.
fn ui(frame: &mut Frame, state: &GameState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(frame.area());

    frame.render_widget(
        Paragraph::new("Фабрика — TUI").block(Block::default().borders(Borders::ALL)),
        chunks[0],
    );

    let info = format!(
        "Тик: {}\nЖелезо: {}\n\nПробел — следующий тик, Q — выход",
        state.tick(),
        state.stock(Resource::Iron)
    );
    frame.render_widget(
        Paragraph::new(info).block(Block::default().borders(Borders::ALL).title("Состояние")),
        chunks[1],
    );
}
