use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use std::{io::stdout, time::Duration};

use ratatui::{
    Terminal,
    backend::{self, CrosstermBackend},
    layout::{Alignment, Constraint, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};

struct App {
    counter: u32,
    last_key: String,
}

impl App {
    fn new() -> Self {
        App {
            counter: 0,
            last_key: String::from("none"),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout());

    let mut terminal = Terminal::new(backend)?;

    terminal.clear()?;

    let mut app = App::new();

    loop {
        terminal.draw(|frame| {
            let area = frame.area();

            let body = Block::default()
                .title("UPS@192.168.0.67")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Yellow));

            let inner = body.inner(area);
            let chunks = Layout::default()
                .direction(ratatui::layout::Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Min(0),
                ])
                .split(inner);
            frame.render_widget(body, area);

            let header = Paragraph::new("Press key 'q' to quit!.")
                .block(Block::default().borders(Borders::ALL));
            frame.render_widget(header, chunks[0]);

            let counter_text = format!("Keypresses: {}", app.counter);

            let counter = Paragraph::new(counter_text)
                .style(Style::default().fg(Color::Yellow))
                .block(Block::default().title("Counter").borders(Borders::ALL));

            frame.render_widget(counter, chunks[1]);

            let last_key_text = format!("Last Key: {}", app.last_key);

            let last_key = Paragraph::new(last_key_text)
                .style(Style::default().fg(Color::Green))
                .block(Block::default().title("Counter").borders(Borders::ALL));

            frame.render_widget(last_key, chunks[2]);
        })?;

        if event::poll(std::time::Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char(c) => {
                        app.counter += 1;
                        app.last_key = c.to_string();
                    }
                    KeyCode::Enter => {
                        app.counter += 1;
                        app.last_key = String::from("Enter");
                    }
                    _ => {}
                }
            }
        }
    }

    execute!(stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
