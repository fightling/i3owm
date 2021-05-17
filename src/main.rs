extern crate chrono;
extern crate i3status_ext;
extern crate openweathermap;

use chrono::prelude::*;
use clap::{crate_version, load_yaml, App};
use std::collections::HashMap;

// continuously inject weather into incoming json lines from i3status and pass through
fn main() {
    // fetch arguments
    let yaml = load_yaml!("arg.yaml");
    let args = App::from(yaml).version(crate_version!()).get_matches();
    // convert arguments to rust variables
    let location = args.value_of("location").unwrap();
    let apikey = args.value_of("apikey").unwrap_or("");
    let units = args.value_of("units").unwrap();
    let format = args.value_of("format").unwrap();
    let lang = args.value_of("lang").unwrap_or("en");
    let position = args
        .value_of("position")
        .unwrap_or("0")
        .parse::<usize>()
        .unwrap_or(0);
    let reverse = args.is_present("reverse");
    let poll = args
        .value_of("poll")
        .unwrap_or("10")
        .parse::<u64>()
        .unwrap_or(10);
    // start our observatory via OWM
    let receiver = &openweathermap::init(location, units, lang, apikey, poll);
    i3status_ext::begin();
    // remeber newest weather update and begin with offline message
    let mut current = String::new();
    loop {
        // update current weather info if there is an update available
        match openweathermap::update(receiver) {
            Some(response) => match response {
                Ok(w) => current = make_string(format, &w, units),
                Err(e) => current = e,
            },
            None => (),
        }
        // insert current weather info and print json string or original line
        i3status_ext::update("weather", position, reverse, &current);
    }
}

// create a hash map of weather fetch closures by key
fn make_string(format: &str, current: &openweathermap::CurrentWeather, units: &str) -> String {
    fn dir(current: &openweathermap::CurrentWeather) -> usize {
        (current.wind.deg as usize % 360) / 45
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
    let update: DateTime<Local> = DateTime::from(Utc.timestamp(current.dt, 0));
    format
        .replace("{update}", &update.format("%H:%M").to_string())
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
                Some(r) => r.h1.unwrap_or(0.0).to_string(),
                None => "-".to_string(),
            }
            .to_string(),
        )
        .replace(
            "{rain.3h}",
            &match &current.rain {
                Some(r) => r.h3.unwrap_or(0.0).to_string(),
                None => "-".to_string(),
            },
        )
        .replace(
            "{snow.1h}",
            &match &current.snow {
                Some(r) => r.h1.unwrap_or(0.0).to_string(),
                None => "-".to_string(),
            },
        )
        .replace(
            "{snow.3h}",
            &match &current.snow {
                Some(r) => r.h3.unwrap_or(0.0).to_string(),
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
