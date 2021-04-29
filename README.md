# I3owm

rust implementation of Open Weather Map add-on for i3status

Example usage in i3config:

```
bar {
status_command i3status | i3owm -p 2 -r -k <key> -c Berlin -f '{icon} {temp_c}°C 💧{humidity}%'
}
```

Output would be like:

```
⛅ 11°C 💧55%
```

## Get API key for OpenWeatherMap

Get your free API-key at https://openweathermap.org/price.


## Build project

```
cargo build
```

## Install project

Within the project directory run:

```
cargo install --path .
```

Then add this to your `.profile`:

```bash
# set PATH so it includes user's private ~/.cargo/bin if it exists
if [ -d "$HOME/.cargo/bin" ] ; then
    PATH="$HOME/.cargo/bin:$PATH"
fi
```

## Options

```
▶ i3owm -h
i3owm 0.1.0
Patrick Hoffmann
Open Weather extension for i3status

Example usage in i3config:

  bar {
    status_command i3status | i3owm -p 2 -r -k <key> -c Berlin -f '{icon} {temp_c}°C 💧{humidity}%'
  }

Output would be like:

    ⛅ 11°C 💧55%


USAGE:
    i3owm [FLAGS] [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -r, --reverse    reverse position (from right)
    -V, --version    Prints version information

OPTIONS:
    -k, --api-key <api>          OpenWeatherMap API key (get one at https://openweathermap.org/api)
    -c, --city <city>            location city
    -f, --format <format>        format string. available keys are:
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
                                 currently observed temperature (within large megalopolises and
                                 urban areas), Kelvin
                                 {temp_min_c}    Like {temp_min} but in Celsius
                                 {temp_max}      Maximum temperature at the moment. This is maximal
                                 currently observed temperature (within large megalopolises and
                                 urban areas), Kelvin
                                 {temp_max_c}    Like {temp_max} but in Celsius
                                 {feels_like}    Temperature. This temperature parameter accounts
                                 for the human perception of weather, Kelvin
                                 {feels_like_c}  Like {feels_like} but in Celsius
                                 {temp}          Temperature,  Kelvin
                                 {temp_c}        Like {temp} but in Celsius
    -p, --position <position>    position of output in JSON when wrapping i3status
```
