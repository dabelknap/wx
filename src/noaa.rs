use std::default::Default;

use chrono::{DateTime, Local};
use reqwest::blocking::{Client, Response};
use serde::Deserialize;

const BASE_URL: &str = "https://api.weather.gov/";

pub mod station {
    use super::*;

    #[derive(Deserialize, Debug, Default)]
    pub struct Station {
        pub properties: Properties,

        geometry: Geometry,
    }

    impl Station {
        pub fn from_station(station_id: &str) -> Result<Self, reqwest::Error> {
            let url = format!("{BASE_URL}/stations/{station_id}");
            get_web_json(&url)?.error_for_status()?.json()
        }

        pub fn coordinates(&self) -> (f32, f32) {
            let lat = self.geometry.coordinates.1;
            let lon = self.geometry.coordinates.0;
            (lat, lon)
        }

        pub fn zone_id(&self) -> &str {
            let tokens: Vec<_> = self.properties.forecast.split("/").collect();
            tokens.last().unwrap()
        }
    }

    #[derive(Deserialize, Debug, Default)]
    pub struct Geometry {
        coordinates: (f32, f32),
    }

    #[derive(Deserialize, Debug)]
    pub struct Properties {
        pub name: String,

        forecast: String,

        #[serde(rename = "stationIdentifier")]
        pub station_identifier: String,
    }

    impl Default for Properties {
        fn default() -> Self {
            Self {
                name: "--".to_string(),
                forecast: "--".to_string(),
                station_identifier: "--".to_string(),
            }
        }
    }
}

pub mod observation {
    use super::*;

    #[derive(Deserialize, Debug, Default)]
    pub struct Observation {
        pub properties: Properties,
    }

    impl Observation {
        pub fn from_station(station_id: &str) -> Result<Self, reqwest::Error> {
            let url = format!("{}/stations/{}/observations/latest", BASE_URL, station_id);
            get_web_json(&url)?.error_for_status()?.json()
        }
    }

    #[derive(Deserialize, Debug)]
    pub struct Properties {
        #[serde(rename = "textDescription")]
        pub description: String,

        pub timestamp: String,

        pub temperature: Value<Option<f32>>,

        #[serde(rename = "windChill")]
        pub wind_chill: Value<Option<f32>>,

        #[serde(rename = "windDirection")]
        pub wind_direction: Value<Option<f32>>,

        #[serde(rename = "windSpeed")]
        pub wind_speed: Value<Option<f32>>,

        #[serde(rename = "relativeHumidity")]
        pub relative_humidity: Value<Option<f32>>,
    }

    impl Default for Properties {
        fn default() -> Self {
            let now: DateTime<Local> = Local::now();
            Self {
                description: "--".to_string(),
                timestamp: now.to_rfc3339(),
                temperature: Value::new(None),
                wind_chill: Value::new(None),
                wind_direction: Value::new(None),
                wind_speed: Value::new(None),
                relative_humidity: Value::new(None),
            }
        }
    }

    #[derive(Deserialize, Debug)]
    pub struct Value<T> {
        pub value: T,
    }

    impl<T> Value<T> {
        fn new(value: T) -> Self {
            Self { value }
        }
    }
}

pub mod gridpoints {
    use super::*;

    #[derive(Deserialize, Debug, Default)]
    pub struct Properties {
        forecast: String,
    }

    #[derive(Deserialize, Debug, Default)]
    pub struct Gridpoints {
        properties: Properties,
    }

    impl Gridpoints {
        pub fn from_coord(lat: f32, lon: f32) -> Result<Self, reqwest::Error> {
            let url = format!("{BASE_URL}points/{lat},{lon}");
            get_web_json(&url)?.error_for_status()?.json()
        }

        pub fn forecast_url(&self) -> &str {
            &self.properties.forecast
        }
    }
}

pub mod forecast {
    use super::*;

    #[derive(Deserialize, Debug, Default)]
    pub struct Forecast {
        pub properties: Properties,
    }

    impl Forecast {
        pub fn from_noaa(url: &str) -> Result<Self, reqwest::Error> {
            get_web_json(url)?.error_for_status()?.json()
        }
    }

    #[derive(Deserialize, Debug, Default)]
    pub struct Properties {
        pub periods: Vec<Results>,
    }

    #[derive(Deserialize, Debug)]
    pub struct Results {
        pub name: Option<String>,

        pub temperature: Option<f32>,

        #[serde(rename = "shortForecast")]
        pub short_forecast: Option<String>,
    }
}

pub mod alerts {
    use super::*;

    #[derive(Deserialize, Debug, Default)]
    pub struct Alerts {
        pub features: Vec<Feature>,
    }

    impl Alerts {
        pub fn from_noaa(zone_id: &str) -> Result<Self, reqwest::Error> {
            let url = format!("{BASE_URL}/alerts/active/zone/{zone_id}");
            get_web_json(&url)?.error_for_status()?.json()
        }
    }

    #[derive(Deserialize, Debug)]
    pub struct Feature {
        pub properties: Properties,
    }

    #[derive(Deserialize, Debug)]
    pub struct Properties {
        pub severity: String,
        pub certainty: String,
        pub event: String,
        pub onset: String,
        pub ends: String,
    }
}

fn get_web_json(url: &str) -> Result<Response, reqwest::Error> {
    let client = Client::builder().user_agent("weatherapp").build()?;
    client.get(url).send()
}
