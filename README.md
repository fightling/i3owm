# I3owm [![Rust](https://github.com/fightling/i3owm/actions/workflows/rust.yml/badge.svg)](https://github.com/fightling/i3owm/actions/workflows/rust.yml)

rust implementation of Open Weather Map add-on for i3status

## Usage Examples

### Display Weather

```
bar {
  status_command i3status | i3owm -rp2 -k <key> -c Berlin,DE -f '{icon} {temp}{temp_unit} 💧{humidity}%'
}
```

Output would be like:

```
⛅ 11°C 💧55%
```

### Display ISS Spotting Events

```
bar {
  status_command i3status | i3owm -rp2 -k <key> -c Berlin,DE -f '{iss_icon}{iss}'
}

```

Output would be like:

```
    🛰+03:12
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
Open Weather extension for i3status

USAGE:
    i3owm [FLAGS] [OPTIONS]

FLAGS:
    -b, --blink
            let ISS icon blink when visible

    -h, --help
            Prints help information

    -n, --notify
            if set shows notifications about ISS getting visible

    -r, --reverse
            reverse position (from right)

    -V, --version
            Prints version information


OPTIONS:
    -k, --apikey <apikey>
            OpenWeatherMap API key (see at https://openweathermap.org/api)

    -C, --cloudiness <cloudiness>
            maximum cloudiness in percent at which ISS can be treated as visible [default: 25]

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
            {iss}           ISS spotting time (HH:MM) or latency (-hh::mm::ss)
                            or duration (+hh::mm::ss)
            {iss_icon}      show 🛰 if ISS is visible
            {iss_space}     space (' ') if any ISS information is displayed
             [default: {city} {icon} {temp}{temp_unit}]

    -l, --lang <lang>
            two character language code of weather descriptions [default: en]

    -L, --level <level>
            watch = only show duration while ISS is visible
            soon = show latency until ISS will be visible (includes 'watch')
            rise = show time of next spotting event (includes 'soon' and 'watch')
             [default: soon] [possible values: watch, soon, rise]

    -c, --location <location>
            city's name maybe followed by comma-separated 2-letter (state code for the USA locations
            and) country code (ISO3166) or city ID (see https://openweathermap.org/find) or
            geographical coordinate as comma-separated latitude and longitude. [default: Berlin,DE]

    -P, --poll <poll>
            duration of polling period in minutes [default: 10]

    -p, --position <position>
            position of output in JSON when wrapping i3status

    -s, --soon <soon>
            duration in minutes when ISS rising is "soon" in minutes [default: 15]

    -u, --units <units>
            use imperial units [default: metric] [possible values: metric, imperial, standard]


EXAMPLE:
    Within your i3 configutation file just pipe the i3status output to i3owm like in the
    following example (you will have top replace <key> with your API key to make it work):

    bar {
      status_command i3status | i3owm -p 2 -r -k <key> -c Berlin,DE -f '{icon} {temp}{temp_unit}'
    }

    Example output: ⛅ 11°C

    bar {
      status_command i3status | i3owm -rp2 -k <key> -c Berlin,DE -f '{iss_icon}{iss}'
    }

    Example output: 🛰+03:12
```


## Reference Documentation

Beside this introduction there is a reference documentation for the source code which can be found [here](https://docs.rs/i3owm).

## Links

### Website

This README tastes better at [i3owm.thats-software.com](http://i3owm.thats-software.com).

### *github* repository

For the source code see [this repository](https://github.com/fightling/i3owm) at *github.com*.

### on *crates.io*

Published at [*crates.io*](https://crates.io/crates/i3owm).

## License

i3status_ext is licensed under the *MIT license* (LICENSE-MIT or http://opensource.org/licenses/MIT)
