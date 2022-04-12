use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, LineWriter, Write};
use std::path::Path;
use std::str::FromStr;

use colored::*;

use clap::Parser;
use serde_json::{self, Value};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Folder to read files from
    folder: String,

    /// File type to search for inside folder
    #[clap(short, long, value_name = "INPUT FILE TYPE", default_value_t = String::from("txt"))]
    input_type: String,
}

fn main() {
    println!("{}", "MMI Parser".cyan().bold());
    println!("{}", "============".cyan().bold());
    let cli = Cli::parse();
    println!("Reading files from: {}...", cli.folder.cyan());

    match fs::read_dir(cli.folder) {
        Ok(files) => {
            for file in files {
                let file = file.expect("Could not process file.");
                let path = file.path();
                let filename = path.to_str().expect("could not parse file path");
                if filename.ends_with(&cli.input_type) {
                    println!("Reading file: {}", filename);
                    match cli.input_type.to_uppercase().as_str() {
                        "TXT" => {
                            let out_file_name =
                                filename.replace(".txt", "_parsed.jsonl").to_string();
                            let outfile = fs::File::create(&out_file_name)
                                .expect("could not create output file");
                            let mut out_writer = LineWriter::new(outfile);
                            // utilize read lines buffer
                            let file = File::open(&path).expect("could not open file");
                            let reader = BufReader::new(file);
                            for line in reader.lines().flatten() {
                                let result = mmi_parser::parse_mmi(&line);
                                let json_val =
                                    serde_json::to_value(result).expect("unable to serialize json");
                                let json_string = serde_json::to_string(&json_val)
                                    .expect("unable to deserialize json");
                                out_writer.write_all(json_string.as_bytes()).unwrap();
                                out_writer.write_all(b"\n").unwrap();
                            }
                        }
                        "JSON" => {
                            // read whole json file
                            let contents = fs::read_to_string(&path)
                                .expect("Something went wrong reading file");
                            let json_data: serde_json::Value = serde_json::from_str(&contents)
                                .expect("could not serialize input file");
                            let results = parse_mmi_from_json(json_data);
                            let json_string = serde_json::to_string(&results)
                                .expect("could not deserialize results");
                            let outname = filename.replace(".json", "_parsed.json").to_string();
                            fs::write(outname.as_str(), json_string.as_bytes())
                                .expect("Unable to write file");
                        }
                        _ => panic!("Unexpected input type"),
                    }
                }
            }
        }
        Err(e) => println!("Error: {}", e.to_string()),
    }
}

// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = fs::File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn parse_mmi_from_json(mut data: Value) -> Value {
    data.get_mut("encounter")
        .expect("encounter not found")
        .as_object_mut()
        .expect("could not find encounter as object")
        .iter_mut()
        .for_each(|(_, encounter)| {
            encounter
                .get_mut("scm-notes")
                .expect("Could not find scm-notes key")
                .as_array_mut()
                .expect("Could not make scm-notes as array")
                .iter_mut()
                .for_each(|note| {
                    let mut results: Vec<Value> = Vec::new();
                    note.get_mut("metamap_output")
                        .expect("Could not find metamap_output key")
                        .as_array_mut()
                        .expect("Could not make metamap_output as array")
                        .iter_mut()
                        .for_each(|mm_output| {
                            let prepared = mm_output.as_str().expect("Could not make string");
                            let mmi_output = mmi_parser::parse_mmi(prepared);
                            let serde_mmi_output = serde_json::to_value(&mmi_output)
                                .expect("Could not serialize mmi_output");
                            results.push(serde_mmi_output);
                        });
                    note.as_object_mut()
                        .expect("couldn't create note obj")
                        .insert("mmi_output".to_string(), serde_json::Value::Array(results));
                })
        });
    data
}
