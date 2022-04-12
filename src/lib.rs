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
enum Location {
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

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
struct Trigger {
    name: String,
    loc: Location,
    loc_position: i32,
    text: String,
    part_of_speech: String,
    negation: bool,
}

fn parse_bool(x: &str) -> bool {
    match x {
        "1" => true,
        "0" => false,
        _ => panic!("Unexpected boolean: {}", x),
    }
}

impl Trigger {
    fn new(
        n: &str,
        loc: &str,
        loc_pos: &str,
        t: &str,
        part_of_speech: &str,
        negation: &str,
    ) -> Trigger {
        Trigger {
            name: n.to_string(),
            loc: Location::from_str(loc).expect("unable to parse Location"),
            loc_position: loc_pos
                .parse::<i32>()
                .expect("unable to parse integer from location"),
            text: t.to_string(),
            part_of_speech: part_of_speech.to_string(),
            negation: parse_bool(negation),
        }
    }
}

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

fn parse_bracketed_info(x: &str) -> Vec<i32> {
    let parts = x
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split('/')
        .map(|x| {
            let y = x.parse::<i32>().expect("could not parse integer");
            y
        })
        .collect::<Vec<i32>>();
    parts
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
enum PositionalInfoType {
    A,
    B,
    C,
    D,
}

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
        // singleton case A
        PositionalInfoType::A
    }
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
struct Position {
    start: i32,
    length: i32,
    case: PositionalInfoType,
}

impl Position {
    fn new(start: i32, length: i32, case: PositionalInfoType) -> Position {
        Position {
            start,
            length,
            case,
        }
    }
}

fn parse_positional_info(info: &str) -> Vec<Position> {
    let tags = tag_pos_info(info);
    let category = categorize_positional_info(tags.0, tags.1, tags.2);
    match category {
        PositionalInfoType::A => info
            .split(';')
            .map(|x| {
                let parts = x
                    .split('/')
                    .map(|x| {
                        let y = x.parse::<i32>().expect(x);
                        y
                    })
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
                            .map(|x| {
                                let y = x.parse::<i32>().expect("could not parse integer");
                                y
                            })
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
                let split_parts = split_with_bracket_context(info);
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

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MmiOutput {
    id: String,
    mmi: String,
    score: String,
    name: String,
    cui: String,
    semantic_types: Vec<String>,
    triggers: Vec<Trigger>,
    location: Location,
    positional_info: Vec<Position>,
    tree_codes: Option<Vec<String>>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_with_bracket_context() {
        let s1 = "[4061/10,4075/11],[4061/10,4075/11]";
        let r1 = split_with_bracket_context(s1);
        assert_eq!(r1, vec!["[4061/10,4075/11]", "[4061/10,4075/11]"])
    }

    // this is a lengthy integration test of the
    // `tag_pos_info` and the `categorize_positional_info` functions
    #[test]
    fn test_pos_info_categorization() {
        // ex 1 type C
        let s1 = "[4061/10,4075/11],[4061/10,4075/11]";
        let r1 = tag_pos_info(s1);
        let cat = categorize_positional_info(r1.0, r1.1, r1.2, r1.3);

        assert_eq!(r1, (false, true, true, true));
        assert_eq!(cat, PositionalInfoType::D);

        let s1 = "117/5;122/4";
        let r1 = tag_pos_info(s1);
        let cat = categorize_positional_info(r1.0, r1.1, r1.2, r1.3);

        assert_eq!(r1, (true, false, false, false));
        assert_eq!(cat, PositionalInfoType::A);

        let s1 = "117/5";
        let r1 = tag_pos_info(s1);
        let cat = categorize_positional_info(r1.0, r1.1, r1.2, r1.3);

        assert_eq!(r1, (false, false, false, false));
        assert_eq!(cat, PositionalInfoType::A);

        let s1 = "117/5,122/4,113/2";
        let r1 = tag_pos_info(s1);
        let cat = categorize_positional_info(r1.0, r1.1, r1.2, r1.3);

        assert_eq!(r1, (false, false, false, true));
        assert_eq!(cat, PositionalInfoType::B);

        let s1 = "[122/4],[117/6]";
        let r1 = tag_pos_info(s1);
        let cat = categorize_positional_info(r1.0, r1.1, r1.2, r1.3);

        assert_eq!(r1, (false, true, false, true));
        assert_eq!(cat, PositionalInfoType::C);
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
        )
    }

    #[test]
    fn test_parse_triggers() {}

    #[test]
    fn test_parse_mmi() {}
}
