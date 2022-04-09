extern crate core;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::str::FromStr;

// let sample_text = "24119710|MMI|637.30|Isopoda|C0598806|[euka]|[\"Isopod\"-ab-1-\"isopod\"-adj-0,\"Isopoda\"-ti-1-\"Isopoda\"-noun-0]|TI;AB|228/6;136/7|B01.050.500.131.365.400";

fn split_text(text: &str) -> Vec<&str> {
    text.split('|').collect()
}

fn label_parts(parts: Vec<&str>) -> HashMap<&str, &str> {
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

fn parse_semantic_types(semantic_types: &str) -> Vec<String> {
    let cleaned = semantic_types
        .strip_prefix('[')
        .unwrap()
        .strip_suffix(']')
        .unwrap();
    cleaned.split(',').map(|x| x.to_string()).collect()
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Location {
    TI,
    AB,
    TX,
    TIAB,
}

impl FromStr for Location {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "TI" => Ok(Location::TI),
            "AB" => Ok(Location::AB),
            "TX" => Ok(Location::TX),
            "TI;AB" => Ok(Location::TIAB),
            _ => Err(()),
        }
    }
}

fn parse_tree_codes(codes: &str) -> Option<Vec<String>> {
    if codes.is_empty() {
        return None;
    }
    Some(codes.split(';').map(|x| x.to_string()).collect())
}

// use with parse triggers
fn split_with_quote_context(x: &str) -> Vec<String> {
    let mut is_in_quotes = false;
    let mut start_position = 0;
    let final_position = x.len();
    let mut parts: Vec<String> = Vec::new();
    for (i, c) in x.chars().enumerate() {
        if c == '\"' {
            is_in_quotes = !is_in_quotes;
        } else if c == ',' && !is_in_quotes {
            parts.push(x[start_position..i].to_string());
            start_position = i + 1;
        } else if i == final_position - 1 {
            // last part
            parts.push(x[start_position..final_position].to_string());
        }
    }
    parts
}

fn parse_triggers(info: &str) -> String {
    // placeholder just returns the string
    info.to_string()
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
struct Position {
    start: u8,
    length: u8,
}

// #[derive(Debug, PartialEq, Eq)]
// pub struct Trigger {
//     name: String,
//     loc: Location,
//     loc_position: u8,
//     text: String,
//     part_of_speech: String,
//     negation: bool,
// }

// fn parse_triggers(triggers: &str) -> Vec<Trigger> {
//     let trigger_list1 = triggers.replace('[', "");
//     let trigger_list2 = trigger_list1.replace(']', "");
//     // removed the quote substitution
//     let trigger_list4 = trigger_list2.split(',');
//     let result = trigger_list4
//         .map(|trigger| {
//             let parts: Vec<&str> = trigger.split('-').collect();
//             println!("{:?}", parts.len());
//             for part in &parts {
//                 println!("{:?}\n", part);
//                 println!("DONE");
//             }
//             let name = parts[0].trim();
//             let loc = parts[1].trim();
//             let loc_pos = parts[2].parse::<u8>().unwrap();
//             let text = parts[3].trim();
//             let pos = parts[4].trim();
//             let negation = parts[5].trim() == "1";
//             Trigger {
//                 name: name.to_string(),
//                 loc: loc.parse().unwrap(),
//                 loc_position: loc_pos,
//                 text: text.to_string(),
//                 part_of_speech: pos.to_string(),
//                 negation,
//             }
//         })
//         .collect();
//     return result;
// }

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MmiOutput {
    pub id: String,
    pub mmi: String,
    pub score: String,
    pub name: String,
    pub cui: String,
    pub semantic_types: Vec<String>,
    pub triggers: String,
    pub location: Location,
    pub positional_info: String,
    pub tree_codes: Option<Vec<String>>,
}

impl MmiOutput {
    pub fn new(parts: HashMap<&str, &str>) -> Self {
        let id = parts["id"].to_string();
        let mmi = parts["mmi"].to_string();
        let score = parts["score"].to_string();
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

pub fn parse_mmi(text: &str) -> MmiOutput {
    let parts = split_text(text);
    let fields = label_parts(parts);
    MmiOutput::new(fields)
}

pub fn parse_mmi_from_json(mut data: Value) -> Value {
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
                            let mmi_output = parse_mmi(prepared);
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


fn parse_positional_info(info: &str) -> String {
    // placeholder just returns the string
    info.to_string()
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
enum PositionalInfoType {
    A,
    B,
    C,
    D,
}

//TODO: more thorough tests
fn tag_pos_info(x: &str) -> (bool, bool, bool, bool) {
    // series of different conditions
    let mut has_semi_colon = false;
    let mut has_brackets = false;
    let mut has_comma = false;
    let mut has_comma_outside_brackets= false;
    let mut in_bracket = false;
    for c in x.chars() {
        // encountered bracket somewhere
        if c == '[' {
            has_brackets = true;
            in_bracket = true;
        } else if c == ']' {
            in_bracket = false;
        } else if c == ';' {
            has_semi_colon = true;
        } else if c == ',' && !in_bracket {
            has_comma_outside_brackets = true;
        } else if c == ',' {
            has_comma = true;
        }
    }
    (has_semi_colon, has_brackets, has_comma, has_comma_outside_brackets)
}

//TODO: more thorough tests
fn categorize_positional_info(has_semi_colon: bool, has_brackets: bool, has_comma: bool, has_comma_outside_brackets: bool) -> PositionalInfoType {
    if has_semi_colon {
        PositionalInfoType::A
    } else if has_comma && !has_brackets {
        PositionalInfoType::B
    } else if has_brackets && has_comma && !has_comma_outside_brackets {
        PositionalInfoType::C
    } else if has_comma_outside_brackets && has_brackets {
        PositionalInfoType::D
    } else {
        panic!("Unable to categorize information based on criteria provided.")
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pos_info_categorization() {
        // ex 1 type C
        let s1 = "[4061/10,4075/11],[4061/10,4075/11]";
        let r1 = tag_pos_info(s1);
        let cat = categorize_positional_info(
            r1.0,
            r1.1,
            r1.2,
            r1.3,
        );

        assert_eq!(r1, (false, true, true, true));
        assert_eq!(cat, PositionalInfoType::D);
    }

    #[test]
    fn test_categorize_pos_info() {
        let sample = "[4061/10,4075/11],[4166/10,4180/11]";
    }

    #[test]
    fn test_quote_splitter() {
        let sample = "[\"Drug, NOS\"-tx-33-\"medicine\"-noun-0,\"Drug, NOS\"-tx-31-\"medicine\"-noun-0,\"Drug, NOS\"-tx-29-\"medication\"-noun-0,\"Drug, NOS\"-tx-5-\"drug\"-noun-0]";
        let r = split_with_quote_context(sample);
        assert_eq!(r.len(), 4);
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
        assert_eq!(label_parts(split_text(sample)), {
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
        assert_eq!(Location::from_str(sample).unwrap(), Location::TIAB);
    }
    #[test]
    #[should_panic]
    fn test_invalid_location() {
        let sample = "BG";
        assert_eq!(Location::from_str(sample).unwrap(), Location::TIAB);
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
        assert_eq!(parse_positional_info(sample), String::from("228/6;136/7"));
    }

    #[test]
    fn test_parse_triggers() {}

    #[test]
    fn test_parse_mmi() {}
}
