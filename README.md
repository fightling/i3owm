# I3owm

rust implementation of Open Weather Map add-on for i3status

Example usage in i3config:

```
bar {
status_command i3status | i3owm -p 2 -r -k <key> -c Berlin,DE -f '{icon} {temp}{temp_unit} 💧{humidity}%'
}
```

Output would be like:

```
⛅ 11°C 💧55%
```

## Get API key for OpenWeatherMap

Get your free API-key at https://openweathermap.org/price.

## Installation

To install this from Rust community's crate registry, one must install Rust. Then do this in the terminal:

```
cargo install i3owm
```

## Options

```
▶ i3owm --help              
i3owm 0.1.4
Patrick Hoffmann
Open Weather extension for i3status

Example usage in i3config:

  bar {
    status_command i3status | i3owm -p 2 -r -k <key> -c Berlin,DE -f '{icon} {temp}{temp_unit}
💧{humidity}%'
  }

Output would be like:

    ⛅ 11°C 💧55%


USAGE:
    i3owm [FLAGS] [OPTIONS]

FLAGS:
    -h, --help
            Prints help information

    -r, --reverse
            reverse position (from right)

    -V, --version
            Prints version information


OPTIONS:
    -k, --api-key <api>
            OpenWeatherMap API key
            (get one at https://openweathermap.org/api)

    -c, --city <city>
            location city
            (city's name, comma, 2-letter country code (ISO3166)) [default: Berlin,DE]

    -i, --city_id <city_id>
            location city ID
            (search your city at https://openweathermap.org/find and take ID out of the link you
            get)

    -f, --format <format>
            available keys are:
            {city}          City name
            {main}          Group of weather parameters (Rain, Snow, Extreme
                            etc.)
            {description}   Weather condition within the group
            {icon}          Weather icon
            {pressure}      Atmospheric pressure (on the sea level, if there is
                            no sea_level or grnd_level data), hPa
            {humidity}      Humidity, %
            {wind}          Wind direction, degrees (meteorological)
            {wind_icon}     Wind direction, (meteorological) as arrow icon
            {wind_speed}    Wind speed, {speed_unit}
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
             [default: {city} {icon} {current} {temp}{temp_unit} {humidity}%]

    -l, --lang <lang>
            two character language code of weather descriptions
            (default is 'en')

    -p, --position <position>
            position of output in JSON when wrapping i3status

    -u, --units <units>
            use imperial units [default: metric] [possible values: metric, imperial, standard]
```
