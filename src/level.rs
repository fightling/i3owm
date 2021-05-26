#[derive(PartialEq, Eq)]
pub enum Level {
    NONE,
    /// only show duration while ISS is visible
    WATCH,
    /// show latency until ISS will be visible (includes 'watch')
    SOON,
    /// show time of next spotting event (includes 'soon' and 'watch')
    RISE,
}
