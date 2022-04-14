//! This crate exists to support the primary functions of the
//! MMI parser command line tool.
//!
//!
//! The primary reference for the field information is found
//! [here](https://lhncbc.nlm.nih.gov/ii/tools/MetaMap/Docs/MMI_Output_2016.pdf)
//! and relies on MetaMap 2016 or newer.
//!
//! The main functionality is encompassed in [`MmiOutput`], [`AaOutput`], and [`parse_mmi`].
//!
//! For questions on implementations of the parsing algorithms for specific sections,
//! please consult the [source](https://github.com/UK-IPOP) which contains well-labeled
//! and fairly documented functions to parse each type.

extern crate core;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{self, Display};
use std::str::FromStr;

/// Splits the provided string reference on vertical bar (pipe symbol)
/// and collects split into vector.
fn split_text(text: &str) -> Vec<&str> {
    text.split('|').collect()
}

/// Labels the parts of the pipe-split string using MMI field labels.
/// Returns a hashmap of field names as keys and their values from the vector.
fn label_mmi_parts(parts: Vec<&str>) -> HashMap<&str, &str> {
    let mut map = HashMap::new();
    map.insert("id", parts[0]);
    map.insert("mmi", parts[1]);
    map.insert("score", parts[2]);
    map.insert("name", parts[3]);
    map.insert("cui", parts[4]);
    map.insert("semantic_types", parts[5]);
    map.insert("triggers", parts[6]);
    map.insert("location", parts[7]);
    map.insert("positional_info", parts[8]);
    map.insert("tree_codes", parts[9]);
    map
}

/// Parses out semantic type field by removing brackets and splitting on commas.
fn parse_semantic_types(semantic_types: &str) -> Vec<String> {
    let cleaned = semantic_types
        .strip_prefix('[')
        .unwrap()
        .strip_suffix(']')
        .unwrap();
    cleaned.split(',').map(|x| x.to_string()).collect()
}

/// Enumeration for Location options.
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Location {
    TI,
    AB,
    TX,
    Tiab,
}

impl FromStr for Location {
    type Err = ValueError;
    /// Parses a Location type from a string reference.
    fn from_str(s: &str) -> std::result::Result<Location, ValueError> {
        match s.to_uppercase().as_str() {
            "TI" => Ok(Location::TI),
            "AB" => Ok(Location::AB),
            "TX" => Ok(Location::TX),
            "TI;AB" => Ok(Location::Tiab),
            _ => Err(ValueError),
        }
    }
}

/// Parses the tree codes by splitting string reference on semicolon and
/// collecting into vector.
/// Returns Optional Vector because tree-codes could be None.
fn parse_tree_codes(codes: &str) -> Option<Vec<String>> {
    if codes.is_empty() {
        return None;
    }
    Some(codes.split(';').map(|x| x.to_string()).collect())
}

/// Utility function for splitting a string reference on a given pattern
/// while *ignoring* inside quotes.
///  
/// This was necessary due to MMI output containing literal-quoted strings with
/// split characters ("," or "-") inside them.
fn split_with_quote_context(x: &str, pattern: char) -> Vec<String> {
    let mut is_in_quotes = false;
    let mut start_position = 0;
    let final_position = x.len();
    let mut parts: Vec<String> = Vec::new();
    for (i, c) in x.chars().enumerate() {
        if c == '\"' {
            is_in_quotes = !is_in_quotes;
        } else if c == pattern && !is_in_quotes {
            parts.push(x[start_position..i].to_string());
            start_position = i + 1;
        } else if i == final_position - 1 {
            // last part
            parts.push(x[start_position..final_position].to_string());
        }
    }
    parts
}

/// Struct to represent Trigger information.
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Trigger {
    /// UMLS concept name
    pub name: String,
    /// location of text
    pub loc: Location,
    /// number of the utterance within the location (starting with 1)
    pub loc_position: i32,
    /// the actual text
    pub text: String,
    /// determined by MedPost Tagger or Lexical Lookup
    pub part_of_speech: String,
    /// True if text is considered negated by MetaMap
    pub negation: bool,
}

/// Utility function to convert string reference to boolean.
///
/// Will panic if string reference is not "1" or "0" because
/// that is the expected output from MetaMap.
fn parse_bool(x: &str) -> bool {
    match x {
        "1" => true,
        "0" => false,
        _ => panic!("Unexpected boolean: {}", x),
    }
}

impl Trigger {
    /// New function to initialize a Trigger.
    pub fn new(
        n: &str,
        loc: &str,
        loc_pos: &str,
        t: &str,
        part_of_speech: &str,
        negation: &str,
    ) -> Trigger {
        Trigger {
            name: n.replace('\"', ""),
            loc: Location::from_str(loc).expect("unable to parse Location"),
            loc_position: loc_pos
                .parse::<i32>()
                .expect("unable to parse integer from location"),
            text: t.replace('\"', ""),
            part_of_speech: part_of_speech.replace('\"', ""),
            negation: parse_bool(negation),
        }
    }
}

/// Parses [`Trigger`] instances from string reference.
fn parse_triggers(info: &str) -> Vec<Trigger> {
    let trigger_list = split_with_quote_context(info, ',');
    trigger_list
        .iter()
        .map(|t| {
            let clean = t.trim_start_matches('[').trim_end_matches(']');
            let parts = split_with_quote_context(clean, '-');
            Trigger::new(
                &parts[0], &parts[1], &parts[2], &parts[3], &parts[4], &parts[5],
            )
        })
        .collect()
}

/// Splits on commas *not* inside brackets.
/// Similar to [split_with_quote_context] except applies to brackets instead of quotes.
fn split_with_bracket_context(x: &str) -> Vec<String> {
    let mut is_in_brackets = false;
    let mut start_position = 0;
    let final_position = x.len();
    let mut parts: Vec<String> = Vec::new();
    for (i, c) in x.chars().enumerate() {
        if c == '[' {
            is_in_brackets = !is_in_brackets;
        } else if c == ']' {
            is_in_brackets = !is_in_brackets;
            if i == final_position - 1 {
                // last part
                parts.push(x[start_position..final_position].to_string());
            }
        } else if c == ',' && !is_in_brackets {
            parts.push(x[start_position..i].to_string());
            start_position = i + 1;
        }
    }
    parts
}

/// Parses bracketed information for positional information.
/// Used in [parse_positional_info]
fn parse_bracketed_info(x: &str) -> Vec<i32> {
    let parts = x
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split('/')
        .map(|x| x.parse::<i32>().expect("could not parse integer"))
        .into_iter()
        .collect::<Vec<i32>>();
    parts
}

/// Positional Information type options
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PositionalInfoType {
    A,
    B,
    C,
    D,
}

/// Tags positional information based on conditions
/// listed in 9a-9d of the reference [document](https://lhncbc.nlm.nih.gov/ii/tools/MetaMap/Docs/MMI_Output_2016.pdf).
fn tag_pos_info(x: &str) -> (bool, bool, bool) {
    // series of different conditions
    let mut has_brackets = false;
    let mut has_comma_inside_brackets = false;
    let mut has_comma_outside_brackets = false;
    let mut in_bracket = false;
    for c in x.chars() {
        // encountered bracket somewhere
        if c == '[' {
            has_brackets = true;
            in_bracket = true;
        } else if c == ']' {
            in_bracket = false;
        } else if c == ',' && !in_bracket {
            has_comma_outside_brackets = true;
        } else if c == ',' && in_bracket {
            has_comma_inside_brackets = true;
        }
    }
    (
        has_brackets,
        has_comma_inside_brackets,
        has_comma_outside_brackets,
    )
}

/// Categorizes the positional information tagged from
/// [tag_pos_info] into a specific category.
fn categorize_positional_info(
    has_brackets: bool,
    has_comma_inside_brackets: bool,
    has_comma_outside_brackets: bool,
) -> PositionalInfoType {
    if !has_comma_outside_brackets && !has_comma_inside_brackets {
        PositionalInfoType::A
    } else if (has_comma_inside_brackets || has_comma_outside_brackets) && !has_brackets {
        PositionalInfoType::B
    } else if has_brackets && !has_comma_inside_brackets && has_comma_outside_brackets {
        PositionalInfoType::C
    } else if has_comma_outside_brackets && has_brackets && has_comma_inside_brackets {
        PositionalInfoType::D
    } else {
        panic!("could not parse positional information.")
    }
}

/// Structure for Position representing start index, length, and Position Type.
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Position {
    /// Start position
    pub start: i32,
    /// Length of matched text
    pub length: i32,
    /// Type of match
    pub case: PositionalInfoType,
}

impl Position {
    /// Initialize new position.
    pub fn new(start: i32, length: i32, case: PositionalInfoType) -> Position {
        Position {
            start,
            length,
            case,
        }
    }
}

/// Parses out a Vector of [`Position`] types from a string reference.
fn parse_positional_info(info: &str) -> Vec<Position> {
    let tags = tag_pos_info(info);
    let category = categorize_positional_info(tags.0, tags.1, tags.2);
    match category {
        PositionalInfoType::A => info
            .split(';')
            .map(|x| {
                let parts = x
                    .split('/')
                    .map(|x| x.parse::<i32>().expect(x))
                    .collect::<Vec<i32>>();
                Position::new(parts[0], parts[1], PositionalInfoType::A)
            })
            .collect(),
        PositionalInfoType::B => info
            .split(';')
            .flat_map(|f| {
                f.split(',')
                    .map(|x| {
                        let parts = x
                            .split('/')
                            .map(|x| x.parse::<i32>().expect("could not parse integer"))
                            .collect::<Vec<i32>>();
                        Position::new(parts[0], parts[1], PositionalInfoType::B)
                    })
                    .collect::<Vec<Position>>()
            })
            .collect::<Vec<Position>>(),
        PositionalInfoType::C => info
            .split(';')
            .flat_map(|f| {
                f.split(',')
                    .map(|x| {
                        let parts = parse_bracketed_info(x);
                        Position::new(parts[0], parts[1], PositionalInfoType::C)
                    })
                    .collect::<Vec<Position>>()
            })
            .collect::<Vec<Position>>(),
        PositionalInfoType::D => info
            .split(';')
            .flat_map(|f| {
                let split_parts = split_with_bracket_context(f);
                split_parts
                    .iter()
                    .flat_map(|y| {
                        y.split(',')
                            .map(|x| {
                                let parts = parse_bracketed_info(x);
                                Position::new(parts[0], parts[1], PositionalInfoType::D)
                            })
                            .collect::<Vec<Position>>()
                    })
                    .collect::<Vec<Position>>()
            })
            .collect(),
    }
}

/// Main struct for entire library.
/// Represents an entire fielded MMI record as one type.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct MmiOutput {
    /// unique identifier
    pub id: String,
    /// always MMI
    pub mmi: String,
    /// score of concept relevance, 0-1000, 1000 being perfect
    pub score: f64,
    /// name of the concept matched
    pub name: String,
    /// CUI for identified UMLS concept
    pub cui: String,
    /// Semantic Type abbreviations
    pub semantic_types: Vec<String>,
    /// Triggers for MMI to flag this concept
    pub triggers: Vec<Trigger>,
    /// Location of concept
    pub location: Location,
    /// Positional information of concept
    pub positional_info: Vec<Position>,
    /// Optional MeSH [tree code(s)](https://www.nlm.nih.gov/mesh/meshhome.html)
    pub tree_codes: Option<Vec<String>>,
}

impl MmiOutput {
    /// Parses a hashmap into MMiOutput field types.
    /// Utilizes all other functionality defined in this module
    /// to assemble/parse each field into its appropriate format and types.
    ///
    /// While this function is useful for building [`MmiOutput`] types,
    /// [parse_mmi] will probably be **much** more practical since it
    /// accepts a string reference and does the field tagging/mapping for you.
    pub fn new(parts: HashMap<&str, &str>) -> Self {
        let id = parts["id"].to_string();
        let mmi = parts["mmi"].to_string();
        let score = parts["score"]
            .parse::<f64>()
            .expect("couldn't parse score value to float");
        let name = parts["name"].to_string();
        let cui = parts["cui"].to_string();
        let semantic_types = parse_semantic_types(parts["semantic_types"]);
        let triggers = parse_triggers(parts["triggers"]);
        let location = Location::from_str(parts["location"]).unwrap();
        let positional_info = parse_positional_info(parts["positional_info"]);
        let tree_codes = parse_tree_codes(parts["tree_codes"]);
        MmiOutput {
            id,
            mmi,
            score,
            name,
            cui,
            semantic_types,
            triggers,
            location,
            positional_info,
            tree_codes,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Output {
    MMI(MmiOutput),
    AA(AaOutput),
}

/// A better alternative to [`MmiOutput::new`] or [`AaOutput::new`]
/// Takes a string reference, splits it on vertical bar (pipe) characters,
/// labels each item with its corresponding field name,
/// passes labeled data into [`MmiOutput::new`] or [`AaOutput::new`].
///
/// This is used to scan over lines in fielded MMI output text files in the main CLI.
/// It detects whether the record is MMI or not by looking at the second item in the pipe-delimited
/// vector and whether it matches MMI, AA/UA, or neither.
///
/// Arguments:
/// * text: a string reference representing a single line of MMI/AA output
///
/// Returns:
/// * Result<Output, Error>: An enumeration with MMI::MmiOutput and AA::AaOutput options. Could return
/// error if a valid option is not found in the second vector position.
///
/// This effectively converts *each* fielded MMI **line** into an [`Output`] of either MMI or AA type..
/// For example:
///
/// ```rust
/// use std::fs::File;
/// use std::io::{self, prelude::*, BufReader};
/// use std::error::Error;
///
/// fn main() -> Result<(), Box<dyn Error>> {
///     let file = File::open("data/MMI_sample.txt")?;
///     // or for AA records
///     // let file = File::open("data/AA_sample.txt"?);
///     let reader = BufReader::new(file);
///
///     for line in reader.lines() {
///         let record = line?;
///         let result = mmi_parser::parse_mmi(record.as_str());
///         println!("{:?}", result?); // must use debug
///     }
///
///     Ok(())
/// }
/// ```
pub fn parse_mmi(text: &str) -> Result<Output> {
    let parts = split_text(text);
    match parts[1] {
        "MMI" => {
            let fields = label_mmi_parts(parts);
            let output = MmiOutput::new(fields);
            Ok(Output::MMI(output))
        }
        "AA" | "UA" => {
            let fields = label_aa_parts(parts);
            let output = AaOutput::new(fields);
            Ok(Output::AA(output))
        }
        _ => Err(ValueError),
    }
}

/// An alternative Result implementation using [`ValueError`]
pub type Result<T> = std::result::Result<T, ValueError>;

/// ValueError occurs when an invalid value was provided
#[derive(Debug)]
pub struct ValueError;

impl Display for ValueError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Received an unexpected value")
    }
}

impl Error for ValueError {}

/// Which type of abbreviation (AA) record exists, either AA or UA (user-defined)
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum AbbreviationType {
    /// MetaMap Acronyms and Abbreviations
    AA,
    /// User defined Acronyms and Abbreviations
    UA,
}

impl FromStr for AbbreviationType {
    type Err = ValueError;
    /// Parses an Abbreviation Type from a string reference.
    fn from_str(s: &str) -> std::result::Result<Self, ValueError> {
        match s.to_uppercase().as_str() {
            "AA" => Ok(AbbreviationType::AA),
            "UA" => Ok(AbbreviationType::UA),
            _ => Err(ValueError),
        }
    }
}

/// Abbreviation and Acronym position information
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct AaPosInfo {
    pub start: i32,
    pub length: i32,
}

impl AaPosInfo {
    /// New function to create positional info type from two str references
    pub fn new(s: &str, l: &str) -> Self {
        let ss = s
            .parse::<i32>()
            .expect("could not parse start position to integer");
        let ll = l.parse::<i32>().expect("could not parse length to integer");
        AaPosInfo {
            start: ss,
            length: ll,
        }
    }
}

/// Main "Secondary" type of program
/// Acronyms and Abbreviations detected by MetaMap
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct AaOutput {
    /// Unique identifier
    pub id: String,
    /// Abbreviation type: either MetaMap defined or User-defined
    pub abbreviation_type: AbbreviationType,
    /// Short form of the acronym/abbreviation
    pub short_form: String,
    /// Long form or expansion
    pub long_form: String,
    /// number of tokens (including whitespace) in short form
    pub short_token_count: i32,
    /// number of characters in short form
    pub short_character_count: i32,
    /// number of tokens (including whitespace) in long form
    pub long_token_count: i32,
    /// number of characters in long form
    pub long_character_count: i32,
    /// starting position of short form followed by ":" followed by character length of short form
    pub positional_info: AaPosInfo,
}

impl AaOutput {
    /// New function for AA types
    ///
    /// Mostly handles parsing strings to integers, also tags the abbreviation type and positional information.
    pub fn new(parts: HashMap<&str, &str>) -> Self {
        let id = parts["id"].to_string();
        let abbreviation_type = AbbreviationType::from_str(parts["abbreviation_type"])
            .expect("couldn't parse abbreviation type (AA or UA)");
        let short_form = parts["short_form"].to_string();
        let long_form = parts["long_form"].to_string();
        let short_token_count = parts["short_token_count"]
            .parse::<i32>()
            .expect("couldn't parse string to integer.");
        let short_character_count = parts["short_character_count"]
            .parse::<i32>()
            .expect("couldn't parse string to integer.");
        let long_token_count = parts["long_token_count"]
            .parse::<i32>()
            .expect("couldn't parse string to integer.");
        let long_character_count = parts["long_character_count"]
            .parse::<i32>()
            .expect("couldn't parse string to integer.");
        let position_parts: Vec<&str> = parts["positional_info"].split(':').collect();
        let positional_info = AaPosInfo::new(position_parts[0], position_parts[1]);
        AaOutput {
            id,
            abbreviation_type,
            short_form,
            long_form,
            short_token_count,
            short_character_count,
            long_token_count,
            long_character_count,
            positional_info,
        }
    }
}

/// Labels AA records with the corresponding field names
pub fn label_aa_parts(parts: Vec<&str>) -> HashMap<&str, &str> {
    let mut map: HashMap<&str, &str> = HashMap::new();
    map.insert("id", parts[0]);
    map.insert("abbreviation_type", parts[1]);
    map.insert("short_form", parts[2]);
    map.insert("long_form", parts[3]);
    map.insert("short_token_count", parts[4]);
    map.insert("short_character_count", parts[5]);
    map.insert("long_token_count", parts[6]);
    map.insert("long_character_count", parts[7]);
    map.insert("positional_info", parts[8]);
    map
}

#[cfg(test)]
mod tests {
    use core::panic;

    use super::*;

    #[test]
    fn test_parse_bool() {
        assert!(parse_bool("1"));
        assert!(!parse_bool("0"));
    }

    #[test]
    #[should_panic]
    fn test_invalid_parse_bool() {
        parse_bool("123");
    }

    #[test]
    fn test_split_with_bracket_context() {
        let s1 = "[4061/10,4075/11],[4061/10,4075/11]";
        let r1 = split_with_bracket_context(s1);
        assert_eq!(r1, vec!["[4061/10,4075/11]", "[4061/10,4075/11]"])
    }

    // this is a beefy integration test of the
    // `tag_pos_info` and the `categorize_positional_info` functions
    #[test]
    fn test_pos_info_categorization() {
        // ex 1 type C
        let s1 = "[4061/10,4075/11],[4061/10,4075/11]";
        let r1 = tag_pos_info(s1);
        let cat = categorize_positional_info(r1.0, r1.1, r1.2);

        assert_eq!(r1, (true, true, true));
        assert_eq!(cat, PositionalInfoType::D);

        let s1 = "117/5;122/4";
        let r1 = tag_pos_info(s1);
        let cat = categorize_positional_info(r1.0, r1.1, r1.2);

        assert_eq!(r1, (false, false, false));
        assert_eq!(cat, PositionalInfoType::A);

        let s1 = "117/5";
        let r1 = tag_pos_info(s1);
        let cat = categorize_positional_info(r1.0, r1.1, r1.2);

        assert_eq!(r1, (false, false, false));
        assert_eq!(cat, PositionalInfoType::A);

        let s1 = "117/5,122/4,113/2";
        let r1 = tag_pos_info(s1);
        let cat = categorize_positional_info(r1.0, r1.1, r1.2);

        assert_eq!(r1, (false, false, true));
        assert_eq!(cat, PositionalInfoType::B);

        let s1 = "[122/4],[117/6]";
        let r1 = tag_pos_info(s1);
        let cat = categorize_positional_info(r1.0, r1.1, r1.2);

        assert_eq!(r1, (true, false, true));
        assert_eq!(cat, PositionalInfoType::C);
    }

    #[test]
    fn test_quote_splitter() {
        let sample = "[\"Drug, NOS\"-tx-33-\"medicine\"-noun-0,\"Drug, NOS\"-tx-31-\"medicine\"-noun-0,\"Drug - NOS\"-tx-29-\"medication\"-noun-0,\"Drug, NOS\"-tx-5-\"drug\"-noun-0]";
        let r = split_with_quote_context(sample, ',');
        assert_eq!(r.len(), 4);
        for x in r {
            let r2 = split_with_quote_context(&x, '-');
            assert_eq!(6, r2.len()); // sextuple
        }
    }

    #[test]
    fn test_split_text() {
        let sample = "24119710|MMI|637.30|Isopoda|C0598806|[euka]|";
        assert_eq!(
            split_text(sample),
            ["24119710", "MMI", "637.30", "Isopoda", "C0598806", "[euka]", ""]
        );
    }

    #[test]
    fn test_name_parts() {
        let sample = "24119710|MMI|637.30|Isopoda|C0598806|[euka]|[\"Isopod\"-ab-1-\"isopod\"-adj-0,\"Isopoda\"-ti-1-\"Isopoda\"-noun-0]|TI;AB|228/6;136/7|B01.050.500.131.365.400";
        assert_eq!(label_mmi_parts(split_text(sample)), {
            let mut map = HashMap::new();
            map.insert("id", "24119710");
            map.insert("mmi", "MMI");
            map.insert("score", "637.30");
            map.insert("name", "Isopoda");
            map.insert("cui", "C0598806");
            map.insert("semantic_types", "[euka]");
            map.insert(
                "triggers",
                "[\"Isopod\"-ab-1-\"isopod\"-adj-0,\"Isopoda\"-ti-1-\"Isopoda\"-noun-0]",
            );
            map.insert("location", "TI;AB");
            map.insert("positional_info", "228/6;136/7");
            map.insert("tree_codes", "B01.050.500.131.365.400");
            map
        });
    }

    #[test]
    fn test_parse_semantic_types() {
        let sample = "[euka,helalo]";
        assert_eq!(parse_semantic_types(sample), ["euka", "helalo"]);
    }

    #[test]
    fn test_location() {
        let sample = "ti";
        assert_eq!(
            Location::from_str(sample.to_uppercase().as_str()).unwrap(),
            Location::TI
        );
        let sample = "AB";
        assert_eq!(Location::from_str(sample).unwrap(), Location::AB);
        let sample = "TX";
        assert_eq!(Location::from_str(sample).unwrap(), Location::TX);
        let sample = "TI;AB";
        assert_eq!(Location::from_str(sample).unwrap(), Location::Tiab);
    }
    #[test]
    #[should_panic]
    fn test_invalid_location() {
        let sample = "BG";
        assert_eq!(Location::from_str(sample).unwrap(), Location::Tiab);
    }

    #[test]
    fn test_parse_tree_codes() {
        let sample = "";
        assert_eq!(parse_tree_codes(sample), None);
        let sample = "B01.050.500.131.365.400";
        assert_eq!(
            parse_tree_codes(sample),
            Some(vec![String::from("B01.050.500.131.365.400")])
        );
        let sample = "B01.050.500.131.365.400;B01.050.500.131.365.400";
        assert_eq!(
            parse_tree_codes(sample),
            Some(vec![
                "B01.050.500.131.365.400".to_string(),
                "B01.050.500.131.365.400".to_string()
            ])
        );
    }

    #[test]
    fn test_parse_positional_info() {
        let sample = "228/6;136/7";
        assert_eq!(
            parse_positional_info(sample),
            vec![
                Position::new(228, 6, PositionalInfoType::A),
                Position::new(136, 7, PositionalInfoType::A)
            ]
        );
        let s1 = "[4061/10,4075/11],[4061/10,4075/11]";
        assert_eq!(
            parse_positional_info(s1),
            vec![
                Position::new(4061, 10, PositionalInfoType::D),
                Position::new(4075, 11, PositionalInfoType::D),
                Position::new(4061, 10, PositionalInfoType::D),
                Position::new(4075, 11, PositionalInfoType::D),
            ]
        );
        let s1 = "7059/5,7073/5";
        assert_eq!(
            parse_positional_info(s1),
            vec![
                Position::new(7059, 5, PositionalInfoType::B),
                Position::new(7073, 5, PositionalInfoType::B),
            ]
        );
        let s1 = "[1351/8],[1437/8]";
        assert_eq!(
            parse_positional_info(s1),
            vec![
                Position::new(1351, 8, PositionalInfoType::C),
                Position::new(1437, 8, PositionalInfoType::C),
            ]
        );
    }

    #[test]
    fn test_new_trigger() {
        let t = ("hi", "tI;aB", "124", "fun times", "testing stuff", "1");
        let tt = Trigger::new(t.0, t.1, t.2, t.3, t.4, t.5);
        let actual_tt = Trigger {
            name: String::from("hi"),
            loc: Location::Tiab,
            loc_position: 124,
            text: "fun times".to_string(),
            part_of_speech: "testing stuff".to_string(),
            negation: true,
        };
        assert_eq!(tt, actual_tt);
    }

    #[test]
    fn test_parse_triggers() {
        let sample = "[\"Crustacea\"-ti-1-\"Crustacea\"-noun-0]";
        let result = parse_triggers(sample);
        assert_eq!(
            result,
            [Trigger {
                name: "Crustacea".to_string(),
                loc: Location::TI,
                loc_position: 1,
                text: "Crustacea".to_string(),
                part_of_speech: "noun".to_string(),
                negation: false
            }]
        );
    }

    #[test]
    fn test_new_mmi() {
        let mut map = HashMap::new();
        map.insert("id", "24119710");
        map.insert("mmi", "MMI");
        map.insert("score", "637.30");
        map.insert("name", "Isopoda");
        map.insert("cui", "C0598806");
        map.insert("semantic_types", "[euka]");
        map.insert(
            "triggers",
            "[\"Isopod\"-ab-1-\"isopod\"-adj-0,\"Isopoda\"-ti-1-\"Isopoda\"-noun-0]",
        );
        map.insert("location", "TI;AB");
        map.insert("positional_info", "228/6;136/7");
        map.insert("tree_codes", "B01.050.500.131.365.400");
        let expected = MmiOutput {
            id: "24119710".to_string(),
            mmi: "MMI".to_string(),
            score: 637.30,
            name: "Isopoda".to_string(),
            cui: "C0598806".to_string(),
            semantic_types: vec!["euka".to_string()],
            triggers: vec![
                Trigger {
                    name: "Isopod".to_string(),
                    loc: Location::AB,
                    loc_position: 1,
                    text: "isopod".to_string(),
                    part_of_speech: "adj".to_string(),
                    negation: false,
                },
                Trigger {
                    name: "Isopoda".to_string(),
                    loc: Location::TI,
                    loc_position: 1,
                    text: "Isopoda".to_string(),
                    part_of_speech: "noun".to_string(),
                    negation: false,
                },
            ],
            location: Location::Tiab,
            positional_info: vec![
                Position {
                    start: 228,
                    length: 6,
                    case: PositionalInfoType::A,
                },
                Position {
                    start: 136,
                    length: 7,
                    case: PositionalInfoType::A,
                },
            ],
            tree_codes: Some(vec!["B01.050.500.131.365.400".to_string()]),
        };
        assert_eq!(expected, MmiOutput::new(map));
    }

    #[test]
    fn test_parse_mmi_for_mmi() {
        let s1 = "3124119710|MMI|637.30|Isopoda|C0598806|[euka]|[\"Isopod\"-ab-1-\"isopod\"-adj-0,\"Isopoda\"-ti-1-\"Isopoda\"-noun-0]|TI;AB|228/6;136/7|B01.050.500.131.365.400";
        let expected = MmiOutput {
            id: "3124119710".to_string(),
            mmi: "MMI".to_string(),
            score: 637.3,
            name: "Isopoda".to_string(),
            cui: "C0598806".to_string(),
            semantic_types: vec!["euka".to_string()],
            triggers: vec![
                Trigger {
                    name: "Isopod".to_string(),
                    loc: Location::AB,
                    loc_position: 1,
                    text: "isopod".to_string(),
                    part_of_speech: "adj".to_string(),
                    negation: false,
                },
                Trigger {
                    name: "Isopoda".to_string(),
                    loc: Location::TI,
                    loc_position: 1,
                    text: "Isopoda".to_string(),
                    part_of_speech: "noun".to_string(),
                    negation: false,
                },
            ],
            location: Location::Tiab,
            positional_info: vec![
                Position {
                    start: 228,
                    length: 6,
                    case: PositionalInfoType::A,
                },
                Position {
                    start: 136,
                    length: 7,
                    case: PositionalInfoType::A,
                },
            ],
            tree_codes: Some(vec!["B01.050.500.131.365.400".to_string()]),
        };
        let parsed = match parse_mmi(s1).unwrap() {
            Output::MMI(x) => x,
            _ => panic!("stuff"),
        };
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_parse_mmi_for_aa() {
        let s1 = "23074487|AA|FY|fiscal years|1|2|3|12|9362:2";
        let expected = match parse_mmi(s1).unwrap() {
            Output::AA(x) => x,
            _ => panic!("stuff"),
        };
        println!("{:?}", expected);
    }

    #[test]
    #[should_panic]
    fn test_panic_parse_mmi() {
        let s1 = "asda|fake|other stuff|";
        parse_mmi(s1).unwrap();
    }

    #[test]
    fn test_abbreviation_type() {
        assert_eq!(
            AbbreviationType::AA,
            AbbreviationType::from_str("AA").unwrap()
        );
        assert_eq!(
            AbbreviationType::UA,
            AbbreviationType::from_str("UA").unwrap()
        );
        assert!(AbbreviationType::from_str("asfnkjsanf").is_err())
    }
}
