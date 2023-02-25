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
mod units;

use crate::app::run_app;

use crate::noaa::{alerts::Alerts, forecast::Forecast, observation::Observation, station::Station};

const STATION: &str = "KMSN";

fn get_weather_data() -> (Observation, Station, Alerts, Forecast) {
    (
        Observation::from_station(STATION).unwrap_or_default(),
        Station::from_station(STATION).unwrap_or_default(),
        Alerts::from_noaa().unwrap_or_default(),
        Forecast::from_noaa().unwrap_or_default(),
    )
}

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let res = run_app(&mut terminal, get_weather_data);

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
