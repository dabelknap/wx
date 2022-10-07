use crossterm::event::{self, Event, KeyCode};
use std::io;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style, Modifier},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table, List, ListItem},
    Frame, Terminal,
};

use crate::weather::{Current, Daily, Units, Forecast};
use crate::noaa::alerts::{Alerts, Feature};

pub fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    current: Current,
    daily: Daily,
    alerts: Alerts,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &current, &daily, &alerts))?;

        if let Event::Key(key) = event::read()? {
            if let KeyCode::Char('q') = key.code {
                return Ok(());
            }
        }
    }
}

fn display_forecast(conditions: &Forecast) -> Vec<Spans> {
    let mut spans = vec![Spans::from("")];
    if let Some(name) = &conditions.name {
        spans.push(Spans::from(vec![
                Span::raw(" "),
                Span::styled(name, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]));
    }
    if let Some(temp) = conditions.temperature {
        spans.push(Spans::from(vec![
                Span::raw(format!(" {:13}", "Temperature")),
                Span::styled(format!("{:.1} F", temp), Style::default().fg(Color::Green)),
        ]));
    }
    if let Some(text) = &conditions.text {
        spans.push(Spans::from(vec![
                Span::raw(format!(" {:13}", "Conditions")),
                Span::styled(format!("{}", text), Style::default().fg(Color::Green)),
        ]));
    }
    spans
}

fn display_alert(alert: &Feature) -> Vec<Spans> {
    vec![Spans::from(""),
        Spans::from(vec![
            Span::raw(" "),
            Span::raw(format!("{:10}", "Event")),
            Span::styled(&alert.properties.event, Style::default().fg(Color::Green)),
        ]),
        Spans::from(vec![
            Span::raw(" "),
            Span::raw(format!("{:10}", "Severity")),
            Span::styled(&alert.properties.severity, Style::default().fg(Color::Green)),
        ]),
        Spans::from(vec![
            Span::raw(" "),
            Span::raw(format!("{:10}", "Certainty")),
            Span::styled(&alert.properties.certainty, Style::default().fg(Color::Green)),
        ]),
    ]
}

fn ui<B: Backend>(f: &mut Frame<B>, current: &Current, daily: &Daily, alerts: &Alerts) {
    let vert_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(4),
            Constraint::Length(f.size().height - 4),
        ])
        .split(f.size());

    let title_widget = Paragraph::new(vec![
        Spans::from(vec![
            Span::raw(" "),
            Span::styled(current.station_id.clone(), Style::default().fg(Color::Blue)),
            Span::raw(" : "),
            Span::styled(current.location.clone(), Style::default().fg(Color::Yellow)),
        ]),
        Spans::from(format!(" {}", current.datetime.format("%d-%m-%Y %H:%M"))),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .border_type(BorderType::Rounded),
    );

    f.render_widget(title_widget, vert_layout[0]);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(vert_layout[1]);

    let lchunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunks[0]);

    let current_block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            " Current Conditions ",
            Style::default().fg(Color::Yellow),
        ))
        .title_alignment(Alignment::Left)
        .border_style(Style::default().fg(Color::Cyan))
        .border_type(BorderType::Rounded);

    let mut rows = vec![];
    if let Some(temp) = current.temperature {
        rows.push(Row::new(vec![
            Cell::from(" Temperature"),
            Cell::from(format!("{:.1} C", temp)).style(Style::default().fg(Color::Green)),
        ]));
    }
    if let (Some(speed), Some(dir)) = (current.wind_speed, current.wind_dir_deg) {
        rows.push(Row::new(vec![
            Cell::from(" Wind"),
            Cell::from(format!("{:.1} KPH ({:.0}°)", speed, dir))
                .style(Style::default().fg(Color::Green)),
        ]));
    }
    if let Some(humid) = current.humidity {
        rows.push(Row::new(vec![
            Cell::from(" Humidity"),
            Cell::from(format!("{:.0}%", humid)).style(Style::default().fg(Color::Green)),
        ]));
    }
    if let Some(text) = &current.text {
        rows.push(Row::new(vec![
            Cell::from(" Conditions"),
            Cell::from(format!("{}", text)).style(Style::default().fg(Color::Green)),
        ]));
    }

    let table = Table::new(rows)
        .block(current_block)
        .widths(&[Constraint::Length(12), Constraint::Length(15)]);

    f.render_widget(table, lchunks[0]);

    let alert_block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(" Alerts ", Style::default().fg(Color::Yellow),
        ))
        .title_alignment(Alignment::Left)
        .border_style(Style::default().fg(Color::Cyan))
        .border_type(BorderType::Rounded);

    let mut list_items = vec![];
    if alerts.features.is_empty() {
        list_items.push(ListItem::new("\n None"));
    } else {
    for alert in &alerts.features {
        list_items.push(ListItem::new(display_alert(&alert)));
    }
    }
    let alert_list = List::new(list_items).block(alert_block);
    f.render_widget(alert_list, lchunks[1]);

    let forecast_block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            " Forecast ",
            Style::default().fg(Color::Yellow),
        ))
        .title_alignment(Alignment::Left)
        .border_style(Style::default().fg(Color::Cyan))
        .border_type(BorderType::Rounded);

    let mut list_items = vec![];
    for forecast in &daily.daily {
        list_items.push(ListItem::new(display_forecast(&forecast.1)));
    }
    let list = List::new(list_items)
        .block(forecast_block);

    f.render_widget(list, chunks[1]);
}
