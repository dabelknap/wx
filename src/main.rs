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

use crate::app::run_app;

use crate::noaa::{alerts::Alerts, forecast::Forecast, observation::Observation, station::Station};

const STATION: &str = "KMSN";

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let observation = Observation::from_station(STATION).unwrap();
    let station = Station::from_station(STATION).unwrap();
    let alerts = Alerts::from_noaa().unwrap();
    let forecast = Forecast::from_noaa().unwrap();

    // create app and run it
    let res = run_app(&mut terminal, &observation, &station, &alerts, &forecast);

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
