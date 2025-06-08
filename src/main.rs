use std::{error::Error, io};

use clap::Parser;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{backend::CrosstermBackend, Terminal};

use crate::app::run_app;
use crate::noaa::{
    alerts::Alerts, forecast::Forecast, gridpoints::Gridpoints, observation::Observation,
    station::Station,
};

mod app;
mod noaa;
mod units;

const ABOUT: &str = "NOAA weather TUI";
const LONG_ABOUT: &str = "
TUI for viewing weather data sourced from NOAA.

The user supplies the identifier for their NOAA station (e.g. KC29, KMSN, KELP,
etc.). You can find your station identifier by checking your local weather on
\"https://noaa.gov\".";

#[derive(Parser, Debug)]
#[command(version, about=ABOUT, long_about = LONG_ABOUT)]
struct Args {
    #[arg(help = "NOAA weather station identifier (e.g. KMSN, KELP, etc.)")]
    station: String,
}

fn get_weather_data(station: &str) -> (Observation, Station, Alerts, Forecast) {
    let obs = Observation::from_station(station).unwrap_or_default();
    let stat = Station::from_station(station).unwrap_or_default();
    let alert = Alerts::from_noaa(stat.zone_id()).unwrap_or_default();
    let (lat, lon) = stat.coordinates();
    let grid = Gridpoints::from_coord(lat, lon).unwrap_or_default();
    let forecast = Forecast::from_noaa(grid.forecast_url()).unwrap_or_default();
    (obs, stat, alert, forecast)
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let res = run_app(&mut terminal, &args.station, get_weather_data);

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
