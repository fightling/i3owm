extern crate chrono;
extern crate regex;
extern crate reqwest;
extern crate serde_json;

use http::StatusCode;
use serde::Deserialize;
use std::sync::mpsc;
use std::{result, thread, time};

#[derive(Deserialize, Debug)]
pub struct Coord {
    pub lon: f64,
    pub lat: f64,
}

#[derive(Deserialize, Debug)]
pub struct Weather {
    pub id: u64,
    pub main: String,
    pub description: String,
    pub icon: String,
}

#[derive(Deserialize, Debug)]
pub struct Main {
    pub temp: f64,
    pub feels_like: f64,
    pub pressure: f64,
    pub humidity: f64,
    pub temp_min: f64,
    pub temp_max: f64,
    pub sea_level: Option<f64>,
    pub grnd_level: Option<f64>,
}

#[derive(Deserialize, Debug)]
pub struct Wind {
    pub speed: f64,
    pub deg: f64,
    pub gust: Option<f64>,
}

#[derive(Deserialize, Debug)]
pub struct Clouds {
    pub all: f64,
}

#[derive(Deserialize, Debug)]
pub struct Volume {
    #[serde(rename = "1h")]
    pub h1: u64,
    #[serde(rename = "3h")]
    pub h3: u64,
}

#[derive(Deserialize, Debug)]
pub struct Sys {
    #[serde(rename = "type")]
    pub type_: u64,
    pub id: u64,
    pub message: Option<f64>,
    pub country: String,
    pub sunrise: u64,
    pub sunset: u64,
}

#[derive(Deserialize, Debug)]
pub struct CurrentWeather {
    pub coord: Coord,
    pub weather: Vec<Weather>,
    pub base: String,
    pub main: Main,
    pub visibility: u64,
    pub wind: Wind,
    pub clouds: Clouds,
    pub rain: Option<Volume>,
    pub snow: Option<Volume>,
    pub dt: i64,
    pub sys: Sys,
    pub timezone: i64,
    pub id: u64,
    pub name: String,
    pub cod: u64,
}

type Receiver = mpsc::Receiver<result::Result<String, String>>;

// start weather fetching which will spawn a thread that signals updates from OWM in json format
// via the returned receiver
pub fn init(city: &str, units: &str, lang: &str, api_key: &str) -> Receiver {
    // generate correct request URL depending on city is id or name
    let url = match city.parse::<u64>().is_ok() {
        true => format!(
            "https://api.openweathermap.org/data/2.5/weather?id={}&units={}&lang={}&appid={}",
            city, units, lang, api_key
        ),
        false => format!(
            "https://api.openweathermap.org/data/2.5/weather?q={}&units={}&lang={}&appid={}",
            city, units, lang, api_key
        ),
    };
    // fork thread that continuously fetches weather updates every 10 minutes
    let period = time::Duration::from_secs(60 * 10);
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        tx.send(Err("loading...".to_string())).unwrap();
        loop {
            let response = reqwest::blocking::get(&url).unwrap();
            match response.status() {
                StatusCode::OK => {
                    tx.send(Ok(response.text().unwrap())).unwrap();
                    thread::sleep(period);
                }
                _ => {
                    tx.send(Err(response.status().to_string()))
                        .unwrap();
                }
            }
        }
    });
    // return receiver that provides the updated weather as json string
    return rx;
}

// get some weather update or None (if there is nothing new)
pub fn update(receiver: &Receiver) -> Option<Result<CurrentWeather,String>> {
    match receiver.try_recv() {
        Ok(response) => match response {
            Ok(json) => match serde_json::from_str(&json) {
                Ok(w) => Some(Ok(w)),
                Err(e) => Some(Err(e.to_string())),
            }
            Err(e) => Some(Err(e.to_string())),
        },
        Err(_e) => None,
    }
}
