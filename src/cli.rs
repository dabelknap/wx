use clap::builder::{styling::AnsiColor, Styles};
use clap::Parser;

const ABOUT: &str = "NOAA weather TUI";

const LONG_ABOUT: &str = "
TUI for viewing weather data sourced from NOAA.

The user supplies the identifier for their NOAA station (e.g. KC29, KMSN, KELP, etc.). You can find
your station identifier by checking your local weather on https://noaa.gov.

The weather station is saved, so subsequent runs of `wx` will use the last station unless otherwise
specified.
";

const STYLES: Styles = Styles::styled()
    .header(AnsiColor::Yellow.on_default())
    .usage(AnsiColor::Green.on_default())
    .literal(AnsiColor::Green.on_default())
    .placeholder(AnsiColor::Green.on_default());

#[derive(Parser, Debug)]
#[command(version, styles=STYLES, about=ABOUT, long_about = LONG_ABOUT)]
pub struct Args {
    #[arg(help = "NOAA weather station identifier (e.g. KMSN, KELP, etc.)")]
    pub station: Option<String>,
}
