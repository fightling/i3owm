use chrono::prelude::*;
use std::collections::HashMap;
pub use open_notify::{DayTime};

/// update properties map with new weather update data
/// #### Parameters
/// - `props`: property map to add data into
/// - `current`: current weather update
/// - `units`: maximum level of spotting display that is wanted (either `"standard"`, `"metric"` or `"imperial"`
pub fn get_weather(
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
