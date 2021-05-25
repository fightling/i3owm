extern crate chrono;
extern crate i3status_ext;
extern crate notify_rust;
extern crate openweathermap;

use chrono::prelude::*;
use clap::{crate_version, load_yaml, App};
use notify_rust::{Notification, Urgency};
use std::collections::HashMap;

#[cfg(test)]
mod tests;

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
    let soon = args
        .value_of("soon")
        .unwrap_or("15")
        .parse::<i64>()
        .unwrap_or(15);
    let max_cloudiness = args
        .value_of("cloudiness")
        .unwrap_or("25")
        .parse::<u64>()
        .unwrap_or(25);
    let notify = args.is_present("notify");
    // start our observatory via OWM
    let owm = &openweathermap::init(location, units, lang, apikey, poll);
    let mut iss: Option<open_notify::Receiver> = None;
    let mut io = i3status_ext::begin().unwrap();
    let mut format_str = String::new();
    let mut props: HashMap<&str, String> = HashMap::new();
    let mut current_spots: Vec<open_notify::Spot> = Vec::new();
    get_spots(&mut props, &Vec::new(), soon, true);
    let mut cloudiness: f64 = 0.0;
    let mut notify_soon: bool = true;
    let mut notify_visible: bool = true;
    let mut duration: i32 = 0;
    loop {
        // update current weather info if there is an update available
        match openweathermap::update(owm) {
            Some(response) => match response {
                Ok(w) => {
                    // remember cloudiness for spotting visibility
                    cloudiness = w.clouds.all;
                    // check if we have to start open_notify thread
                    if iss.is_none() {
                        iss = Some(open_notify::init(w.coord.lat, w.coord.lon, 0.0, 1));
                        // reset format string
                        format_str = format.to_string();
                    }
                    // get weather properties
                    get_weather(&mut props, &w, &units);
                }
                Err(e) => format_str = e,
            },
            None => (),
        }
        match iss {
            Some(ref iss) => match open_notify::update(iss) {
                Some(response) => match response {
                    Ok(s) => {
                        // remember duration of current spotting event in milliseconds for motification timeout
                        duration = match open_notify::find_current(&s) {
                            Some(s) => (s.duration.num_seconds() * 1000) as i32,
                            None => 0,
                        };
                        // rememeber current spotting events
                        current_spots = s;
                        // reset format string
                        format_str = format.to_string();
                    }
                    Err(e) => {
                        if e != openweathermap::LOADING {
                            format_str = e
                        }
                    }
                },
                None => (),
            },
            None => (),
        }
        // check for notifications
        if notify {
            if props["{iss_soonicon}"] != "" {
                if notify_soon {
                    Notification::new()
                        .summary("i3owm")
                        .body("ISS will bee visible soon!")
                        .urgency(Urgency::Low)
                        .show()
                        .unwrap();
                    notify_soon = false;
                }
            } else {
                notify_soon = true;
            }
            if props["{iss_duration}"] != "" {
                if notify_visible {
                    Notification::new()
                        .summary("i3owm")
                        .body("ISS is visible now!")
                        .timeout(duration)
                        .show()
                        .unwrap();
                    notify_visible = false;
                }
            } else {
                notify_visible = true;
            }
        }
        // continuously get spot properties
        get_spots(
            &mut props,
            &current_spots,
            soon,
            cloudiness <= max_cloudiness as f64,
        );
        // insert current properties and print json string or original line
        i3status_ext::update(
            &mut io,
            "i3owm",
            position,
            reverse,
            &format_string(&format_str, &props),
        )
        .unwrap();
    }
}

fn get_weather(
    props: &mut HashMap<&str, String>,
    current: &openweathermap::CurrentWeather,
    units: &str,
) {
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

    props.insert("{update}", update.format("%H:%M").to_string());
    props.insert("{city}", current.name.as_str().to_string());
    props.insert("{main}", current.weather[0].main.as_str().to_string());
    props.insert(
        "{description}",
        current.weather[0].description.as_str().to_string(),
    );
    props.insert("{icon}", icon(&current.weather[0].icon).to_string());
    props.insert("{pressure}", current.main.pressure.to_string());
    props.insert("{humidity}", current.main.humidity.to_string());
    props.insert("{wind}", current.wind.deg.to_string());
    props.insert("{wind_deg}", current.wind.deg.to_string());
    props.insert("{wind}", {
        let directions = ["N", "NO", "O", "SO", "S", "SW", "W", "NW"];
        directions[dir(current)].to_string()
    });
    props.insert("{wind_icon}", {
        let icons = ["↓", "↙", "←", "↖", "↑", "↗", "→", "↘"];
        icons[dir(current)].to_string()
    });
    props.insert("{deg_unit}", "°".to_string());
    props.insert("{wind_speed}", current.wind.speed.round().to_string());
    props.insert("{visibility}", current.visibility.to_string());
    props.insert("{visibility_km}", (current.visibility / 1000).to_string());
    props.insert(
        "{rain.1h}",
        match &current.rain {
            Some(r) => r.h1.unwrap_or(0.0).to_string(),
            None => "-".to_string(),
        }
        .to_string(),
    );
    props.insert(
        "{rain.3h}",
        match &current.rain {
            Some(r) => r.h3.unwrap_or(0.0).to_string(),
            None => "-".to_string(),
        },
    );
    props.insert(
        "{snow.1h}",
        match &current.snow {
            Some(r) => r.h1.unwrap_or(0.0).to_string(),
            None => "-".to_string(),
        },
    );
    props.insert(
        "{snow.3h}",
        match &current.snow {
            Some(r) => r.h3.unwrap_or(0.0).to_string(),
            None => "-".to_string(),
        },
    );
    props.insert("{temp_min}", current.main.temp_min.round().to_string());
    props.insert("{temp_max}", current.main.temp_max.round().to_string());
    props.insert("{feels_like}", current.main.temp.round().to_string());
    props.insert("{temp}", current.main.temp.round().to_string());
    props.insert(
        "{temp_unit}",
        match units {
            "standard" => "K",
            "metric" => "°C",
            "imperial" => "°F",
            _ => "",
        }
        .to_string(),
    );
    props.insert(
        "{speed_unit}",
        match units {
            "standard" => "m/s",
            "metric" => "m/s",
            "imperial" => "mi/h",
            _ => "",
        }
        .to_string(),
    );
}
static mut BLINK: bool = false;

fn get_spots(
    props: &mut HashMap<&str, String>,
    spots: &Vec<open_notify::Spot>,
    soon: i64,
    visibility: bool,
) {
    // some icons
    let satellite = "🛰".to_string();
    let eye = "👁".to_string();
    let empty = "".to_string();
    // get current and upcoming spotting event
    let current = open_notify::find_current(spots);
    let upcoming = open_notify::find_upcoming(spots);
    // clear all ISS properties
    props.insert("{iss_icon}", empty.clone());
    props.insert("{iss_iconblink}", empty.clone());
    props.insert("{iss_duration}", empty.clone());
    props.insert("{iss_soonicon}", empty.clone());
    props.insert("{iss_soon}", empty.clone());
    props.insert("{iss_risetime}", empty.clone());
    // check if we can see the sky
    if visibility {
        match current {
            // check if we have a current spotting event
            Some(spot) => {
                props.insert("{iss_icon}", satellite.clone());
                // for blinking we use a global static
                unsafe {
                    props.insert(
                        "{iss_iconblink}",
                        match BLINK {
                            true => satellite.clone(),
                            false => eye.clone(),
                        },
                    );
                    BLINK = !BLINK;
                }
                let duration = spot.risetime - Local::now();
                let duration = format!(
                    "+{:02}:{:02}",
                    duration.num_minutes(),
                    duration.num_seconds()
                );
                props.insert("{iss_duration}", duration);
            }
            // if not check if we have an upcoming spotting event
            None => match upcoming {
                Some(spot) => {
                    let duration = spot.risetime - Local::now();
                    if duration < chrono::Duration::minutes(soon) {
                        props.insert("{iss_soonicon}", satellite);
                        let duration = format!(
                            "-{:02}:{:02}",
                            duration.num_minutes(),
                            duration.num_seconds() % 60
                        );
                        props.insert("{iss_soon}", duration);
                    } else {
                        props.insert("{iss_risetime}", spot.risetime.format("%H:%M").to_string());
                    }
                }
                None => (),
            },
        }
    }
}

fn format_string(format: &str, props: &HashMap<&str, String>) -> String {
    let mut result: String = format.to_string();
    let mut iss: bool = false;
    // replace all keys by their values
    for (k, v) in props {
        let r = result.replace(k, v);
        if r != result {
            result = r;
            // tests if an '{iss_' key of value was inserted
            iss = iss || (k.contains("{iss_") && v != "");
        }
    }
    // insert space at '{iss_space}' if we inserted '{iss_' keys of value
    return result.replace(
        "{iss_space}",
        match iss {
            true => " ",
            false => "",
        },
    );
}
