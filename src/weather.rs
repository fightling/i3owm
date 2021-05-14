// let satellite = "🛰";

extern crate chrono;
extern crate regex;
extern crate reqwest;
extern crate serde;
extern crate serde_json;

use chrono::prelude::*;
use http::StatusCode;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::mpsc;
use std::{result, thread, time};

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
    {wind}          Wind direction, degrees (meteorological)
    {wind_icon}     Wind direction, (meteorological) as arrow icon
    {wind_speed}    Wind speed, {speed_unit}
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

pub type Receiver = mpsc::Receiver<result::Result<String, String>>;

// start weather fetching which will spawn a thread that signals updates from OWM in json format
// via the returned receiver
pub fn init(
    city: &str,
    units: &str,
    lang: &str,
    api_key: &str,
) -> Receiver {
    // generate correct request URL depending on city is id or name
    let url = match city.parse::<u64>() {
        Ok(ok) => format!(
            "https://api.openweathermap.org/data/2.5/weather?id={}&units={}&lang={}&appid={}",
            city, units, lang, api_key
        ),
        Err(e) => format!(
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
    return rx;
}

fn icon(icon_id: &str) -> &str {
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
    return icons.get(icon_id).unwrap();
}

pub fn update(
    rx: mpsc::Receiver<result::Result<String, String>>,
    units: &str,
) -> Option<HashMap<&str, String>> {
    match rx.try_recv() {
        Ok(response) => match serde_json::from_str(&response.unwrap()) {
            Ok(x) => {
                let v: Value = x;
                let directions = ["↑", "↗", "→", "↘", "↓", "↙", "←", "↖", "↑"];
                let mut data = HashMap::new();
                data.insert("{update}", Local::now().format("%H:%M").to_string());
                data.insert("{city}", v["name"].to_string());
                data.insert("{main}", v["weather"][0]["main"].to_string());
                data.insert("{description}", v["weather"][0]["description"].to_string());
                data.insert(
                    "{icon}",
                    icon(v["weather"][0]["icon"].as_str().unwrap()).to_string(),
                );
                data.insert("{pressure}", v["main"]["pressure"].to_string());
                data.insert("{humidity}", v["main"]["humidity"].to_string());
                data.insert("{wind}", v["wind"]["deg"].to_string());
                data.insert(
                    "{wind_icon}",
                    directions[(&v["wind"]["deg"].as_f64().unwrap() / 45.0).round() as usize]
                        .to_string(),
                );
                data.insert(
                    "{wind_speed}",
                    v["wind"]["speed"].as_f64().unwrap().round().to_string(),
                );
                data.insert("{visibility}", v["visibility"].to_string());
                data.insert(
                    "{visibility_km}",
                    (v["visibility"].as_i64().unwrap() / 1000).to_string(),
                );
                data.insert(
                    "{rain.1h}",
                    match v["rain"]["rain.1h"].as_i64() {
                        Some(v) => v,
                        None => 0i64,
                    }
                    .to_string(),
                );
                data.insert(
                    "{rain.3h}",
                    match v["rain"]["rain.3h"].as_i64() {
                        Some(v) => v,
                        None => 0i64,
                    }
                    .to_string(),
                );
                data.insert(
                    "{snow.1h}",
                    match v["snow"]["snow.1h"].as_i64() {
                        Some(v) => v,
                        None => 0i64,
                    }
                    .to_string(),
                );
                data.insert(
                    "{snow.3h}",
                    match v["snow"]["snow.3h"].as_i64() {
                        Some(v) => v,
                        None => 0i64,
                    }
                    .to_string(),
                );
                data.insert(
                    "{temp_min}",
                    v["main"]["temp_min"].as_f64().unwrap().round().to_string(),
                );
                data.insert(
                    "{temp_max}",
                    v["main"]["temp_max"].as_f64().unwrap().round().to_string(),
                );
                data.insert(
                    "{feels_like}",
                    v["main"]["temp"].as_f64().unwrap().round().to_string(),
                );
                data.insert(
                    "{temp}",
                    v["main"]["temp"].as_f64().unwrap().round().to_string(),
                );
                data.insert(
                    "{temp_unit}",
                    match units {
                        "standard" => "K",
                        "metric" => "°C",
                        "imperial" => "°F",
                        _ => "",
                    }
                    .to_string(),
                );
                data.insert(
                    "{speed_unit}",
                    match units {
                        "standard" => "m/s",
                        "metric" => "m/s",
                        "imperial" => "mi/h",
                        _ => "",
                    }
                    .to_string(),
                );
                return Some(data);
            }
            Err(e) => return None,
        },
        Err(e) => return None,
    }
}
