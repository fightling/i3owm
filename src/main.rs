extern crate chrono;
extern crate regex;
extern crate reqwest;
extern crate serde;
extern crate serde_json;

use chrono::prelude::*;
use clap::{App, Arg};
use http::StatusCode;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::error;
use std::sync::mpsc;
use std::{thread, time};
use text_io::read;

fn insert_weather(
    format: &str,
    line: &str,
    weather: Result<String, String>,
    position: usize,
    reverse: bool,
    units: &str,
    update: DateTime<Local>,
) -> Result<String, serde_json::Error> {
    let result: String;
    // Parse the string of data into serde_json::Value.
    match weather {
        Ok(w) => {
            let v: Value = serde_json::from_str(&w)?;
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
            let directions = ["↑", "↗", "→", "↘", "↓", "↙", "←", "↖"];
            result = format
                .replace("{update}", &update.format("%H:%M").to_string())
                .replace("{city}", v["name"].as_str().unwrap())
                .replace("{main}", v["weather"][0]["main"].as_str().unwrap())
                .replace(
                    "{description}",
                    v["weather"][0]["description"].as_str().unwrap(),
                )
                .replace(
                    "{icon}",
                    icons
                        .get(v["weather"][0]["icon"].as_str().unwrap())
                        .unwrap(),
                )
                .replace(
                    "{pressure}",
                    &v["main"]["pressure"].as_i64().unwrap().to_string(),
                )
                .replace(
                    "{humidity}",
                    &v["main"]["humidity"].as_i64().unwrap().to_string(),
                )
                .replace("{deg}", &v["wind"]["deg"].as_i64().unwrap().to_string())
                .replace(
                    "{deg_icon}",
                    directions[(&v["wind"]["deg"].as_f64().unwrap() / 45.0).round() as usize],
                )
                .replace("{speed}", &v["wind"]["speed"].as_f64().unwrap().to_string())
                .replace(
                    "{visibility}",
                    &v["visibility"].as_i64().unwrap().to_string(),
                )
                .replace(
                    "{visibility_km}",
                    &(v["visibility"].as_i64().unwrap() / 1000).to_string(),
                )
                .replace(
                    "{rain.1h}",
                    &match v["rain"]["rain.1h"].as_i64() {
                        Some(v) => v,
                        None => 0i64,
                    }
                    .to_string(),
                )
                .replace(
                    "{rain.3h}",
                    &match v["rain"]["rain.3h"].as_i64() {
                        Some(v) => v,
                        None => 0i64,
                    }
                    .to_string(),
                )
                .replace(
                    "{snow.1h}",
                    &match v["snow"]["snow.1h"].as_i64() {
                        Some(v) => v,
                        None => 0i64,
                    }
                    .to_string(),
                )
                .replace(
                    "{snow.3h}",
                    &match v["snow"]["snow.3h"].as_i64() {
                        Some(v) => v,
                        None => 0i64,
                    }
                    .to_string(),
                )
                .replace(
                    "{temp_min}",
                    &v["main"]["temp_min"].as_f64().unwrap().round().to_string(),
                )
                .replace(
                    "{temp_max}",
                    &v["main"]["temp_max"].as_f64().unwrap().round().to_string(),
                )
                .replace(
                    "{feels_like}",
                    &v["main"]["temp"].as_f64().unwrap().round().to_string(),
                )
                .replace(
                    "{temp}",
                    &v["main"]["temp"].as_f64().unwrap().round().to_string(),
                )
                .replace(
                    "{temp_unit}",
                    &match units { "standard" => "K", "metric" => "°C", "imperial" => "°F", _ => "" }
                )
                .replace(
                    "{speed_unit}",
                    &match units { "standard" => "m/s", "metric" => "m/s", "imperial" => "mi/h", _ => "" }
                )
        }
        Err(e) => result = e,
    }
    #[derive(Serialize, Deserialize, Debug)]
    struct I3StatusItem {
        name: String,
        instance: Option<String>,
        markup: String,
        color: Option<String>,
        full_text: String,
    };

    let mut items: Vec<I3StatusItem> = serde_json::from_str(&line)?;

    let w: I3StatusItem = I3StatusItem {
        full_text: result.to_string(),
        markup: "none".to_string(),
        name: "weather".to_string(),
        instance: None,
        color: None,
    };
    if reverse {
        items.insert(items.len() - 1 - position, w);
    } else {
        items.insert(position, w);
    }

    let mut r = format!("{:?}", items);

    // FIXIT: all the following replacements are needed because I just can not deal
    // with serde_json the right way :/ PLEASE HELP!

    // remove all the 'Item' names
    // thought about using '#[serde(rename = "name")]' but could not make it work
    r = r.replace("I3StatusItem", "");
    // remove optional values which are 'None'
    // tried '#[serde(skip_serializing_if = "Option::is_none")]' but did not work.
    r = r.replace(", color: None", "");
    r = r.replace(", instance: None", "");
    // add quotations arround json names. can you setup serge_json doing that?
    r = r.replace("full_text", "\"full_text\"");
    r = r.replace("instance", "\"instance\"");
    r = r.replace("color", "\"color\"");
    r = r.replace("markup", "\"markup\"");
    r = r.replace("name", "\"name\"");
    // remove the 'Some()' envelop from all optional values
    let re = Regex::new(r"Some\((?P<v>[^\)]*)\)").unwrap();

    return Ok(re.replace_all(&r, "$v").to_owned().to_string());
}

fn main() -> Result<(), Box<dyn error::Error>> {
    // fetch arguments
    let args = App::new("i3owm")
        .version("0.1.0")
        .about(
            "Open Weather extension for i3status

Example usage in i3config:

  bar {
    status_command i3status | i3owm -p 2 -r -k <key> -c Berlin -f '{icon} {temp_c}°C 💧{humidity}%'
  }

Output would be like:

    ⛅ 11°C 💧55%

",
        )
        .author("Patrick Hoffmann")
        .args(&[
            Arg::new("api")
                .about("OpenWeatherMap API key
(get one at https://openweathermap.org/api)")
                .short('k')
                .long("api-key")
                .takes_value(true),
            Arg::new("city")
                .about("location city
(city's name, comma, 2-letter country code (ISO3166))")
                .short('c')
                .long("city")
                .takes_value(true)
                .required_unless_present("city_id")
                .conflicts_with("city_id")
                .default_value("Berlin,DE"),
            Arg::new("city_id")
                .about("location city ID
(search your city at https://openweathermap.org/find and take ID out of the link you get)")
                .short('i')
                .long("city_id")
                .takes_value(true)
                .required_unless_present("city")
                .conflicts_with("city"),
            Arg::new("format")
                .about("format string")
                .long_about( "available keys are:
{city}          City name
{main}          Group of weather parameters (Rain, Snow, Extreme
                etc.)
{description}   Weather condition within the group
{icon}          Weather icon
{pressure}      Atmospheric pressure (on the sea level, if there is
                no sea_level or grnd_level data), hPa
{humidity}      Humidity, %
{deg}           Wind direction, degrees (meteorological)
{deg_icon}      Wind direction, (meteorological) as arrow icon
{speed}         Wind speed, {speed_unit}
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
",
                )
                .short('f')
                .long("format")
                .default_value("{city} {icon} {current} {temp}{temp_unit} {humidity}%")
                .takes_value(true),
            Arg::new("position")
                .about("position of output in JSON when wrapping i3status")
                .short('p')
                .long("position")
                .takes_value(true),
            Arg::new("reverse")
                .about("reverse position (from right)")
                .short('r')
                .long("reverse"),
            Arg::new("units")
                .about("use imperial units")
                .short('u')
                .long("units")
                .takes_value(true)
                .possible_values(&["metric", "imperial", "standard"])
                .default_value("metric"),
        ])
        .get_matches();
    let city = args.value_of("city").unwrap();
    let city_id = args.value_of("city_id").unwrap_or("");
    let apikey = args.value_of("api").unwrap_or("");
    let units = args.value_of("units").unwrap();
    let format = args.value_of("format").unwrap();
    let url = match city.is_empty() {
        false => format!(
            "https://api.openweathermap.org/data/2.5/weather?q={}&units={}&appid={}",
            city, units, apikey
        ),
        true => format!(
            "https://api.openweathermap.org/data/2.5/weather?id={}&units={}&appid={}",
            city_id, units, apikey
        ),
    };
    let position = args
        .value_of("position")
        .unwrap_or("0")
        .parse::<usize>()
        .unwrap();
    let reverse = args.is_present("reverse");
    // read first two lines and ignore them
    let line: String = read!("{}\n");
    println!("{}", line);
    let line: String = read!("{}\n");
    println!("{}", line);
    // create http client
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

    let mut weather = Err(String::new());
    let mut update = Local::now();
    loop {
        let mut line: String = read!("{}\n");
        let prefix = line.chars().next().unwrap() == ',';
        if prefix {
            line.remove(0);
        }
        match rx.try_recv() {
            Ok(w) => {
                weather = w;
                update = Local::now();
            }
            _ => (),
        }
        if prefix {
            print!(",")
        }
        match insert_weather(format, &line, weather.clone(), position, reverse, units, update) {
            Ok(l) => line = l,
            _ => (),
        }
        println!("{}", line);
    }
}
