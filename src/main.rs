use std::{
    env,
    fs::{self},
};
use std::fs::remove_file;

use colored::*;

use serde_json::{self};

fn main() {
    println!("{}", "MMI Parser".bright_green().bold());
    println!("{}", "============".bright_green().bold());
    let args: Vec<String> = env::args().collect();
    let folder = &args[1];
    remove_file("data/sample_parsed.json").unwrap();
    remove_file("../data/sample_parsed.txt.json").unwrap();
    println!("Reading files from: {}", folder);
    match fs::read_dir(folder) {
        Ok(files) => {
            for file in files {
                let file = file.unwrap();
                let path = file.path();
                let filename = path.to_str().unwrap();
                if filename.ends_with(".json") {
                    println!("Reading file: {:}", filename);
                    let contents =
                        fs::read_to_string(file.path()).expect("Something went wrong reading file");
                    let json_data: serde_json::Value = serde_json::from_str(&contents).unwrap();
                    let modeled = mmi_parser::parse_mmi_from_json(json_data);
                    let json_string = serde_json::to_string(&modeled).unwrap();
                    let outname = filename.replace(".json", "_parsed.json").to_string();
                    println!("Writing file: {:}", outname.as_str());
                    fs::write(outname.as_str(), json_string.as_bytes())
                        .expect("Unable to write file");
                }
            }
        }
        Err(e) => println!("Error: {}", e.to_string()),
    }
    match fs::read_dir(folder) {
        Ok(files) => {
            for file in files {
                let file = file.unwrap();
                let path = file.path();
                let filename = path.to_str().unwrap();
                if filename.ends_with(".txt") {
                    println!("Reading file: {:}", filename);
                    let contents =
                        fs::read_to_string(file.path()).expect("Something went wrong reading file");
                    let result = mmi_parser::parse_mmi(&contents);
                    let json_val = serde_json::to_value(result).expect("unable to create json");
                    let json_string = serde_json::to_string(&json_val).unwrap();
                    let outname = filename.replace(".txt", "_parsed_txt.json").to_string();
                    println!("Writing file: {:}", outname.as_str());
                    fs::write(outname.as_str(), json_string.as_bytes())
                        .expect("Unable to write file");
                }
            }
        }
        Err(e) => println!("Error: {}", e.to_string()),
    }
}
