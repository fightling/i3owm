# I3owm [![Rust](https://github.com/fightling/i3owm/actions/workflows/rust.yml/badge.svg)](https://github.com/fightling/i3owm/actions/workflows/rust.yml)

rust implementation of Open Weather Map add-on for i3status

## Contents

<!-- MDTOC maxdepth:4 firsth1:2 numbering:0 flatten:0 bullets:1 updateOnSave:1 -->

- [Contents](#contents)   
- [Installation](#installation)   
- [Usage](#usage)   
   - [Program Arguments](#program-arguments)   
      - [Required Arguments](#required-arguments)   
      - [Options](#options)   
      - [Optional Arguments](#optional-arguments)   
   - [Display Format](#display-format)   
      - [Available Properties](#available-properties)   
      - [Testing your Display Format](#testing-your-display-format)   
      - [Weather](#weather)   
         - [Output](#output)   
      - [ISS Spotting Events](#iss-spotting-events)   
         - [Output](#output)   
      - [Complex Example](#complex-example)   
         - [Output](#output)   
   - [Integration into i3status](#integration-into-i3status)   
- [Reference Documentation](#reference-documentation)   
- [Links](#links)   
   - [Website](#website)   
   - [*github* repository](#github-repository)   
   - [on *crates.io*](#on-cratesio)   
- [License](#license)   

<!-- /MDTOC -->

## Installation

To install this from Rust community's crate registry, one must have installed *Rust* an *Cargo*.
Then enter this in the terminal:

```
cargo install i3owm
```

## Usage

### Program Arguments

#### Required Arguments

| Option              | Parameter  | Description |
|---------------------|------------|-------------|
| `-k`, `--apikey`    | `<apikey>` | Set OpenWeatherMap API key (see at https://openweathermap.org/price) |

#### Options

| Option              | Parameter  | Description |
|---------------------|------------|-------------|
| `-b`, `--blink`     |            | Let ISS icon blink when visible |
| `-h`, `--help`      |            | Prints help information |
| `-n`, `--notify`    |            | Show notifications about ISS getting visible |
| `-r`, `--reverse`   |            | Reverse position (from right) |
| `-t`, `--test`      |            | Do **not** process i3status from stdin, instead show formatted string |
| `-V`, `--version`   |            | Prints version information |

#### Optional Arguments

| Option              | Parameter  | Description | Default |
|---------------------|------------|-------------|---------|
| `-f`, `--format`    | `<format>` | Format string including one ore more of the following keys | `{city} {icon} {temp}{temp_unit}` |
| `-c`, `--location`  | `<location>` | City's name maybe followed by comma-separated 2-letter (state code for the USA locations and) country code (ISO3166) or city ID (see https://openweathermap.org/find) or geographical coordinate as comma-separated latitude and longitude. | `Berlin,DE` |
| `-C`, `--cloudiness` | `<cloudiness>` | Maximum cloudiness in percent at which ISS can be treated as visible | `25` |
| `-l`, `--lang`      | `<lang>`   | Two character language code of weather descriptions | `en` |
| `-L`, `--level`     | `<level>`  | ISS minimum show level: `watch`: duration when visible; `soon`: latency until visible; `rise`: spotting time | `soon` |
| `-P`, `--poll`      | `<poll>`   | Duration of polling period in minutes | `10` |
| `-p`, `--position`  | `<position>` | Position of output in JSON when wrapping i3status | `0` |
| `-s`, `--soon`      | `<soon>`   | Duration in minutes when ISS rising is "soon" in minutes | `15` |
| `-u`, `--units`     | `<units>`  | Use imperial units (`metric`, `imperial` or `standard`) | `metric` |

### Display Format

#### Available Properties

Choose your display format by inserting the following properties keys into your format string:

| Key               | Description | Example |
|-------------------|-------------|---------|
| `{city}`          |  City name | `Berlin` |
| `{main}`          |  Group of weather parameters | `Clouds` |
| `{description}`   |  Weather condition within the group | `scattered clouds` |
| `{icon}`          |  Weather icon | `🌞`,`🌛`, `🌤`, `⛅`, `🌧`,`🌦`,`🌩`,`❄`,`🌫` |
| `{pressure}`      |  Atmospheric pressure (sea level or ground level), hPa | `1010` |
| `{humidity}`      |  Humidity, % | `45` |
| `{wind}`          |  Wind direction | `N`, `NO`, `O`, `SO`, `S`, `SW`, `W`, `NW` |
| `{wind_icon}`     |  Wind direction as arrow icon | `↓`, `↙`, `←`, `↖`, `↑`, `↗`, `→`, `↘` |
| `{wind_speed}`    |  Wind speed | `m/s`, `mi/h` |
| `{wind_deg}`      |  Wind direction, degrees (meteorological) | `56` |
| `{deg_unit}`      |  Direction unit | `°` |
| `{visibility}`    |  Visibility, meter | `10000` |
| `{visibility_km}` |  Visibility, kilometer | `10` |
| `{rain.1h}`       |  Rain volume for the last 1 hour, mm | `12` |
| `{rain.3h}`       |  Rain volume for the last 3 hours, mm | `32` |
| `{snow.1h}`       |  Snow volume for the last 1 hour, mm | `11` |
| `{snow.3h}`       |  Snow volume for the last 3 hours, mm | `24` |
| `{temp_min}`      |  Minimum temperature at the moment | `-8` |
| `{temp_max}`      |  Maximum temperature at the moment | `10` |
| `{feels_like}`    |  Temperature for the human perception of weather | `8` |
| `{temp}`          |  Temperature | `15` |
| `{temp_unit}`     |  Temperature | `°C`, `°F`, `K` |
| `{speed_unit}`    |  Wind speed unit | `m/s` |
| `{update}`        |  Local time of last update | `12:45` |
| `{iss}`           |  ISS spotting time, latency or duration | `12:10`, `-02:21`, `+01:15` |
| `{iss_icon}`      |  show icon if ISS is visible | `🛰` |  
| `{iss_space}`     |  inserts space (`' '`) if any ISS information is displayed | ` ` |

#### Testing your Display Format

To make testing easy *i3owm* has an option `-t` (or `--test`) which disables processing of input from i3status and just produces the *i3owm* related output string.
We use that option to test some examples without *i3status*.

#### Weather

To get the weather we could use the following command line:

```
i3owm -t -k <key> -c Berlin,DE -f '{icon} {temp}{temp_unit} 💧{humidity}%'
```

##### Output

```
loading...
⛅ 11°C 💧55%
```

#### ISS Spotting Events

To get ISS spotting events we could use the following parameters:

```
i3owm -t -k <key> -c Berlin,DE -f '{iss_icon}{iss}'
```

##### Output

```
loading...
🛰+03:12
```

#### Complex Example

In this complex example we use the following parameters:

```
i3owm -tnb -Lrise -C100 -k <key> -cBerlin -f'{iss_icon}{iss}{iss_space}{icon} {temp}{temp_unit} 💧{humidity}% {wind_icon}{wind_speed}{speed_unit} ({update})'
```

##### Output

```
loading...
⛅ 14°C 💧70% ↑2m/s (13:47)
🛰16:37 ⛅ 14°C 💧70% ↑2m/s (13:47)
```

### Integration into i3status

To use your *i3owm* command line in your i3 configuration you need to remove option `-t` and append a pipe symbol `|` and your command line to your it (usually at `.config/i3/config`).

```
bar {
  status_command i3status | i3owm <your options>
}
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
