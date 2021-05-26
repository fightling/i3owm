use std::time::Duration;
use notify_rust::{Notification, Urgency};
use crate::level::Level;

pub struct Notify {
    suppress: bool,
    soon: bool,
    visible: bool,
}

impl Notify {
    pub fn new(suppress: bool) -> Notify {
        Notify {
            suppress: suppress,
            soon: false,
            visible: false,
        }
    }
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
