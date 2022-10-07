#![allow(dead_code)]

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io};
use tui::{backend::CrosstermBackend, Terminal};

mod app;
mod noaa;
mod weather;

use crate::app::run_app;

use crate::noaa::{Observation, Station, alerts::Alerts};
use crate::weather::{Current, Units, Daily, Forecast};
use chrono::DateTime;

const STATION: &str = "KMSN";

fn get_weather() -> Current {
    let obs = Observation::from_station(STATION).unwrap();
    let sta = Station::from_station(STATION).unwrap();

    Current {
        station_id: sta.properties.station_identifier,
        location: sta.properties.name,
        datetime: DateTime::from(DateTime::parse_from_rfc3339(&obs.properties.timestamp).unwrap()),
        temperature: obs.properties.temperature.value,
        humidity: obs.properties.relative_humidity.value,
        wind_speed: obs.properties.wind_speed.value,
        wind_dir_deg: obs.properties.wind_direction.value,
        condition: None,
        warnings: vec![],
        text: Some(obs.properties.description),
        display_units: Units::Metric,
    }
}

fn get_daily() -> Daily {
    let fore = noaa::Forecast::from_noaa().unwrap();
    let mut days = vec![];
    for f in fore.properties.periods {
        let d = Forecast {
            temperature: Some(f.temperature),
            display_units: Units::Imperial,
            name: Some(f.name),
            text: Some(f.short_forecast),
        };
        let ts = DateTime::from(DateTime::parse_from_rfc3339(&f.start_time).unwrap());
        days.push((ts, d));
    }
    Daily { daily: days }
}

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let current = get_weather();
    let daily = get_daily();

    // create app and run it
    let res = run_app(&mut terminal, current, daily, Alerts::from_noaa().unwrap());

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}
