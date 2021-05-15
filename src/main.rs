extern crate chrono;
extern crate regex;
extern crate reqwest;
extern crate serde;
extern crate serde_json;

use clap::{crate_version, App, Arg, ArgMatches};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::error;
use text_io::read;

mod weather;

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
    let receiver = weather::receiver(city, units, lang, apikey);
    let converter = weather::converter(units);
    // read first two lines and ignore them
    // TODO: this code stinks!
    let line: String = read!("{}\n");
    println!("{}", line);
    let line: String = read!("{}\n");
    println!("{}", line);
    // remeber newest weather update and begin with offline message
    let mut current = "[offline]".to_string();
    loop {
        let mut line: String = read!("{}\n");
        // handle prefix comma
        if line.chars().next().unwrap() == ',' {
            line.remove(0);
            print!(",")
        }
        // update current weather info if there is an update available
        match weather::update(format, &receiver, &converter) {
            Some(update) => current = update,
            _ => (),
        }
        // insert current weather info and print json string or original line
        match insert(&line, "weather", position, reverse, &current) {
            Ok(new) => println!("{}", new),
            _ => println!("{}", line),
        }
    }
}

// insert  new named i3status item into json string at position from left or right (reverse)
fn insert(
    line: &str,
    name: &str,
    position: usize,
    reverse: bool,
    what: &str,
) -> Result<String, serde_json::Error> {
    //  i3status json entry in a struct
    #[derive(Serialize, Deserialize, Debug)]
    struct I3StatusItem {
        name: String,
        instance: Option<String>,
        markup: String,
        color: Option<String>,
        full_text: String,
    }
    // read all incoming entries
    let mut items: Vec<I3StatusItem> = serde_json::from_str(&line)?;
    // insert this one
    let w: I3StatusItem = I3StatusItem {
        full_text: what.to_string(),
        markup: "none".to_string(),
        name: name.to_string(),
        instance: None,
        color: None,
    };
    // insert at given position
    if reverse {
        items.insert(items.len() - 1 - position, w);
    } else {
        items.insert(position, w);
    }
    // format output back up json string
    return Ok(format_json(format!("{:?}", items)));
}

// preprocess output so that i3bar will eat it
fn format_json(line: String) -> String {
    // FIXIT: all the following replacements are needed because I just can not deal
    // with serde_json the right way :/ PLEASE HELP!
    let mut line = line;

    // remove all the 'Item' names
    // thought about using '#[serde(rename = "name")]' but could not make it work
    line = line.replace("I3StatusItem", "");
    // remove optional values which are 'None'
    // tried '#[serde(skip_serializing_if = "Option::is_none")]' but did not work.
    line = line.replace(", color: None", "");
    line = line.replace(", instance: None", "");
    // add quotations arround json names. can you setup serge_json doing that?
    line = line.replace("full_text", "\"full_text\"");
    line = line.replace("instance", "\"instance\"");
    line = line.replace("color", "\"color\"");
    line = line.replace("markup", "\"markup\"");
    line = line.replace("name", "\"name\"");
    // remove the 'Some()' envelop from all optional values
    let re = Regex::new(r"Some\((?P<v>[^\)]*)\)").unwrap();

    return re.replace_all(&line, "$v").to_owned().to_string();
}
