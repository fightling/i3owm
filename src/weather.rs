extern crate chrono;
extern crate regex;
extern crate reqwest;
extern crate serde_json;

use chrono::prelude::*;
use http::StatusCode;
use serde::Deserialize;
use std::collections::HashMap;
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
    pub deg: u64,
    pub gust: Option<f64>,
}

#[derive(Deserialize, Debug)]
pub struct Clouds {
    pub all: u64,
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
    pub dt: u64,
    pub sys: Sys,
    pub timezone: i64,
    pub id: u64,
    pub name: String,
    pub cod: u64,
}

// get list of the weather properties as text table
pub fn help() -> &'static str {
    return "available keys are:
    {city}          City name
    {main}          Group of weather parameters (Rain, Snow, Extreme
    etc.)
    {description}   Weather condition within the group
    {icon}          Weather icon
    {pressure}      Atmospheric pressure (on the sea level, if there is
    no sea_level or grnd_level data), hPa
    {humidity}      Humidity, %
    {wind}          Wind direction as N, NW, W, SW, S, SO, O or NO
    {wind_icon}     Wind direction as arrow icon
    {wind_speed}    Wind speed, {speed_unit}
    {wind_deg}      Wind direction, degrees (meteorological)
    {deg_unit}      Direction unit (degrees: °)
    {visibility}    Visibility, meter
    {visibility_km} Visibility, kilometer
    {rain.1h}       Rain volume for the last 1 hour, mm
    {rain.3h}       Rain volume for the last 3 hours, mm
    {snow.1h}       Snow volume for the last 1 hour, mm
    {snow.3h}       Snow volume for the last 3 hours, mm
    {temp_min}      Minimum temperature at the moment. This is minimal
    currently observed temperature (within large
    megalopolises and urban areas), {temp_unit}
    {temp_max}      Maximum temperature at the moment. This is maximal
    currently observed temperature (within large
    megalopolises and urban areas), {temp_unit}
    {feels_like}    Temperature. This temperature parameter accounts
    for the human perception of weather, {temp_unit}
    {temp}          Temperature, {temp_unit}
    {temp_unit}     Temperature
    (standard=K, metric=°C, imperial=°F)
    {speed_unit}    Wind speed unit
    (standard=m/s, metric=m/s, imperial=mi/h
    {update}        Local time of last update, HH:MM
    ";
}

// owm accessor
pub struct Owm<'a>(Receiver, Converter<'a>);

// start weather fetching which will spawn a thread that signals updates from OWM in json format
// via the returned receiver
pub fn init<'a>(city: &str, units: &'a str, lang: &str, api_key: &str) -> Owm<'a> {
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
        tx.send(Err("[offline]".to_string())).unwrap();
        loop {
            let response = reqwest::blocking::get(&url).unwrap();
            match response.status() {
                StatusCode::OK => {
                    tx.send(Ok(response.text().unwrap())).unwrap();
                    thread::sleep(period);
                }
                _ => {
                    tx.send(Err(format!("[{}]", response.status()).to_string()))
                        .unwrap();
                }
            }
        }
    });
    // return receiver that provides the updated weather as json string
    return Owm(rx, converter(units));
}

type Converter<'a> = HashMap<&'a str, Box<dyn Fn(&CurrentWeather) -> String>>;
type Receiver = mpsc::Receiver<result::Result<String, String>>;

fn deg(c: &CurrentWeather) -> u64 {
    c.wind.deg
}
fn dir(v: &CurrentWeather) -> usize {
    (deg(v) as usize % 360) / 45
}

// create a hash map of weather fetch closures by key
fn converter(units: &str) -> Converter {
    let mut data: Converter = HashMap::new();
    data.insert(
        "{update}",
        Box::new(|_c| Local::now().format("%H:%M").to_string()),
    );
    data.insert("{city}", Box::new(|c| c.name.to_string()));
    data.insert("{main}", Box::new(|c| c.weather[0].main.to_string()));
    data.insert(
        "{description}",
        Box::new(|c| c.weather[0].description.to_string()),
    );
    data.insert("{icon}", Box::new(|c| icon(&c.weather[0].icon)));
    data.insert("{pressure}", Box::new(|c| c.main.pressure.to_string()));
    data.insert("{humidity}", Box::new(|c| c.main.humidity.to_string()));
    data.insert("{wind}", Box::new(|c| c.wind.deg.to_string()));
    data.insert("{wind_deg}", Box::new(|c| deg(c).to_string()));
    data.insert(
        "{wind}",
        Box::new(|c| {
            let directions = ["N", "NO", "O", "SO", "S", "SW", "W", "NW"];
            directions[dir(c)].to_string()
        }),
    );
    data.insert(
        "{wind_icon}",
        Box::new(|c| {
            let icons = ["↓", "↙", "←", "↖", "↑", "↗", "→", "↘"];
            icons[dir(c)].to_string()
        }),
    );
    data.insert("{deg_unit}", Box::new(|_c| "°".to_string()));
    data.insert(
        "{wind_speed}",
        Box::new(|c| c.wind.speed.round().to_string()),
    );
    data.insert("{visibility}", Box::new(|c| c.visibility.to_string()));
    data.insert(
        "{visibility_km}",
        Box::new(|c| (c.visibility / 1000).to_string()),
    );
    data.insert(
        "{rain.1h}",
        Box::new(|c| match &c.rain {
            Some(r) => r.h1.to_string(),
            None => "-".to_string(),
        }),
    );
    data.insert(
        "{rain.3h}",
        Box::new(|c| match &c.rain {
            Some(r) => r.h3.to_string(),
            None => "-".to_string(),
        }),
    );
    data.insert(
        "{snow.1h}",
        Box::new(|c| match &c.snow {
            Some(r) => r.h1.to_string(),
            None => "-".to_string(),
        }),
    );
    data.insert(
        "{snow.3h}",
        Box::new(|c| match &c.snow {
            Some(r) => r.h3.to_string(),
            None => "-".to_string(),
        }),
    );
    data.insert(
        "{temp_min}",
        Box::new(|c| c.main.temp_min.round().to_string()),
    );
    data.insert(
        "{temp_max}",
        Box::new(|c| c.main.temp_max.round().to_string()),
    );
    data.insert(
        "{feels_like}",
        Box::new(|c| c.main.temp.round().to_string()),
    );
    data.insert("{temp}", Box::new(|c| c.main.temp.round().to_string()));
    data.insert(
        "{temp_unit}",
        Box::new(match units {
            "standard" => |_v| "K".to_string(),
            "metric" => |_v| "°C".to_string(),
            "imperial" => |_v| "°F".to_string(),
            _ => |_v| "".to_string(),
        }),
    );
    data.insert(
        "{speed_unit}",
        Box::new(match units {
            "standard" => |_v| "m/s".to_string(),
            "metric" => |_v| "m/s".to_string(),
            "imperial" => |_v| "mi/h".to_string(),
            _ => |_v| "".to_string(),
        }),
    );
    return data;
}

// get some weather update or None (if there is nothing new)
pub fn update(format: &str, owm: &Owm) -> Option<String> {
    match owm.0.try_recv() {
        Ok(response) => match response {
            Ok(json) => match serde_json::from_str(&json) {
                Ok(w) => {
                    return Some(formatter(format, w, &owm.1));
                }
                Err(e) => return Some(e.to_string()),
            },
            Err(e) => return Some(e),
        },
        Err(_e) => return None,
    }
}

// get a unicode symbol that matches the OWM icon
fn icon(icon_id: &str) -> String {
    let icons: HashMap<&str, &str> = [
        ("01d", "🌞"),
        ("01n", "🌛"),
        ("02d", "🌤"),
        ("02n", "🌤"),
        ("03d", "⛅"),
        ("03n", "⛅"),
        ("04d", "⛅"),
        ("04n", "⛅"),
        ("09d", "🌧"),
        ("09n", "🌧"),
        ("10d", "🌦"),
        ("10n", "🌦"),
        ("11d", "🌩"),
        ("11n", "🌩"),
        ("13d", "❄"),
        ("13n", "❄"),
        ("50d", "🌫"),
        ("50n", "🌫"),
    ]
    .iter()
    .cloned()
    .collect();
    return icons.get(&icon_id).unwrap_or(&"🚫").to_string();
}

// format weather from given data into string
fn formatter(format: &str, w: CurrentWeather, converter: &Converter) -> String {
    let mut result = format.to_string();
    for (k, v) in converter {
        result = result.replace(k, &v(&w));
    }
    return result;
}
