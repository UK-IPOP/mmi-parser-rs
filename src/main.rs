//! Main Command Line Application for parsing fielded MMI data into
//! json format.
//!
//! A simple use case of the tool would look like:
//! ```bash
//! mmi_parser data
//! ```
//! which would parse all of the `.txt` files inside your data directory.
//!
//! The output of the program is a 1:1 mapping where a new file is created for each
//! file that is parsed.  This helps maintain indexing integrity when scanning MetaMap output.
//! The output files are in jsonlines format which allows you to buffer-read the files later and
//! also maintains the integrity of linking each line with its original fielded MMI output.
//! The output files have the same title as their .txt counterparts plus
//! a `_parsed` label to ensure clarity that they represent parsed data.

use std::error::Error;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, LineWriter, Write};

use colored::*;

use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use walkdir::WalkDir;

/// A simple program to parse fielded MMI output from txt into jsonl.
///
/// Expects to find `.txt` files inside the provided <FOLDER> and will
/// scan each line of MMI output from each file and transfer it to
/// a single line of json inside a parsed jsonlines file with the same name.
///
/// For more information see the [README](https://github.com/UK-IPOP/mmi-parser-rs) or the
/// [API Docs](https://docs.rs/mmi-parser/latest/mmi_parser/)
#[derive(Parser, Debug)]
#[clap(author, version)]
struct Cli {
    /// Folder to read files from
    folder: String,
}

fn initialize_progress(items: u64) -> ProgressBar {
    let pb = ProgressBar::new(items);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({eta})",
            )
            .progress_chars("#>-"),
    );
    pb
}
/// Main function.
fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    println!("{}", "MMI Parser".cyan().bold());
    println!("{}", "============".cyan().bold());
    println!(
        "{} {}",
        "Reading files from:".cyan(),
        cli.folder.cyan().bold()
    );

    let walker = WalkDir::new(&cli.folder);

    let mut file_count = 0;
    for e in walker.into_iter() {
        let name = e.unwrap();
        if name.file_name().to_str().unwrap().ends_with(".txt") {
            file_count += 1
        }
    }

    println!("{}", file_count);
    let bar = initialize_progress(file_count as u64);

    match fs::read_dir(cli.folder) {
        Ok(files) => {
            for file in files {
                let file = file.expect("Could not process file.");
                let path = file.path();
                let filename = path.to_str().expect("could not parse file path");
                if filename.ends_with(".txt") {
                    let out_file_name = filename.replace(".txt", "_parsed.jsonl").to_string();
                    let out_file =
                        fs::File::create(&out_file_name).expect("could not create output file");
                    let mut out_writer = LineWriter::new(out_file);
                    // utilize read lines buffer
                    let file = File::open(&path).expect("could not open file");
                    let reader = BufReader::new(file);
                    for line in reader.lines().flatten() {
                        let result = mmi_parser::parse_mmi(&line);
                        match result {
                            Ok(val) => {
                                let json_val =
                                    serde_json::to_value(val).expect("unable to serialize json");
                                let json_string = serde_json::to_string(&json_val)
                                    .expect("unable to deserialize json");
                                out_writer.write_all(json_string.as_bytes()).unwrap();
                                out_writer.write_all(b"\n").unwrap();
                            }
                            Err(e) => {
                                println!("{}", filename.red().bold());
                                return Err(Box::new(e));
                            }
                        }
                    }
                }
                bar.inc(1)
            }
        }
        Err(e) => {
            println!("couldn't scan directory");
            return Err(Box::new(e));
        }
    }
    bar.finish_and_clear();
    println!("{}", "Done.".cyan());
    Ok(())
}
