use chrono::prelude::*;
use std::collections::HashMap;
pub use open_notify::DayTime;
use crate::level::Level;

#[derive(PartialEq, Eq)]
pub enum Visibility {
    VISIBLE,
    INVISIBLE,
}

impl Visibility {
    pub fn from_bool(visible: bool) -> Visibility {
        match visible {
            true => Visibility::VISIBLE,
            false => Visibility::INVISIBLE,
        }
    }
    fn is_visible(&self) -> bool {
        self == &Visibility::VISIBLE
    }
}

pub fn new_properties<'a>() -> HashMap<&'a str, String> {
    let mut props: HashMap<&str, String> = HashMap::new();
    // insert empty values to all spotting properties (so that we can replace them when ISS report is still missing)
    get_spots(
        &mut props,
        &Vec::new(),
        0,
        &Visibility::VISIBLE,
        None,
        false,
        &Level::RISE,
    );
    return props;
}

/// update properties map with new open-notify data
/// #### Parameters
/// - `props`: property map to add data into
/// - `spots`: spotting events from open-notify
/// - `soon`: maximum duration in minutes which will be treated as *soon*
/// - `visibility`: `true` if sky is visible
/// - `daytime`: some daytime if spotting at daytime should be skipped
/// - `blink`: `true` if icon shall blink while spotting
/// - `level`: maximum level of spotting display that is wanted
/// #### Return value
/// - level of spotting display that was used
pub fn get_spots(
    props: &mut HashMap<&str, String>,
    spots: &Vec<open_notify::Spot>,
    soon: i64,
    visibility: &Visibility,
    daytime: Option<&DayTime>,
    blink: bool,
    level: &Level,
) -> Level {
    // some icons
    let satellite = "🛰".to_string();
    let eye = "👁".to_string();
    let empty = "".to_string();
    // get current and upcoming spotting event
    let current = open_notify::find_current(spots, daytime, chrono::Local::now());
    let upcoming = open_notify::find_upcoming(spots, daytime, chrono::Local::now());
    // check if we can see the sky
    if visibility.is_visible() {
        match current {
            // check if we have a current spotting event
            Some(spot) => {
                // insert (maybe blinking) icon
                props.insert(
                    "{iss_icon}",
                    match blink {
                        false => satellite.clone(),
                        true => eye.clone(),
                    },
                );
                // calculate duration until current spotting event
                let duration = Local::now() - spot.risetime;
                // format duration (remove any leading zeros)
                let duration = format!(
                    "+{:02}:{:02}:{:02}",
                    duration.num_hours(),
                    duration.num_minutes() % 60,
                    duration.num_seconds() % 60
                )
                .replace("00:", "");
                // insert duration
                props.insert("{iss}", duration);
                return Level::WATCH;
            }
            // if not check if we have an upcoming spotting event
            None => match upcoming {
                Some(spot) => {
                    // calculate duration until upcoming spotting event
                    let duration = spot.risetime - Local::now();
                    // check if duration is soon
                    if duration < chrono::Duration::minutes(soon)
                        && [Level::SOON, Level::RISE, Level::FAR].contains(&level)
                    {
                        // insert icon
                        props.insert("{iss_icon}", satellite.clone());
                        // format duration (remove any leading zeros)
                        let duration = format!(
                            "-{:02}:{:02}:{:02}",
                            duration.num_hours(),
                            duration.num_minutes() % 60,
                            duration.num_seconds() % 60
                        )
                        .replace("00:", "");
                        // insert duration
                        props.insert("{iss}", duration);
                        return Level::SOON;
                    } else if [Level::RISE, Level::FAR].contains(&level) {
                        // insert icon
                        props.insert("{iss_icon}", satellite.clone());
                        // format and insert time
                        if duration > chrono::Duration::days(1) {
                            props.insert("{iss}", spot.risetime.format("%x %R").to_string());
                        } else {
                            props.insert("{iss}", spot.risetime.format("%R").to_string());
                        }
                        return Level::RISE;
                    }
                }
                None => {
                    if level == &Level::FAR {
                        match spots.last() {
                            Some(spot) => {
                                let duration = spot.risetime - Local::now();
                                // insert icon
                                props.insert("{iss_icon}", satellite.clone());
                                // format and insert time
                                props.insert("{iss}", format!(">{}", duration.num_days()));
                                return Level::FAR;
                            }
                            _ => ()
                        }
                    }
                },
            },
        }
    }
    // remove unused keys
    props.insert("{iss_icon}", empty.clone());
    props.insert("{iss}", empty.clone());
    return Level::NONE;
}
