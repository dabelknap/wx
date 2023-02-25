use crossterm::event::{self, Event, KeyCode};
use std::io;
use std::{time::Duration, time::Instant};
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Cell, List, ListItem, Paragraph, Row, Table},
    Frame, Terminal,
};

use crate::noaa::alerts;
use crate::noaa::forecast;
use crate::noaa::observation;
use crate::noaa::station;

use chrono::{DateTime, Local};

const MISSING: &str = "--";

pub fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    get_data: fn() -> (
        observation::Observation,
        station::Station,
        alerts::Alerts,
        forecast::Forecast,
    ),
) -> io::Result<()> {
    let (mut observation, mut station, mut alerts, mut forecast) = get_data();
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, &observation, &station, &alerts, &forecast))?;

        if let Event::Key(key) = event::read()? {
            if let KeyCode::Char('q') = key.code {
                return Ok(());
            }
        }

        if last_tick.elapsed() >= Duration::from_millis(1000) {
            last_tick = Instant::now();
            (observation, station, alerts, forecast) = get_data();
        }
    }
}

fn display_forecast(conditions: &forecast::Results) -> Vec<Spans> {
    let mut spans = vec![Spans::from("")];

    let name = if let Some(ref name) = conditions.name {
        name.clone()
    } else {
        MISSING.to_string()
    };
    spans.push(Spans::from(vec![
        Span::raw(" "),
        Span::styled(
            name,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
    ]));

    let temp = if let Some(temp) = conditions.temperature {
        format!("{temp:.1} F")
    } else {
        MISSING.to_string()
    };
    spans.push(Spans::from(vec![
        Span::raw(format!(" {:13}", "Temperature")),
        Span::styled(temp, Style::default().fg(Color::Green)),
    ]));

    let text = if let Some(ref sf) = conditions.short_forecast {
        sf.clone()
    } else {
        MISSING.to_string()
    };
    spans.push(Spans::from(vec![
        Span::raw(format!(" {:13}", "Conditions")),
        Span::styled(text, Style::default().fg(Color::Green)),
    ]));
    spans
}

fn display_alert(alert: &alerts::Feature) -> Vec<Spans> {
    vec![
        Spans::from(""),
        Spans::from(vec![
            Span::raw(" "),
            Span::raw(format!("{:10}", "Event")),
            Span::styled(&alert.properties.event, Style::default().fg(Color::Green)),
        ]),
        Spans::from(vec![
            Span::raw(" "),
            Span::raw(format!("{:10}", "Severity")),
            Span::styled(
                &alert.properties.severity,
                Style::default().fg(Color::Green),
            ),
        ]),
        Spans::from(vec![
            Span::raw(" "),
            Span::raw(format!("{:10}", "Certainty")),
            Span::styled(
                &alert.properties.certainty,
                Style::default().fg(Color::Green),
            ),
        ]),
        Spans::from(vec![
            Span::raw(" "),
            Span::raw(format!("{:10}", "Onset")),
            Span::styled(&alert.properties.onset, Style::default().fg(Color::Green)),
        ]),
        Spans::from(vec![
            Span::raw(" "),
            Span::raw(format!("{:10}", "Ends")),
            Span::styled(&alert.properties.ends, Style::default().fg(Color::Green)),
        ]),
    ]
}

fn display_current_conditions(current: &observation::Properties) -> Table {
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
    rows.push(Row::new(vec![Cell::from("")]));

    let temp = if let Some(temp) = current.temperature.value {
        format!("{temp:.1} C")
    } else {
        MISSING.to_string()
    };
    rows.push(Row::new(vec![
        Cell::from(" Temperature"),
        Cell::from(temp).style(Style::default().fg(Color::Green)),
    ]));

    let wind = if let (Some(speed), Some(dir)) =
        (current.wind_speed.value, current.wind_direction.value)
    {
        format!("{speed:.1} KPH ({dir:.0}°)")
    } else {
        MISSING.to_string()
    };
    rows.push(Row::new(vec![
        Cell::from(" Wind"),
        Cell::from(wind).style(Style::default().fg(Color::Green)),
    ]));

    let humid = if let Some(humid) = current.relative_humidity.value {
        format!("{humid:.0}%")
    } else {
        MISSING.to_string()
    };
    rows.push(Row::new(vec![
        Cell::from(" Humidity"),
        Cell::from(humid).style(Style::default().fg(Color::Green)),
    ]));

    let text = if current.description.is_empty() {
        MISSING.to_string()
    } else {
        current.description.clone()
    };
    rows.push(Row::new(vec![
        Cell::from(" Conditions"),
        Cell::from(text).style(Style::default().fg(Color::Green)),
    ]));

    Table::new(rows)
        .block(current_block)
        .widths(&[Constraint::Length(12), Constraint::Length(15)])
}

fn display_headline<'a>(
    station: &'a station::Properties,
    observation: &'a observation::Properties,
) -> Paragraph<'a> {
    let date: DateTime<Local> =
        DateTime::from(DateTime::parse_from_rfc3339(&observation.timestamp).unwrap());
    Paragraph::new(vec![
        Spans::from(vec![
            Span::raw(" "),
            Span::styled(
                station.station_identifier.clone(),
                Style::default().fg(Color::Blue),
            ),
            Span::raw(" : "),
            Span::styled(station.name.clone(), Style::default().fg(Color::Yellow)),
        ]),
        Spans::from(format!(" {}", date.format("%d-%m-%Y %H:%M"))),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .border_type(BorderType::Rounded),
    )
}

fn ui<B: Backend>(
    f: &mut Frame<B>,
    current: &observation::Observation,
    station: &station::Station,
    alerts: &alerts::Alerts,
    forecast: &forecast::Forecast,
) {
    let vert_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(4),
            Constraint::Length(f.size().height - 4),
        ])
        .split(f.size());

    let title_widget = display_headline(&station.properties, &current.properties);
    f.render_widget(title_widget, vert_layout[0]);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(vert_layout[1]);

    let lchunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunks[0]);

    let current_conditions = display_current_conditions(&current.properties);
    f.render_widget(current_conditions, lchunks[0]);

    let alert_block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(" Alerts ", Style::default().fg(Color::Yellow)))
        .title_alignment(Alignment::Left)
        .border_style(Style::default().fg(Color::Cyan))
        .border_type(BorderType::Rounded);

    let mut list_items = vec![];
    if alerts.features.is_empty() {
        list_items.push(ListItem::new(format!("\n  {MISSING}")));
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
    for fc in &forecast.properties.periods {
        list_items.push(ListItem::new(display_forecast(fc)));
    }
    let list = List::new(list_items).block(forecast_block);

    f.render_widget(list, chunks[1]);
}
