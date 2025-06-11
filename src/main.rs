use std::env;
use std::fs::{create_dir, read_to_string, OpenOptions};
use std::path::PathBuf;
use std::sync::LazyLock;
use std::{error::Error, io, io::Write};

use clap::Parser;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{backend::CrosstermBackend, Terminal};

use crate::app::run_app;
use crate::cli::Args;
use crate::noaa::{
    alerts::Alerts, forecast::Forecast, gridpoints::Gridpoints, observation::Observation,
    station::Station,
};

mod app;
mod cli;
mod noaa;
mod units;

const CACHE_FILE: &str = "station";

#[cfg(target_os = "macos")]
static CACHE_PATH: LazyLock<Option<PathBuf>> = LazyLock::new(|| {
    let home = env::var("HOME").ok()?;
    let mut path = PathBuf::new();
    path.push(home);
    path.push("Library/Application Support/WX");
    path.push(CACHE_FILE);
    Some(path)
});

fn get_weather_data(station: &str) -> (Observation, Station, Alerts, Forecast) {
    let obs = Observation::from_station(station).unwrap_or_default();
    let stat = Station::from_station(station).unwrap_or_default();
    let alert = Alerts::from_noaa(stat.zone_id()).unwrap_or_default();
    let (lat, lon) = stat.coordinates();
    let grid = Gridpoints::from_coord(lat, lon).unwrap_or_default();
    let forecast = Forecast::from_noaa(grid.forecast_url()).unwrap_or_default();
    (obs, stat, alert, forecast)
}

fn get_station_from_cache() -> Option<String> {
    if let Some(ref path) = *CACHE_PATH {
        read_to_string(path).ok()
    } else {
        None
    }
}

fn cache_station(station: &str) -> Option<()> {
    if let Some(ref path) = *CACHE_PATH {
        let dir = path.parent()?;
        if !dir.exists() {
            create_dir(dir).ok()?;
        }
        let mut file = OpenOptions::new()
            .truncate(true)
            .create(true)
            .write(true)
            .open(path)
            .ok()?;
        file.write_all(station.as_bytes()).ok()?;
    }
    Some(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let station = if let Some(station) = args.station {
        station
    } else if let Some(station) = get_station_from_cache() {
        station
    } else {
        return Err("Specify weather station identifier.".into());
    };

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let res = run_app(&mut terminal, &station, get_weather_data);

    cache_station(&station);

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
