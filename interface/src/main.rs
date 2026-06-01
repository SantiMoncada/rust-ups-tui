use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use std::{io::stdout, time::Duration};

use ratatui::{
    Terminal,
    backend::{self, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    symbols,
    text::{self, Line, Text},
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout());

    let mut terminal = Terminal::new(backend)?;

    terminal.clear()?;

    loop {
        terminal.draw(|frame| {
            let area = frame.area();

            let body = Block::default()
                .title(" UPS@192.168.0.67 ")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Rounded)
                .style(Style::default().fg(Color::Yellow));

            let inner_body = body.inner(area);

            let body_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Ratio(2, 3), Constraint::Ratio(1, 3)])
                .split(inner_body);

            let chart_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Ratio(2, 1), Constraint::Ratio(1, 2)])
                .split(body_layout[0]);
            frame.render_widget(body, area);

            let text_information = Text::from(vec![
                Line::from("Up time 67 Days"),
                Line::from("Total 50Wh"),
                Line::from("Daily 8Wh"),
                Line::from("Status"),
            ]);

            let data = vec![
                (0.0, 2.0),
                (1.0, 5.0),
                (2.0, 3.0),
                (3.0, 8.0),
                (4.0, 6.0),
                (5.0, 1.0),
                (6.0, 4.0),
            ];

            let dataset = Dataset::default()
                .marker(symbols::Marker::Dot)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(Color::LightGreen))
                .data(&data);

            let chart = Chart::new(vec![dataset])
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(ratatui::widgets::BorderType::Rounded)
                        .border_style(Style::default().fg(Color::Gray))
                        .title(" Charge ")
                        .title_alignment(ratatui::layout::HorizontalAlignment::Center),
                )
                .x_axis(
                    Axis::default().bounds([0.0, 6.0]), // min and max of your data
                )
                .y_axis(
                    Axis::default()
                        .title("Y")
                        .bounds([0.0, 10.0])
                        .labels(vec!["0", "5", "10"])
                        .style(Style::default().fg(Color::Gray)),
                );

            let dataset2 = Dataset::default()
                // .name("my data")
                .marker(symbols::Marker::Dot) // or Dot, Block
                .graph_type(GraphType::Line) // or Bar
                .style(Style::default().fg(Color::Red))
                .data(&data);

            let chart2 = Chart::new(vec![dataset2])
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(ratatui::widgets::BorderType::Rounded)
                        .border_style(Style::default().fg(Color::Gray))
                        .title(" Power ")
                        .title_alignment(ratatui::layout::HorizontalAlignment::Center),
                )
                .x_axis(
                    Axis::default().bounds([0.0, 4.0]), // min and max of your data
                )
                .y_axis(
                    Axis::default()
                        // .title("Y")
                        .bounds([0.0, 10.0]), // .labels(vec!["0", "5", "10"]),
                );

            frame.render_widget(&chart, chart_layout[0]);
            frame.render_widget(&chart2, chart_layout[1]);
            frame.render_widget(&text_information, body_layout[1]);
        })?;

        if event::poll(std::time::Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('c')
                        if key
                            .modifiers
                            .contains(crossterm::event::KeyModifiers::CONTROL) =>
                    {
                        break;
                    }
                    KeyCode::Char('q') => break,
                    _ => {}
                }
            }
        }
    }

    execute!(stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
