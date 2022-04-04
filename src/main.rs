use std::fs;

use colored::*;
use mmi_parser;

fn main() {
    println!("{}", "MMI Parser".bright_green().bold());
    let filename = "data/sample.txt";
    let contents = fs::read_to_string(filename).expect("Something went wrong reading the file");

    let cleaned = contents.split("\n").collect::<Vec<&str>>();

    for line in cleaned {
        let mmi = mmi_parser::parse_mmi(line);
        println!("{:#?}", mmi);
    }
}
