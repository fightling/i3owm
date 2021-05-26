extern crate chrono;
extern crate i3status_ext;
extern crate notify_rust;
extern crate openweathermap;

use chrono::prelude::*;
use clap::{crate_version, load_yaml, App};
use notify_rust::{Notification, Urgency};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

#[cfg(test)]
mod tests;

/// continuously inject weather into incoming json lines from i3status and pass through
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
    let blink = args.is_present("blink");
    let level = args.value_of("level").unwrap_or("soon");
    let level = match level {
        "watch" => Level::WATCH,
        "rise" => Level::RISE,
        _ => Level::SOON,
    };
    let test = args.is_present("test");
    // start our observatory via OWM
    let owm = &openweathermap::init(location, units, lang, apikey, poll);
    // open-notify receiver will get created if we get coordinates from weather update
    let mut iss: Option<open_notify::Receiver> = None;
    // start i3status parsing
    let mut io = match test {
        false => i3status_ext::begin().unwrap(),
        true => i3status_ext::begin_dummy().unwrap(),
    };
    // we may override
    let mut format_str = openweathermap::LOADING.to_string();
    let mut cloudiness: f64 = 0.0;
    let mut duration: i32 = 0;
    let mut notify_soon: bool = true;
    let mut notify_visible: bool = true;
    let mut blinking: bool = false;
    // latest spotting update
    let mut latest_spottings: Vec<open_notify::Spot> = Vec::new();
    // all fetched information
    let mut props: HashMap<&str, String> = HashMap::new();
    // insert empty values to all spotting properties
    get_spots(&mut props, &latest_spottings, soon, true, false, &level);
    loop {
        // update current weather info if there is an update available
        match openweathermap::update(owm) {
            Some(response) => match response {
                Ok(w) => {
                    // remember cloudiness for spotting visibility
                    cloudiness = w.clouds.all;
                    // check if we have to start open_notify thread
                    if iss.is_none() && format.contains("{iss_") {
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
                        latest_spottings = s;
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
        // continuously get spot properties
        match get_spots(
            &mut props,
            &latest_spottings,
            soon,
            cloudiness <= max_cloudiness as f64,
            blinking,
            &level,
        ) {
            // check for notifications
            Level::SOON => {
                if notify && notify_soon {
                    Notification::new()
                        .summary("i3owm")
                        .body("ISS will bee visible soon!")
                        .urgency(Urgency::Low)
                        .show()
                        .unwrap();
                    notify_soon = false;
                    notify_visible = true;
                }
            }
            Level::WATCH => {
                if notify && notify_visible {
                    Notification::new()
                        .summary("i3owm")
                        .body("ISS is visible now!")
                        .timeout(duration)
                        .show()
                        .unwrap();
                    notify_visible = false;
                }
            }
            _ => {
                notify_visible = true;
                notify_soon = true;
            }
        }
        // toggle blinking flag
        if blink {
            blinking = !blinking;
        }
        let output = format_string(&format_str, &props);
        if !test {
            // insert current properties and print json string or original line
            i3status_ext::update(&mut io, "i3owm", position, reverse, &output).unwrap();
        } else {
            println!("{}", output);
            thread::sleep(Duration::from_secs(1));
        }
    }
}

/// update properties map with new weather update data
/// #### Parameters
/// - `props`: property map to add data into
/// - `current`: current weather update
/// - `units`: maximum level of spotting display that is wanted (either `"standard"`, `"metric"` or `"imperial"`
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

#[derive(PartialEq, Eq)]
enum Level {
    NONE,
    /// only show duration while ISS is visible
    WATCH,
    /// show latency until ISS will be visible (includes 'watch')
    SOON,
    /// show time of next spotting event (includes 'soon' and 'watch')
    RISE,
}

/// update properties map with new open-notify data
/// #### Parameters
/// - `props`: property map to add data into
/// - `spots`: spotting events from open-notify
/// - `soon`: maximum duration in minutes which will be treated as *soon*
/// - `visibility`: `true` if sky is visible
/// - `blink`: `true` if icon shall blink while spotting
/// - `level`: maximum level of spotting display that is wanted
/// #### Return value
/// - level of spotting display that was used
fn get_spots(
    props: &mut HashMap<&str, String>,
    spots: &Vec<open_notify::Spot>,
    soon: i64,
    visibility: bool,
    blink: bool,
    level: &Level,
) -> Level {
    // some icons
    let satellite = "🛰".to_string();
    let eye = "👁".to_string();
    let empty = "".to_string();
    // get current and upcoming spotting event
    let current = open_notify::find_current(spots);
    let upcoming = open_notify::find_upcoming(spots);
    // check if we can see the sky
    if visibility {
        match current {
            // check if we have a current spotting event
            Some(spot) => {
                // insert (maybe blinking) icon
                props.insert(
                    "{iss_icon}",
                    match blink {
                        false => satellite.clone(),
                        true => eye.clone(),
                    },
                );
                // calculate duration until current spotting event
                let duration = spot.risetime - Local::now();
                // format duration (remove any leading zeros)
                let duration = format!(
                    "+{:02}:{:02}:{:02}",
                    duration.num_hours(),
                    duration.num_minutes() % 60,
                    duration.num_seconds() % 60
                )
                .replace("00:", "");
                // insert duration
                props.insert("{iss}", duration);
                return Level::WATCH;
            }
            // if not check if we have an upcoming spotting event
            None => match upcoming {
                Some(spot) => {
                    // calculate duration until upcoming spotting event
                    let duration = spot.risetime - Local::now();
                    // check if duration is soon
                    if duration < chrono::Duration::minutes(soon)
                        && [Level::SOON, Level::RISE].contains(&level)
                    {
                        // insert icon
                        props.insert("{iss_icon}", satellite.clone());
                        // format duration (remove any leading zeros)
                        let duration = format!(
                            "-{:02}:{:02}:{:02}",
                            duration.num_hours(),
                            duration.num_minutes() % 60,
                            duration.num_seconds() % 60
                        )
                        .replace("00:", "");
                        // insert duration
                        props.insert("{iss}", duration);
                        return Level::SOON;
                    } else if level == &Level::RISE {
                        // insert icon
                        props.insert("{iss_icon}", satellite.clone());
                        // format and insert time
                        if duration > chrono::Duration::days(1) {
                            props.insert("{iss}", spot.risetime.format("%x %R").to_string());
                        } else {
                            props.insert("{iss}", spot.risetime.format("%R").to_string());
                        }
                        return Level::RISE;
                    }
                }
                None => (),
            },
        }
    }
    // remove unused keys
    props.insert("{iss_icon}", empty.clone());
    props.insert("{iss}", empty.clone());
    return Level::NONE;
}

/// insert properties into format string
/// #### Parameters
/// - `format`: output format (string including some of the available keys)
/// - `props`: property map to add data into
/// #### Return value
/// - formatted string
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
