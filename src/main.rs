extern crate chrono;
extern crate i3status_ext;
extern crate notify_rust;
extern crate openweathermap;
//#[macro_use]
extern crate clap;

use clap::Parser;
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

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Location city name, city ID or coordinate
    ///
    /// City's name maybe followed by comma-separated 2-letter (state code for the USA locations and) country code (ISO3166) or city ID (see https://openweathermap.org/find) or geographical coordinate as comma-separated latitude and longitude.
    #[clap(short='c', long, value_parser, default_value_t = String::from("Berlin,DE"))]
    location: String,

    /// OpenWeatherMap API key (see at https://openweathermap.org/api)
    #[clap(short = 'k', long, value_parser)]
    apikey: String,

    /// Display format string
    ///
    /// Format string including one ore more of the following keys
    ///
    ///   {city}          City name
    ///
    ///   {main}          Group of weather parameters (Rain, Snow, Extreme etc.)
    ///
    ///   {description}   Weather condition within the group
    ///
    ///   {icon}          Weather icon
    ///
    ///   {pressure}      Atmospheric pressure (on the sea level, if there is no sea_level or grnd_level data), hPa
    ///
    ///   {humidity}      Humidity, %
    ///
    ///   {wind}          Wind direction as N, NW, W, SW, S, SO, O or NO
    ///
    ///   {wind_icon}     Wind direction as arrow icon
    ///
    ///   {wind_speed}    Wind speed, {speed_unit}
    ///
    ///   {wind_deg}      Wind direction, degrees (meteorological)
    ///
    ///   {deg_unit}      Direction unit (degrees: °)
    ///
    ///   {visibility}    Visibility, meter
    ///
    ///   {visibility_km} Visibility, kilometer
    ///
    ///   {rain.1h}       Rain volume for the last 1 hour, mm
    ///
    ///   {rain.3h}       Rain volume for the last 3 hours, mm
    ///
    ///   {snow.1h}       Snow volume for the last 1 hour, mm
    ///
    ///   {snow.3h}       Snow volume for the last 3 hours, mm
    ///
    ///   {temp_min}      Minimum temperature at the moment. This is minimal currently observed temperature (within large megalopolises and urban areas), {temp_unit}
    ///
    ///   {temp_max}      Maximum temperature at the moment. This is maximal currently observed temperature (within large megalopolises and urban areas), {temp_unit}
    ///
    ///   {feels_like}    Temperature. This temperature parameter accounts for the human perception of weather, {temp_unit}
    ///
    ///   {temp}          Temperature, {temp_unit}
    ///
    ///   {temp_unit}     Temperature (standard=K, metric=°C, imperial=°F)
    ///
    ///   {speed_unit}    Wind speed unit
    ///                   (standard=m/s, metric=m/s, imperial=mi/h)
    ///
    ///   {update}        Local time of last update, HH:MM
    ///
    ///   {iss}           ISS spotting time (HH:MM) or latency (-hh::mm::ss) or duration (+hh::mm::ss)
    ///
    ///   {iss_icon}      show 🛰  if ISS is visible
    ///
    ///   {iss_space}     space (' ') if any ISS information is displayed
    #[clap(short, long, value_parser, default_value_t=String::from("{city} {icon} {temp}{temp_unit}"))]
    format: String,

    /// Position of output in JSON when wrapping i3status
    #[clap(short, long, value_parser, default_value_t = 0)]
    position: usize,

    /// Two character language code of weather descriptions
    #[clap(short, long, value_parser, default_value_t = String::from("en"))]
    lang: String,

    /// Reverse position (from right)
    #[clap(short, long, action)]
    reverse: bool,

    /// Use imperial units
    #[clap(short, long, value_parser, default_value_t = String::from("metric"))]
    units: String,

    ///  Duration of polling period in minutes
    #[clap(short = 'P', long, value_parser, default_value_t = 10)]
    poll: u64,

    /// Duration in minutes when ISS rising is "soon" in minutes
    #[clap(short, long, value_parser, default_value_t = 15)]
    soon: i64,

    /// Maximum cloudiness in percent at which ISS can be treated as visible
    #[clap(short = 'C', long = "cloudiness", value_parser, default_value_t = 25)]
    max_cloudiness: u64,

    /// Show ISS spotting events when they are at daytime
    #[clap(short = 'D', long = "daytime", action)]
    dayspot: bool,

    /// ISS spotting level
    ///
    /// watch = only show duration while ISS is visible
    /// soon = show latency until ISS will be visible (includes 'watch')
    /// rise = show time of next spotting event (includes 'soon' and 'watch')
    /// far = show prediction time in days if no prediction available
    #[clap(short='L', long, value_enum, default_value_t = Level::SOON)]
    level: Level,

    /// Let ISS icon blink when visible
    #[clap(short, long, action)]
    blink: bool,

    /// Show notifications about ISS getting visible
    #[clap(short, long, action)]
    notify: bool,

    /// Do not process i3status from stdin, instead show formatted string
    #[clap(short, long, action)]
    test: bool,

    /// Number of ISS spottings that will be fetched from open-notify.org
    #[clap(short = 'T', long, value_parser, default_value_t = 100)]
    prevision: u8,
}
/// continuously inject weather into incoming json lines from i3status and pass through
fn main() {
    // fetch arguments
    let args = Args::parse();
    // start our observatory via OWM
    let owm = &openweathermap::init(
        &args.location,
        &args.units,
        &args.lang,
        &args.apikey,
        args.poll,
    );
    // open-notify receiver will get created if we get coordinates from weather update
    let mut iss: Option<open_notify::Receiver> = None;
    // start i3status parsing
    let mut io = match args.test {
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
    let mut notify = Notify::new(args.notify);
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
                    visible = w.clouds.all <= args.max_cloudiness as f64;
                    // remember daytime from current weather if wanted
                    if !args.dayspot {
                        daytime = DayTime::from_utc(w.sys.sunrise, w.sys.sunset);
                        dt = Some(&daytime);
                    }
                    // check if we have to start open_notify thread
                    if iss.is_none() && args.format.contains("{iss_") {
                        iss = Some(open_notify::init(
                            w.coord.lat,
                            w.coord.lon,
                            0.0,
                            args.prevision,
                            90,
                        ));
                    }
                    // get weather properties
                    get_weather(&mut props, &w, &args.units);
                    // reset format string
                    format_str = args.format.to_string();
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
                        format_str = args.format.to_string();
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
            args.soon,
            visible,
            dt,
            blinking,
            &args.level,
        );
        // check if we shall generate a notification
        notify.notification(duration, level);
        // toggle blinking flag
        if args.blink {
            blinking = !blinking;
        }
        let output = format_string(&format_str, &props);
        if !args.test {
            // insert current properties and print json string or original line
            i3status_ext::update(&mut io, "i3owm", args.position, args.reverse, &output).unwrap();
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
