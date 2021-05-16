extern crate serde_json;

use regex::Regex;
use serde::{Deserialize, Serialize};
use text_io::read;

// i3status json entry in a struct
#[derive(Serialize, Deserialize, Debug)]
struct I3StatusItem {
    name: String,
    instance: Option<String>,
    markup: String,
    color: Option<String>,
    full_text: String,
}

pub fn begin() {
    // read first two lines, check and ignore them
    let line: String = read!("{}\n");
    println!("{}", line);
    assert!(line == "{\"version\":1}");
    let line: String = read!("{}\n");
    println!("{}", line);
    assert!(line == "[");
}

// insert new named i3status item into json string at position from left or right (reverse)
pub fn update(name: &str, position: usize, reverse: bool, what: &str) {
    // read one line from stdin
    let mut line: String = read!("{}\n");
    // check if begin() was called
    assert!(line != "{\"version\":1}");
    assert!(line != "[");
    // handle prefix comma
    if line.chars().next().unwrap() == ',' {
        line.remove(0);
        print!(",")
    }
    // read all incoming entries
    match serde_json::from_str(&line) {
        Ok(i) => {
            let mut items: Vec<I3StatusItem> = i;
            // insert this one
            let w: I3StatusItem = I3StatusItem {
                full_text: what.to_string(),
                markup: "none".to_string(),
                name: name.to_string(),
                instance: None,
                color: None,
            };
            // insert at given position
            if reverse {
                items.insert(items.len() - 1 - position, w);
            } else {
                items.insert(position, w);
            }
            // format output back up json string
            println!("{}", format_json(format!("{:?}", items)));
        }
        _ => println!("{}", line),
    }
}

// preprocess output so that i3bar will eat it
fn format_json(line: String) -> String {
    // FIXIT: all the following replacements are needed because I just can not deal
    // with serde_json the right way :/ PLEASE HELP!
    let line = line
        // remove all the 'Item' names
        // thought about using '#[serde(rename = "name")]' but could not make it work
        .replace("I3StatusItem", "")
        // remove optional values which are 'None'
        // tried '#[serde(skip_serializing_if = "Option::is_none")]' but did not work.
        .replace(", color: None", "")
        .replace(", instance: None", "")
        // add quotations arround json names. can you setup serge_json doing that?
        .replace("full_text", "\"full_text\"")
        .replace("instance", "\"instance\"")
        .replace("color", "\"color\"")
        .replace("markup", "\"markup\"")
        .replace("name", "\"name\"");
    // remove the 'Some()' envelop from all optional values
    let re = Regex::new(r"Some\((?P<v>[^\)]*)\)").unwrap();
    re.replace_all(&line, "$v").to_owned().to_string()
}
