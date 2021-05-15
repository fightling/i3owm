extern crate chrono;
extern crate regex;
extern crate reqwest;
extern crate serde_json;

use chrono::prelude::*;
use http::StatusCode;
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

// type of update receiver channel whic you get from init
type Receiver = mpsc::Receiver<result::Result<String, String>>;

// start weather fetching which will spawn a thread that signals updates from OWM in json format
// via the returned receiver
pub fn receiver(city: &str, units: &str, lang: &str, api_key: &str) -> Receiver {
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
    return rx;
}

type Converter<'a> = HashMap<&'a str, Box<dyn Fn(&Value) -> String>>;

fn deg(v: &Value) -> f64 {
    v["wind"]["deg"].as_f64().unwrap().round()
}
fn dir(v: &Value) -> usize {
    (deg(v).round() as usize % 360) / 45
}

// create a hash map of weather fetch closures by key
pub fn converter(units: &str) -> Converter {
    let mut data: Converter = HashMap::new();
    data.insert(
        "{update}",
        Box::new(|_v| Local::now().format("%H:%M").to_string()),
    );
    data.insert("{city}", Box::new(|v| v["name"].to_string()));
    data.insert("{main}", Box::new(|v| v["weather"][0]["main"].to_string()));
    data.insert(
        "{description}",
        Box::new(|v| v["weather"][0]["description"].to_string()),
    );
    data.insert(
        "{icon}",
        Box::new(|v| icon(v["weather"][0]["icon"].as_str().unwrap()).to_string()),
    );
    data.insert(
        "{pressure}",
        Box::new(|v| v["main"]["pressure"].to_string()),
    );
    data.insert(
        "{humidity}",
        Box::new(|v| v["main"]["humidity"].to_string()),
    );
    data.insert("{wind}", Box::new(|v| v["wind"]["deg"].to_string()));
    data.insert("{wind_deg}", Box::new(move |v| deg(v).to_string()));
    data.insert(
        "{wind}",
        Box::new(|v| {
            let directions = ["N", "NO", "O", "SO", "S", "SW", "W", "NW"];
            directions[dir(v)].to_string()
        }),
    );
    data.insert(
        "{wind_icon}",
        Box::new(move |v| {
            let icons = ["↓", "↙", "←", "↖", "↑", "↗", "→", "↘"];
            icons[dir(v)].to_string()
        }),
    );
    data.insert("{deg_unit}", Box::new(|_v| "°".to_string()));
    data.insert(
        "{wind_speed}",
        Box::new(|v| v["wind"]["speed"].as_f64().unwrap().round().to_string()),
    );
    data.insert("{visibility}", Box::new(|v| v["visibility"].to_string()));
    data.insert(
        "{visibility_km}",
        Box::new(|v| (v["visibility"].as_i64().unwrap() / 1000).to_string()),
    );
    data.insert(
        "{rain.1h}",
        Box::new(|v| {
            match v["rain"]["rain.1h"].as_i64() {
                Some(v) => v,
                None => 0i64,
            }
            .to_string()
        }),
    );
    data.insert(
        "{rain.3h}",
        Box::new(|v| {
            match v["rain"]["rain.3h"].as_i64() {
                Some(v) => v,
                None => 0i64,
            }
            .to_string()
        }),
    );
    data.insert(
        "{snow.1h}",
        Box::new(|v| {
            match v["snow"]["snow.1h"].as_i64() {
                Some(v) => v,
                None => 0i64,
            }
            .to_string()
        }),
    );
    data.insert(
        "{snow.3h}",
        Box::new(|v| {
            match v["snow"]["snow.3h"].as_i64() {
                Some(v) => v,
                None => 0i64,
            }
            .to_string()
        }),
    );
    data.insert(
        "{temp_min}",
        Box::new(|v| v["main"]["temp_min"].as_f64().unwrap().round().to_string()),
    );
    data.insert(
        "{temp_max}",
        Box::new(|v| v["main"]["temp_max"].as_f64().unwrap().round().to_string()),
    );
    data.insert(
        "{feels_like}",
        Box::new(|v| v["main"]["temp"].as_f64().unwrap().round().to_string()),
    );
    data.insert(
        "{temp}",
        Box::new(|v| v["main"]["temp"].as_f64().unwrap().round().to_string()),
    );
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
pub fn update(format: &str, rx: &Receiver, converter: &Converter) -> Option<String> {
    match rx.try_recv() {
        Ok(response) => match response {
            Ok(json) => match serde_json::from_str(&json) {
                Ok(w) => {
                    return Some(formatter(format, w, converter));
                }
                Err(_e) => return Some("[error]".to_string()),
            },
            Err(e) => return Some(e),
        },
        Err(_e) => return None,
    }
}

// get a unicode symbol that matches the OWM icon
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
    return icons.get(icon_id).unwrap_or(&"🚫");
}

// format weather from given data into string
fn formatter(format: &str, w: Value, data: &Converter) -> String {
    let mut result = format.to_string();
    for (k, v) in data {
        result = result.replace(k, &v(&w));
    }
    return result;
}
