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
i3owm 0.1.5
▶ i3owm --help
Open Weather extension for i3status

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
    -k, --apikey <apikey>
            OpenWeatherMap API key (see at https://openweathermap.org/api)

    -c, --city <city>
            city's name, comma, 2-letter country code (ISO3166) [default: Berlin,DE]

    -f, --format <format>
            format string including one ore more of the following keys
            {city}          City name
            {main}          Group of weather parameters (Rain, Snow, Extreme
                            etc.)
            {description}   Weather condition within the group
            {icon}          Weather icon
            {pressure}      Atmospheric pressure (on the sea level, if there is
                            no sea_level or grnd_level data), hPa
            {humidity}      Humidity, %
            {wind}          Wind direction as N, NW, W, SW, S, SO, O or NO
            {wind_icon}     Wind direction as arrow icon
            {wind_speed}    Wind speed, {speed_unit}
            {wind_deg}      Wind direction, degrees (meteorological)
            {deg_unit}      Direction unit (degrees: °)
            {visibility}    Visibility, meter
            {visibility_km} Visibility, kilometer
            {rain.1h}       Rain volume for the last 1 hour, mm
            {rain.3h}       Rain volume for the last 3 hours, mm
            {snow.1h}       Snow volume for the last 1 hour, mm
            {snow.3h}       Snow volume for the last 3 hours, mm
            {temp_min}      Minimum temperature at the moment. This is minimal currently observed
                            temperature (within large megalopolises and urban areas), {temp_unit}
            {temp_max}      Maximum temperature at the moment. This is maximal
                            currently observed temperature (within large
                            megalopolises and urban areas), {temp_unit}
            {feels_like}    Temperature. This temperature parameter accounts
                            for the human perception of weather, {temp_unit}
            {temp}          Temperature, {temp_unit}
            {temp_unit}     Temperature
                            (standard=K, metric=°C, imperial=°F)
            {speed_unit}    Wind speed unit
                            (standard=m/s, metric=m/s, imperial=mi/h)
            {update}        Local time of last update, HH:MM
             [default: {city} {icon} {temp}{temp_unit}]

    -i, --id <id>
            location city ID (see https://openweathermap.org/find)

    -l, --lang <lang>
            two character language code of weather descriptions [default: en]

    -p, --position <position>
            position of output in JSON when wrapping i3status

    -u, --units <units>
            use imperial units [default: metric] [possible values: metric, imperial, standard]


EXAMPLE:
    Within your i3 configutation file just pipe the i3status output to i3owm like in the
    following example (you will have top replace <key> with your API key to make it work):

    bar {
      status_command i3status | i3owm -p 2 -r -k <key> -c Berlin,DE -f '{icon} {temp}{temp_unit}'
    }

    Example output: ⛅ 11°C
```
