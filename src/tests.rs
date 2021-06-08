// Note this useful idiom: importing names from outer (for mod tests) scope.
use super::*;
use regex::Regex;

const satellite: &str = "🛰";
const eye: &str = "👁";

fn apikey() -> String {
    match std::env::var("OWM_APIKEY") {
        Ok(key) => key,
        Err(_r) => {
            eprintln!("error: set API-key with environment valiable OWM_APIKEY");
            "".to_string()
        }
    }
}

fn test_key(format: &str, level: &Level, n: u8) -> String {
    match openweathermap::blocking::weather("Berlin,DE", "metric", "en", &apikey()) {
        Ok(w) => {
            let mut props: HashMap<&str, String> = HashMap::new();
            get_weather(&mut props, &w, &"metric");
            match open_notify::blocking::spot(w.coord.lat, w.coord.lon, 0.0, n) {
                Ok(spots) => {
                    get_spots(
                        &mut props,
                        &spots,
                        30,
                        &Visibility::VISIBLE,
                        None,
                        false,
                        &level,
                    );
                    let s = format_string(format, &props);
                    // check if all keys have been replaced
                    assert!(s.find("{").is_none());
                    assert!(s.find("}").is_none());
                    return s;
                }
                Err(_e) => {
                    assert!(false);
                    return String::new();
                }
            }
        }
        Err(_e) => {
            assert!(false);
            return String::new();
        }
    }
}

#[test]
fn test_allkeys() {
    // take long help to have a sample with all keys in it
    let yaml = load_yaml!("arg.yaml");
    let mut format = Vec::new();
    App::from(yaml).write_long_help(&mut format).unwrap();
    let format: String = std::str::from_utf8(format.as_slice()).unwrap().to_string();
    // cut out the example at the end because it contains `{` and `}` which are not marking any key names
    let format: &str = &format[0..format.find("EXAMPLE:").unwrap()];
    test_key(&format, &Level::RISE, 100);
}

#[test]
fn test_keydoublette() {
    test_key(&"{update}{update}", &Level::RISE, 100);
}

#[test]
fn test_iss_levels() {
    let far = Regex::new(r"🛰\d+" ).unwrap();
    let rise = Regex::new(r"🛰(\d?\d\.\d?\d\.\d\d\d\d)?(\d?\d:)?\d?\d" ).unwrap();
    let soon = Regex::new(r"🛰-((\d\d:)?\d\d:)?\d\d" ).unwrap();
    let watch = Regex::new(r"[🛰👁]\+((\d?\d:)?\d?\d:)?\d\d" ).unwrap();
    let s = test_key(&"{iss_icon}{iss}{iss_space}", &Level::FAR, 100);
    assert!(far.is_match(&s) || rise.is_match(&s) || soon.is_match(&s) || watch.is_match(&s));
    let s = test_key(&"{iss_icon}{iss}{iss_space}", &Level::RISE, 100);
    assert!( s.is_empty() || rise.is_match(&s) || soon.is_match(&s) || watch.is_match(&s));
    let s = test_key(&"{iss_icon}{iss}{iss_space}", &Level::SOON, 100);
    assert!( s.is_empty() || soon.is_match(&s) || watch.is_match(&s));
    let s = test_key(&"{iss_icon}{iss}{iss_space}", &Level::WATCH, 100);
    assert!( s.is_empty() || watch.is_match(&s));
}
