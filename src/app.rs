use crossterm::event::{self, Event, KeyCode};
use std::io;
use std::time::Duration;
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
use crate::units::direction::degree_to_compass;
use crate::units::speed::kph2mph;
use crate::units::temperature::c2f;
use std::sync::{mpsc, mpsc::Receiver, Arc, RwLock};
use std::thread;

use chrono::{DateTime, Local};

const MISSING: &str = "--";

type WeatherData = (
    observation::Observation,
    station::Station,
    alerts::Alerts,
    forecast::Forecast,
);

pub fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    get_data: fn() -> WeatherData,
) -> io::Result<()> {
    let weather_data = Arc::new(RwLock::new(get_data()));
    let rx = start_workers(weather_data.clone(), get_data);
    loop {
        if let Ok(data) = weather_data.read() {
            terminal.draw(|f| ui(f, &data.0, &data.1, &data.2, &data.3))?;
        }

        match rx.recv().unwrap() {
            AppEvent::Redraw => (),
            AppEvent::Exit => return Ok(()),
        }
    }
}

enum AppEvent {
    Redraw,
    Exit,
}

fn start_workers(
    weather_data: Arc<RwLock<WeatherData>>,
    get_data: fn() -> WeatherData,
) -> Receiver<AppEvent> {
    let (tx, rx) = mpsc::channel();

    // Web request worker.
    let web_tx = tx.clone();
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(10));
        let data = get_data();
        if let Ok(mut wdat) = weather_data.write() {
            *wdat = data;
        }
        _ = web_tx.send(AppEvent::Redraw);
    });

    // Handle TUI events.
    let event_tx = tx.clone();
    thread::spawn(move || loop {
        match event::read().unwrap() {
            Event::Key(key) => {
                if let KeyCode::Char('q') = key.code {
                    _ = event_tx.send(AppEvent::Exit);
                }
            }
            Event::Resize(_, _) => {
                _ = event_tx.send(AppEvent::Redraw);
            }
            _ => (),
        }
    });

    rx
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
    let onset: DateTime<Local> =
        DateTime::from(DateTime::parse_from_rfc3339(&alert.properties.onset).unwrap());
    let ends: DateTime<Local> =
        DateTime::from(DateTime::parse_from_rfc3339(&alert.properties.ends).unwrap());
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
            Span::styled(
                format!("{}", onset.format("%d-%m-%Y %H:%M")),
                Style::default().fg(Color::Green),
            ),
        ]),
        Spans::from(vec![
            Span::raw(" "),
            Span::raw(format!("{:10}", "Ends")),
            Span::styled(
                format!("{}", ends.format("%d-%m-%Y %H:%M")),
                Style::default().fg(Color::Green),
            ),
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
        let temp = c2f(temp);
        format!("{temp:.1} F")
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
        let speed = kph2mph(speed);
        let compass = degree_to_compass(dir);
        format!("{speed:.1} MPH ({compass})")
    } else {
        MISSING.to_string()
    };
    rows.push(Row::new(vec![
        Cell::from(" Wind"),
        Cell::from(wind).style(Style::default().fg(Color::Green)),
    ]));

    let wind_chill = if let Some(wind_chill) = current.wind_chill.value {
        let wind_chill = c2f(wind_chill);
        format!("{wind_chill:.1} F")
    } else {
        MISSING.to_string()
    };
    rows.push(Row::new(vec![
        Cell::from(" Wind Chill"),
        Cell::from(wind_chill).style(Style::default().fg(Color::Green)),
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
        .widths(&[Constraint::Length(12), Constraint::Length(25)])
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
