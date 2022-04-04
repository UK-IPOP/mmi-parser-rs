use std::collections::HashMap;
use std::str::FromStr;

// let sample_text = "24119710|MMI|637.30|Isopoda|C0598806|[euka]|[\"Isopod\"-ab-1-\"isopod\"-adj-0,\"Isopoda\"-ti-1-\"Isopoda\"-noun-0]|TI;AB|228/6;136/7|B01.050.500.131.365.400";

fn split_text(text: &str) -> Vec<&str> {
    text.split("|").collect()
}

fn name_parts(parts: Vec<&str>) -> HashMap<&str, &str> {
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
        .strip_prefix("[")
        .unwrap()
        .strip_suffix("]")
        .unwrap();
    cleaned.split(",").map(|x| x.to_string()).collect()
}

#[derive(PartialEq, Eq, Debug)]
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
    if codes == "" {
        return None;
    }
    Some(codes.split(";").map(|x| x.to_string()).collect())
}

#[derive(PartialEq, Eq, Debug)]
struct Position {
    start: u8,
    length: u8,
}

fn parse_positional_info(info: &str) -> String {
    // placeholder just returns the string
    info.to_string()
}

#[derive(Debug, PartialEq, Eq)]
pub struct Trigger {
    name: String,
    loc: Location,
    loc_position: u8,
    text: String,
    part_of_speech: String,
    negation: bool,
}

fn parse_triggers(triggers: &str) -> Vec<Trigger> {
    let trigger_list1 = triggers.replace("[", "");
    let trigger_list2 = trigger_list1.replace("]", "");
    let trigger_list3 = trigger_list2.replace("\"", "");
    let trigger_list4 = trigger_list3.split(",").into_iter();
    let result = trigger_list4
        .map(|trigger| {
            let parts = trigger.split("-").collect::<Vec<&str>>();
            let name = parts[0];
            let loc = parts[1];
            let loc_pos = parts[2].parse::<u8>().unwrap();
            let text = parts[3];
            let pos = parts[4];
            let negation = parts[5] == "1";
            Trigger {
                name: name.to_string(),
                loc: loc.parse().unwrap(),
                loc_position: loc_pos,
                text: text.to_string(),
                part_of_speech: pos.to_string(),
                negation,
            }
        })
        .collect();
    return result;
}

#[derive(Debug, PartialEq, Eq)]

pub struct MmiOutput {
    pub id: String,
    pub mmi: String,
    pub score: String,
    pub name: String,
    pub cui: String,
    pub semantic_types: Vec<String>,
    pub triggers: Vec<Trigger>,
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
        let positional_info = parts["positional_info"].to_string();
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
    let fields = name_parts(parts);
    MmiOutput::new(fields)
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(name_parts(split_text(sample)), {
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
    fn test_parse_triggers() {
        let sample = "[\"Isopod\"-ab-1-\"isopod\"-adj-0,\"Isopoda\"-ti-1-\"Isopoda\"-noun-0]";
        let result = parse_triggers(sample);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "Isopod".to_string());
        assert_eq!(result[0].loc, Location::AB);
        assert_eq!(result[0].loc_position, 1);
        assert_eq!(result[0].text, "isopod".to_string());
        assert_eq!(result[0].part_of_speech, "adj".to_string());
        assert_eq!(result[0].negation, false);
        assert_eq!(
            result[1],
            Trigger {
                name: "Isopoda".to_string(),
                loc: Location::TI,
                loc_position: 1,
                text: "Isopoda".to_string(),
                part_of_speech: "noun".to_string(),
                negation: false,
            },
        );
    }

    #[test]
    fn test_parse_mmi() {
        ()
    }
}
