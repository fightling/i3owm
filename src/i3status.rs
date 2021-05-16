extern crate serde_json;

use regex::Regex;
use serde::{Deserialize, Serialize};
use text_io::read;

pub fn begin() {
    // read first two lines and ignore them
    // TODO: this code stinks!
    let line: String = read!("{}\n");
    println!("{}", line);
    let line: String = read!("{}\n");
    println!("{}", line);
}
pub fn read() -> String {
    let mut line: String = read!("{}\n");
    // handle prefix comma
    if line.chars().next().unwrap() == ',' {
        line.remove(0);
        print!(",")
    }
    return line;
}
// insert  new named i3status item into json string at position from left or right (reverse)
pub fn write(line: &str, name: &str, position: usize, reverse: bool, what: &str) {
    //  i3status json entry in a struct
    #[derive(Serialize, Deserialize, Debug)]
    struct I3StatusItem {
        name: String,
        instance: Option<String>,
        markup: String,
        color: Option<String>,
        full_text: String,
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
    let mut line = line;

    // remove all the 'Item' names
    // thought about using '#[serde(rename = "name")]' but could not make it work
    line = line.replace("I3StatusItem", "");
    // remove optional values which are 'None'
    // tried '#[serde(skip_serializing_if = "Option::is_none")]' but did not work.
    line = line.replace(", color: None", "");
    line = line.replace(", instance: None", "");
    // add quotations arround json names. can you setup serge_json doing that?
    line = line.replace("full_text", "\"full_text\"");
    line = line.replace("instance", "\"instance\"");
    line = line.replace("color", "\"color\"");
    line = line.replace("markup", "\"markup\"");
    line = line.replace("name", "\"name\"");
    // remove the 'Some()' envelop from all optional values
    let re = Regex::new(r"Some\((?P<v>[^\)]*)\)").unwrap();

    return re.replace_all(&line, "$v").to_owned().to_string();
}
