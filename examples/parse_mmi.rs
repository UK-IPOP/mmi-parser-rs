//! Example usage of default command line program if passed `data` folder.
//!
//! Looks specifically for the MMI sample data file.
//!
//! Results will be available in `data/MMI_sample_parsed.jsonl`

use std::fs::{self, File};
use std::io::{BufRead, BufReader, LineWriter, Write};

use colored::*;

const FOLDER: &str = "data";
const INPUT_TYPE: &str = "txt";

/// Main function.
fn main() {
    println!("{}", "MMI Parser".cyan().bold());
    println!("{}", "============".cyan().bold());
    println!("Reading files from: {}", FOLDER.cyan());

    match fs::read_dir(FOLDER) {
        Ok(files) => {
            for file in files {
                let file = file.expect("Could not process file.");
                let path = file.path();
                let filename = path.to_str().expect("could not parse file path");
                if filename.ends_with(INPUT_TYPE) && filename.contains("MMI") {
                    println!("Reading file: {}", filename.cyan());
                    let out_file_name = filename.replace(".txt", "_parsed.jsonl").to_string();
                    let out_file =
                        fs::File::create(&out_file_name).expect("could not create output file");
                    let mut out_writer = LineWriter::new(out_file);
                    // utilize read lines buffer
                    let file = File::open(&path).expect("could not open file");
                    let reader = BufReader::new(file);
                    for line in reader.lines().flatten() {
                        let result = mmi_parser::parse_mmi(&line);
                        if result.is_err() {
                            panic!("Example failed!")
                        }
                        let json_val = serde_json::to_value(result.unwrap())
                            .expect("unable to serialize json");
                        let json_string =
                            serde_json::to_string(&json_val).expect("unable to deserialize json");
                        out_writer.write_all(json_string.as_bytes()).unwrap();
                        out_writer.write_all(b"\n").unwrap();
                    }
                }
            }
        }
        Err(e) => println!("Error: {}", e),
    }
}
