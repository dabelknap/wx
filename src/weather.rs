use chrono::{Date, DateTime, Local, FixedOffset};

pub enum Condition {
    Clear,
    Overcast,
    Rain,
    Snow,
}

pub enum Warning {
    Tornado,
    Flood,
    Hail,
}

pub enum Units {
    Metric,
    Imperial,
}

pub struct Current {
    pub station_id: String,
    pub location: String,
    pub datetime: DateTime<Local>,
    pub temperature: Option<f32>,
    pub humidity: Option<f32>,
    pub wind_speed: Option<f32>,
    pub wind_dir_deg: Option<f32>,
    pub condition: Option<Condition>,
    pub warnings: Vec<Warning>,
    pub text: Option<String>,
    pub display_units: Units,
}

pub struct Forecast {
    pub name: Option<String>,
    pub temperature: Option<f32>,
    pub display_units: Units,
    pub text: Option<String>,
}

pub struct Daily {
    pub daily: Vec<(DateTime<Local>, Forecast)>
}

pub struct Hourly {
    pub date: Date<FixedOffset>,
    pub hourly: Vec<(DateTime<Local>, Forecast)>,
}
