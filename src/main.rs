extern crate chrono;
extern crate regex;
extern crate reqwest;
extern crate serde;
extern crate serde_json;

use chrono::prelude::*;
use clap::{crate_version, App, Arg};
use http::StatusCode;
use std::error;
use std::sync::mpsc;
use std::{thread, time};
use text_io::read;

mod weather;

//use weather::insert_weather;

// let satellite = "🛰";

fn main() -> Result<(), Box<dyn error::Error>> {
    // fetch arguments
    let args = App::new("i3owm")
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
                .long_about( weather::help() )
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
        .get_matches();
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
    // read first two lines and ignore them
    let line: String = read!("{}\n");
    println!("{}", line);
    let line: String = read!("{}\n");
    println!("{}", line);
    // start observatory
    let weather = weather::init(city, units, lang, apikey);
    loop {
        let mut line: String = read!("{}\n");
        let prefix = line.chars().next().unwrap() == ',';
        if prefix {
            line.remove(0);
        }
        if prefix {
            print!(",")
        }
        match weather::update(weather, units) {
            Some(properties) => replace(properties,line),
            _ => (),
        }
        println!("{}", line);
    }
}

fn replace( data : HashMap<&str, String>, &String line ) {
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
