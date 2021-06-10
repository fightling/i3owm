extern crate chrono;
extern crate i3status_ext;
extern crate notify_rust;
extern crate openweathermap;

use clap::{crate_version, load_yaml, App};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

mod level;
mod notify;
mod spot;
mod weather;

use level::Level;
use notify::Notify;
use spot::*;
use weather::*;

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
    let dayspot = args.is_present("daytime");
    let notify = args.is_present("notify");
    let blink = args.is_present("blink");
    let level = args.value_of("level").unwrap_or("soon");
    let level = match level {
        "watch" => Level::WATCH,
        "rise" => Level::RISE,
        "far" => Level::FAR,
        _ => Level::SOON,
    };
    let test = args.is_present("test");
    let prevision = args
        .value_of("prevision")
        .unwrap_or("100")
        .parse::<u8>()
        .unwrap_or(100);
    // start our observatory via OWM
    let owm = &openweathermap::init(location, units, lang, apikey, poll);
    // open-notify receiver will get created if we get coordinates from weather update
    let mut iss: Option<open_notify::Receiver> = None;
    // start i3status parsing
    let mut io = match test {
        false => i3status_ext::begin().unwrap(),
        true => i3status_ext::begin_dummy().unwrap(),
    };
    // we may override format for error messages
    let mut format_str = openweathermap::LOADING.to_string();
    // remember visibility from weather report for ISS spotting
    let mut visible: bool = false;
    // remember daytime from weather report for ISS spotting
    let mut daytime: DayTime;
    let mut dt: Option<&DayTime> = None;
    // remember duration of current spotting event in milliseconds for motification timeout
    let mut duration = Duration::from_millis(0);
    // state of current notification
    let mut notify = Notify::new(notify);
    // create blinking flag
    let mut blinking: bool = false;
    // latest spotting update
    let mut spottings: Vec<open_notify::Spot> = Vec::new();
    // all fetched information
    let mut props: HashMap<&str, String> = new_properties();
    loop {
        // update current weather info if there is an update available
        match openweathermap::update(owm) {
            Some(response) => match response {
                Ok(w) => {
                    // remember cloudiness for spotting visibility
                    visible = w.clouds.all <= max_cloudiness as f64;
                    // remember daytime from current weather if wanted
                    if !dayspot {
                        daytime = DayTime::from_utc(w.sys.sunrise, w.sys.sunset);
                        dt = Some(&daytime);
                    }
                    // check if we have to start open_notify thread
                    if iss.is_none() && format.contains("{iss_") {
                        iss = Some(open_notify::init(w.coord.lat, w.coord.lon, 0.0, prevision, 90));
                    }
                    // get weather properties
                    get_weather(&mut props, &w, &units);
                    // reset format string
                    format_str = format.to_string();
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
                        duration = match open_notify::find_current(&s, dt, chrono::Local::now()) {
                            Some(s) => Duration::from_millis(s.duration.num_milliseconds() as u64),
                            None => Duration::from_millis(0),
                        };
                        // rememeber current spotting events
                        spottings = s;
                        // reset format string
                        format_str = format.to_string();
                    }
                    Err(e) => {
                        // do not show "loading..." twice
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
        let level = get_spots(
            &mut props,
            &spottings,
            soon,
            visible,
            dt,
            blinking,
            &level,
        );
        // check if we shall generate a notification
        notify.notification(duration, level);
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

/// insert properties into format string
/// #### Parameters
/// - `format`: output format (string including some of the available keys)
/// - `props`: property map to get data to insert from
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
