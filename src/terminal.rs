use std::{
    io::{stdout, Stdout},
    time::Duration,
};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Row, Table},
    Terminal,
};

use crate::handlers::{
    app::App,
    config::CompleteConfig,
    event::{Config, Event, Events, Key},
};

fn reset_terminal() {
    disable_raw_mode().unwrap();

    execute!(stdout(), LeaveAlternateScreen).unwrap();
}

fn init_terminal() -> Terminal<CrosstermBackend<Stdout>> {
    enable_raw_mode().unwrap();

    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();

    let backend = CrosstermBackend::new(stdout);

    Terminal::new(backend).unwrap()
}

pub async fn ui_driver(mut config: CompleteConfig, mut app: App) {
    let original_hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |panic| {
        reset_terminal();
        original_hook(panic);
    }));

    let mut events = Events::with_config(Config {
        exit_key: Key::Null,
        tick_rate: Duration::from_millis(config.terminal.tick_delay),
    })
    .await;

    let mut terminal = init_terminal();

    terminal.clear().unwrap();

    let quitting = |mut terminal: Terminal<CrosstermBackend<Stdout>>| {
        disable_raw_mode().unwrap();
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )
        .unwrap();
        terminal.show_cursor().unwrap();
    };

    'outer: loop {
        terminal
            .draw(|frame| {
                let vertical_chunk_constraints = vec![Constraint::Min(1)];

                let vertical_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints(vertical_chunk_constraints.as_ref())
                    .split(frame.size());

                let table = Table::new(
                    posts
                        .iter()
                        .map(|f| Row::new(vec![f.title.as_str()]))
                        .collect::<Vec<Row>>(),
                )
                .style(Style::default().fg(Color::White))
                .header(
                    Row::new(vec!["Title"])
                        .style(Style::default().fg(Color::Yellow))
                        .bottom_margin(1),
                )
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("[ Reddit feed ]"),
                )
                .widths(&[Constraint::Percentage(100)])
                .column_spacing(1)
                .highlight_style(Style::default().add_modifier(Modifier::BOLD));

                frame.render_widget(table, vertical_chunks[0])
            })
            .unwrap();

        if let Some(Event::Input(key)) = &events.next().await {
            match key {
                Key::Esc => {
                    quitting(terminal);
                    break 'outer;
                }
                _ => {}
            }
        }
    }
}
