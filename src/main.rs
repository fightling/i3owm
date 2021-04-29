extern crate regex;
extern crate reqwest;
extern crate serde;
extern crate serde_json;

use clap::{App, Arg};
use http::StatusCode;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::error;
use text_io::read;

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    let client = reqwest::Client::new();
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
                .about("OpenWeatherMap API key (get one at https://openweathermap.org/api)")
                .short('k')
                .long("api-key")
                .takes_value(true),
            Arg::new("city")
                .about("location city")
                .short('c')
                .long("city")
                .takes_value(true),
            Arg::new("format")
                .about(
                    "format string. available keys are:
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
{speed}         Wind speed, meter/sec
{visibility}    Visibility, meter
{visibility_km} vVisibility, kilometer
{rain.1h}       Rain volume for the last 1 hour, mm
{rain.3h}       Rain volume for the last 3 hours, mm
{snow.1h}       Snow volume for the last 1 hour, mm
{snow.3h}       Snow volume for the last 3 hours, mm
{temp_min}      Minimum temperature at the moment. This is minimal
                currently observed temperature (within large
                megalopolises and urban areas), Kelvin
{temp_min_c}    Like {temp_min} but in Celsius
{temp_max}      Maximum temperature at the moment. This is maximal
                currently observed temperature (within large
                megalopolises and urban areas), Kelvin
{temp_max_c}    Like {temp_max} but in Celsius
{feels_like}    Temperature. This temperature parameter accounts
                for the human perception of weather, Kelvin
{feels_like_c}  Like {feels_like} but in Celsius
{temp}          Temperature,  Kelvin
{temp_c}        Like {temp} but in Celsius",
                )
                .short('f')
                .long("format")
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
        ])
        .get_matches();

    let city: &str = args.value_of("city").unwrap_or("Berlin");
    let apikey: &str = args.value_of("api").unwrap_or("");
    let format: &str = args
        .value_of("format")
        .unwrap_or("{icon} {current} {temp_c} °C");
    let url: String = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}",
        city, apikey
    );
    let position: usize = args
        .value_of("position")
        .unwrap_or("0")
        .parse::<usize>()
        .unwrap();
    let reverse = args.is_present("reverse");

    let mut line: String = read!("{}\n");
    println!("{}", line);
    line = read!("{}\n");
    println!("{}", line);

    loop {
        line = read!("{}\n");

        let prefix = line.chars().next().unwrap() == ',';

        if prefix {
            line.remove(0);
        }
        let res = client.get(&url).send().await?;
        match res.status() {
            StatusCode::UNAUTHORIZED => println!("error: please provide API key"),
            StatusCode::OK => {
                // Move and borrow value of `res`
                let body = res.text().await?;
                //println!("Body:\n{}", body);

                // Parse the string of data into serde_json::Value.
                let v: Value = serde_json::from_str(&body)?;

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
                let result: String = format
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
                        "{temp_min_c}",
                        &(v["main"]["temp_min"].as_f64().unwrap() - 273.15)
                            .round()
                            .to_string(),
                    )
                    .replace(
                        "{temp_max}",
                        &v["main"]["temp_max"].as_f64().unwrap().round().to_string(),
                    )
                    .replace(
                        "{temp_max_c}",
                        &(v["main"]["temp_max"].as_f64().unwrap() - 273.15)
                            .round()
                            .to_string(),
                    )
                    .replace(
                        "{feels_like}",
                        &v["main"]["temp"].as_f64().unwrap().round().to_string(),
                    )
                    .replace(
                        "{feels_like_c}",
                        &(v["main"]["temp"].as_f64().unwrap() - 273.15)
                            .round()
                            .to_string(),
                    )
                    .replace(
                        "{temp}",
                        &v["main"]["temp"].as_f64().unwrap().round().to_string(),
                    )
                    .replace(
                        "{temp_c}",
                        &(v["main"]["temp"].as_f64().unwrap() - 273.15)
                            .round()
                            .to_string(),
                    );

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
                let after = re.replace_all(&r, "$v");

                if prefix {
                    print!(",")
                }
                println!("{}", after);
            }
            _ => print!("error: could not reach OpenWeatherMap website"),
        }
    }
}
