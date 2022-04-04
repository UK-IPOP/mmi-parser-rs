use std::{
    env,
    fs::{self, File},
    io::Write,
};

use colored::*;
use mmi_parser;
use serde_json::{self, Map, Value};

fn main() {
    println!("{}", "MMI Parser".bright_green().bold());
    println!("{}", "============".bright_green().bold());
    // let args: Vec<String> = env::args().collect();
    // let folder = &args[1];
    // println!("Reading files from: {}", folder);
    // match fs::read_dir(folder) {
    //     Ok(files) => {
    //         for file in files {
    //             let file = file.unwrap();
    //             let path = file.path();
    //             let filename = path.to_str().unwrap();
    //             println!("Reading file: {:}", filename);
    //             let contents = fs::read_to_string(filename).unwrap();
    //             let mut data: serde_json::Value =
    //                 serde_json::from_str(&contents).expect("could not parse json");
    //             parse_mmi_from_json(&mut data);
    //             let json_string = serde_json::to_string(data).expect("could not serialize json");
    //             println!("Writing file: {:}", filename);
    //             fs::write(filename, json_string)?;
    //         }
    //     }
    //     Err(e) => println!("{}", "Error: ".bright_red().bold() + e.to_string()),
    // }

    let file2 = "data/sample.json";
    let contents2 = fs::read_to_string(file2).expect("Something went wrong reading file2");
    let mmi_outputs: serde_json::Value = serde_json::from_str(&contents2).unwrap();
    let added_model = mmi_parser::parse_mmi_from_json(mmi_outputs);
    let json_string = serde_json::to_string(&added_model).unwrap();
    let mut f = File::create("data/sample_output.json").unwrap();
    f.write_all(json_string.as_bytes()).unwrap();
}
