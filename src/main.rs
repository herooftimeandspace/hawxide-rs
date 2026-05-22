use std::{
    io::{self, Stdout},
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

mod game;
mod render;

use game::{Game, MoveDirection};
use render::GameWidget;

const TICK_RATE: Duration = Duration::from_millis(50);

fn main() -> io::Result<()> {
    let mut terminal = setup_terminal()?;
    let result = run(&mut terminal);
    restore_terminal(&mut terminal)?;
    result
}

fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()
}

fn run(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
    let mut game = Game::new();
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|frame| frame.render_widget(GameWidget::new(&game), frame.area()))?;

        let timeout = TICK_RATE.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            if !game.started() && key.code != KeyCode::Esc {
                game.start();
                last_tick = Instant::now();
                continue;
            }

            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Up | KeyCode::Char('e') | KeyCode::Char('E') => {
                    game.move_player(MoveDirection::Up);
                }
                KeyCode::Down | KeyCode::Char('d') | KeyCode::Char('D') => {
                    game.move_player(MoveDirection::Down);
                }
                KeyCode::Char('r') | KeyCode::Char('R') if game.is_game_over() => {
                    game = Game::new();
                }
                _ => {}
            }
        }

        if last_tick.elapsed() >= TICK_RATE {
            let size = terminal.size()?;
            let playfield = GameWidget::playfield_from_area(size.into());
            game.tick(last_tick.elapsed(), playfield);
            last_tick = Instant::now();
        }
    }

    Ok(())
}
