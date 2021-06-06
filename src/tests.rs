// Note this useful idiom: importing names from outer (for mod tests) scope.
use super::*;

fn apikey() -> String {
    match std::env::var("OWM_APIKEY") {
        Ok(key) => key,
        Err(_r) => {
            eprintln!("error: set API-key with environment valiable OWM_APIKEY");
            "".to_string()
        }
    }
}

fn test_key(format: &str) {
    match openweathermap::blocking::weather("Berlin,DE", "metric", "en", &apikey()) {
        Ok(w) => {
            let mut props: HashMap<&str, String> = HashMap::new();
            get_weather(&mut props, &w, &"metric");
            match open_notify::blocking::spot(w.coord.lat, w.coord.lon, 0.0, 100) {
                Ok(spots) => {
                    get_spots(
                        &mut props,
                        &spots,
                        30,
                        &Visibility::VISIBLE,
                        None,
                        false,
                        &Level::RISE,
                    );
                    let s = format_string(format, &props);
                    // check if all keys have been replaced
                    assert!(s.find("{").is_none());
                    assert!(s.find("}").is_none());
                }
                Err(_e) => assert!(false),
            }
        }
        Err(_e) => assert!(false),
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
    test_key(&format);
}

#[test]
fn test_keydoublette() {
    test_key(&"{update}{update}");
}
