use reqwest::blocking::{Client, Response};
use serde::Deserialize;

const BASE_URL: &str = "https://api.weather.gov/";

#[derive(Deserialize, Debug)]
pub struct Station {
    pub properties: StationProperties,
}

#[derive(Deserialize, Debug)]
pub struct StationProperties {
    pub name: String,
    pub forecast: String,
    #[serde(rename = "stationIdentifier")]
    pub station_identifier: String,
}

#[derive(Deserialize, Debug)]
pub struct Observation {
    pub properties: ObservationProperties,
}

#[derive(Deserialize, Debug)]
pub struct ObservationProperties {
    #[serde(rename = "textDescription")]
    pub description: String,
    pub timestamp: String,
    pub temperature: Value<Option<f32>>,
    #[serde(rename = "windDirection")]
    pub wind_direction: Value<Option<f32>>,
    #[serde(rename = "windSpeed")]
    pub wind_speed: Value<Option<f32>>,
    #[serde(rename = "relativeHumidity")]
    pub relative_humidity: Value<Option<f32>>,
}

#[derive(Deserialize, Debug)]
pub struct Value<T> {
    pub value: T,
}

#[derive(Deserialize, Debug)]
pub struct Forecast {
    pub properties: ForecastProperties,
}

#[derive(Deserialize, Debug)]
pub struct ForecastProperties {
    pub periods: Vec<ForecastResults>,
}

#[derive(Deserialize, Debug)]
pub struct ForecastResults {
    pub name: String,
    #[serde(rename = "startTime")]
    pub start_time: String,
    pub temperature: f32,
    #[serde(rename = "shortForecast")]
    pub short_forecast: String,
}

pub mod alerts {
    use super::{get_web_json, Deserialize, BASE_URL};

    #[derive(Deserialize, Debug)]
    pub struct Alerts {
        pub title: String,
        pub updated: String,
        pub features: Vec<Feature>,
    }

    #[derive(Deserialize, Debug)]
    pub struct Feature {
        pub properties: Properties,
    }

    #[derive(Deserialize, Debug)]
    pub struct Properties {
        pub severity: String,
        pub certainty: String,
        pub urgency: String,
        pub event: String,
    }

    impl Alerts {
        pub fn from_noaa() -> Result<Self, reqwest::Error> {
            let url = format!("{}/alerts/active/zone/{}", BASE_URL, "WIZ063");
            get_web_json(&url)?.error_for_status()?.json()
        }
    }
}

fn get_web_json(url: &str) -> Result<Response, reqwest::Error> {
    let client = Client::builder().user_agent("weatherapp").build()?;
    client.get(url).send()
}

impl Station {
    pub fn from_station(station_id: &str) -> Result<Self, reqwest::Error> {
        let url = format!("{}/stations/{}", BASE_URL, station_id);
        get_web_json(&url)?.error_for_status()?.json()
    }
}

impl Observation {
    pub fn from_station(station_id: &str) -> Result<Self, reqwest::Error> {
        let url = format!("{}/stations/{}/observations/latest", BASE_URL, station_id);
        get_web_json(&url)?.error_for_status()?.json()
    }
}

impl Forecast {
    pub fn from_noaa() -> Result<Self, reqwest::Error> {
        let url = format!("{}/gridpoints/MKX/37,64/forecast", BASE_URL);
        get_web_json(&url)?.error_for_status()?.json()
    }
}
