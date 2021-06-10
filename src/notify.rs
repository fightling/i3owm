use std::time::Duration;
use notify_rust::{Notification, Urgency};
use crate::level::Level;

/// state of the current notification to prevent multiple notifications appear at the same time
pub struct Notify {
    suppress: bool,
    soon: bool,
    visible: bool,
}

impl Notify {
    /// create a notification state and initialize
    /// #### Parameters
    /// - `suppress`: supress any notification if true
    /// #### Return value
    /// returns the state instance
    pub fn new(suppress: bool) -> Notify {
        Notify {
            suppress: suppress,
            soon: true,
            visible: true,
        }
    }
    /// notify user about soon or current spotting ISS
    /// #### Parameters
    /// - `duration`: duration of the notification to appear
    /// - `level`: current spotting level
    pub fn notification(&mut self, duration: Duration, level: Level) {
        match level {
            Level::SOON => {
                if !self.suppress && self.soon {
                    Notification::new()
                        .appname("i3owm")
                        .summary("Upcoming: ISS spotting")
                        .body("ISS will bee visible soon!")
                        .urgency(Urgency::Low)
                        .show()
                        .unwrap();
                    self.soon = false;
                    self.visible = true;
                }
            }
            Level::WATCH => {
                if !self.suppress && self.visible {
                    Notification::new()
                        .appname("i3owm")
                        .summary("NOW: ISS spotting")
                        .body("ISS is visible NOW\nLook at the sky, weather seems ok!")
                        .timeout(duration.as_millis() as i32)
                        .show()
                        .unwrap();
                    self.visible = false;
                }
            }
            _ => {
                self.visible = true;
                self.soon = true;
            }
        }
    }
}
