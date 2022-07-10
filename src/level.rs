use clap::ValueEnum;
use std::fmt;

#[derive(PartialEq, Eq, ValueEnum, Clone, Debug)]
pub enum Level {
    /// None
    NONE,
    /// only show duration while ISS is visible
    WATCH,
    /// show latency until ISS will be visible (includes 'watch')
    SOON,
    /// show time of next spotting event (includes 'soon' and 'watch')
    RISE,
    /// show prediction time if no spotting event was found
    FAR,
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Level::NONE => "none",
                Level::WATCH => "watch",
                Level::SOON => "soon",
                Level::RISE => "rise",
                Level::FAR => "far",
            }
        )
    }
}
