extern crate chrono;
extern crate regex;
extern crate reqwest;
extern crate serde;

use chrono::prelude::*;
use clap::{crate_version, App, Arg, ArgMatches};
use std::collections::HashMap;
use std::error;

mod weather;
mod i3status;

// get arguments from application
fn get_args() -> ArgMatches {
    App::new("i3owm")
    .version(crate_version!())
    .about(
        "Open Weather extension for i3status

Example usage in i3config:

bar {
status_command i3status | i3owm -p 2 -r -k <key> -c Berlin,DE -f '{icon} {temp}{temp_unit} 💧{humidity}%'
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
            .long_about(  "available keys are:
    {city}          City name
    {main}          Group of weather parameters (Rain, Snow, Extreme
    etcurrent.)
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
    ")
            .short('f')
            .long("format")
            .default_value("{city} {icon} {current} {temp}{temp_unit} {humidity}%")
            .takes_value(true),
        Arg::new("position")
            .about("position of output in JSON when wrapping i3status")
            .short('p')
            .long("position")
            .takes_value(true),
        Arg::new("lang")
            .about("two character language code of weather descriptions
(default is 'en')")
            .short('l')
            .long("lang")
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
    .get_matches()
}

// continuously inject weather into incoming json lines from i3status and pass through
fn main() -> Result<(), Box<dyn error::Error>> {
    // fetch arguments
    let args = get_args();
    // convert arguments to rust variables
    let city = args
        .value_of("city_id")
        .unwrap_or(args.value_of("city").unwrap());
    let apikey = args.value_of("api").unwrap_or("");
    let units = args.value_of("units").unwrap();
    let format = args.value_of("format").unwrap();
    let lang = args.value_of("lang").unwrap_or("en");
    let position = args
        .value_of("position")
        .unwrap_or("0")
        .parse::<usize>()
        .unwrap();
    let reverse = args.is_present("reverse");
    // start our observatory via OWM
    let receiver = &weather::init(city, units, lang, apikey);
    i3status::begin();
    // remeber newest weather update and begin with offline message
    let mut current = String::new();
    loop {
        let status = i3status::read();
        // update current weather info if there is an update available
        match weather::update(receiver) {
            Some(response) => match response {
                Ok(w) => current = converter(format, &w, units),
                Err(e) => current = e,
            },
            None => (),
        }
        // insert current weather info and print json string or original line
        i3status::write(&status, "weather", position, reverse, &current);
    }
}

fn dir(current: &weather::CurrentWeather) -> usize {
    (current.wind.deg as usize % 360) / 45
}

// create a hash map of weather fetch closures by key
fn converter(format: &str, current: &weather::CurrentWeather, units: &str) -> String {
    format
        .replace("{update}", &Local::now().format("%H:%M").to_string())
        .replace("{city}", current.name.as_str())
        .replace("{main}", current.weather[0].main.as_str())
        .replace("{description}", current.weather[0].description.as_str())
        .replace("{icon}", icon(&current.weather[0].icon))
        .replace("{pressure}", &current.main.pressure.to_string())
        .replace("{humidity}", &current.main.humidity.to_string())
        .replace("{wind}", &current.wind.deg.to_string())
        .replace("{wind_deg}", &current.wind.deg.to_string())
        .replace("{wind}", {
            let directions = ["N", "NO", "O", "SO", "S", "SW", "W", "NW"];
            directions[dir(current)]
        })
        .replace("{wind_icon}", {
            let icons = ["↓", "↙", "←", "↖", "↑", "↗", "→", "↘"];
            icons[dir(current)]
        })
        .replace("{deg_unit}", "°")
        .replace("{wind_speed}", &current.wind.speed.round().to_string())
        .replace("{visibility}", &current.visibility.to_string())
        .replace("{visibility_km}", &(current.visibility / 1000).to_string())
        .replace(
            "{rain.1h}",
            &match &current.rain {
                Some(r) => r.h1.to_string(),
                None => "-".to_string(),
            }
            .to_string(),
        )
        .replace(
            "{rain.3h}",
            &match &current.rain {
                Some(r) => r.h3.to_string(),
                None => "-".to_string(),
            },
        )
        .replace(
            "{snow.1h}",
            &match &current.snow {
                Some(r) => r.h1.to_string(),
                None => "-".to_string(),
            },
        )
        .replace(
            "{snow.3h}",
            &match &current.snow {
                Some(r) => r.h3.to_string(),
                None => "-".to_string(),
            },
        )
        .replace("{temp_min}", &current.main.temp_min.round().to_string())
        .replace("{temp_max}", &current.main.temp_max.round().to_string())
        .replace("{feels_like}", &current.main.temp.round().to_string())
        .replace("{temp}", &current.main.temp.round().to_string())
        .replace(
            "{temp_unit}",
            match units {
                "standard" => "K",
                "metric" => "°C",
                "imperial" => "°F",
                _ => "",
            },
        )
        .replace(
            "{speed_unit}",
            match units {
                "standard" => "m/s",
                "metric" => "m/s",
                "imperial" => "mi/h",
                _ => "",
            },
        )
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
    return icons.get(&icon_id).unwrap_or(&"🚫");
}
