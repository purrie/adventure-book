use std::collections::HashMap;

use crate::{
    adventure::{Adventure, Choice, Condition, Name, Page, Record},
    evaluation::Random,
    file::read_page,
    window::MainWindow,
};
use regex::Regex;

/// Changes currently displayed page.
///
/// It refreshes windows contents to update changes in records and fills story and choices
pub fn render_page(
    main_window: &mut MainWindow,
    adventure: &Adventure,
    page_name: &String,
    rand: &mut Random,
) -> Result<Page, String> {
    let page;
    match read_page(&adventure.path, page_name) {
        Ok(p) => page = p,
        Err(e) => {
            // TODO do proper error handling
            panic!(
                "Page {} of {} failed to load due to: {}",
                page_name, adventure.title, e
            );
        }
    }
    let story = parse_story_text(&page.story, &adventure.records, &adventure.names);
    let choices;
    match parse_choices(&page.choices, &page.conditions, &adventure.records, rand) {
        Ok(v) => choices = v,
        Err(e) => return Err(e),
    }

    main_window.game_window.fill_choices(choices);
    main_window.game_window.fill_records(&adventure.records);
    main_window.game_window.display_story(story);
    Ok(page)
}
/// Parses supplied text and returns string with tags replaced with their values as found in records and names maps
fn parse_story_text(
    story_text: &String,
    records: &HashMap<String, Record>,
    names: &HashMap<String, Name>,
) -> String {
    let reg = Regex::new(r"\[\s*(\w+(?:\s|\w)*)\]").unwrap();

    let mut res = story_text.clone();
    while let Some(caps) = reg.captures(&res) {
        let whole = caps.get(0).unwrap();
        let name = caps.get(1).unwrap();
        if records.contains_key(name.as_str()) {
            res.replace_range(
                whole.range(),
                &records.get(name.as_str()).unwrap().value_as_string(),
            );
        } else if names.contains_key(name.as_str()) {
            res.replace_range(whole.range(), &names.get(name.as_str()).unwrap().value);
        } else {
            panic!("Adventure lacks '{}' record or name", name.as_str());
        }
    }
    res
}
/// Goes over all choices, evaluating their conditions and returns a list of bool and string tuples, representing whatever the choice meets conditions and the text of the choice
fn parse_choices(
    choices: &Vec<Choice>,
    conditions: &HashMap<String, Condition>,
    records: &HashMap<String, Record>,
    rand: &mut Random,
) -> Result<Vec<(bool, String)>, String> {
    let mut res = Vec::new();
    for choice in choices.iter() {
        let enabled;
        if choice.has_condition() {
            if let Some(con) = conditions.get(&choice.condition) {
                match con.evaluate(records, rand) {
                    Ok(v) => enabled = v,
                    Err(e) => return Err(e),
                }
            } else {
                // TODO probably error handlign in case condition doesn't exist
                enabled = false;
            }
        } else {
            enabled = true;
        }
        res.push((enabled, choice.text.clone()));
    }

    Ok(res)
}

#[derive(Clone)]
pub enum Event {
    DisplayMainMenu,
    DisplayAdventureSelect,
    StartAdventure,
    QuitToMainMenu,
    Quit,
    SelectAdventure(String),
    StoryChoice(usize),
    EditAdventure,

    EditorSave,

    EditorAddPage,
    EditorRemovePage,
    EditorOpenMeta,
    EditorOpenPage(String),
    EditorAddRecord,
    EditorAddName,
    EditorInsertRecord(String),
    EditorInsertName(String),
    EditorEditRecord(usize),
    EditorEditName(usize),
    EditorRemoveRecord(usize),
    EditorRemoveName(usize),

    /// This event is used to select data block for sub editors in pages
    EditorSelectInSubEditor(i32),
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        adventure::{Choice, Condition, Name, Record},
        evaluation::Random,
    };

    use super::{parse_choices, parse_story_text};

    #[test]
    fn story_text_parsing() {
        let story = "You approach castle [castle name] in hopes of finding adventure. With only [gold] gold coins in your pouch, you know you will have to accept any job you will be offered. However, you're sure you will find something good. The name of [name] should be well known in these parts.".to_string();
        let expected = "You approach castle Stonehill in hopes of finding adventure. With only 13 gold coins in your pouch, you know you will have to accept any job you will be offered. However, you're sure you will find something good. The name of Joseph the Adventurer should be well known in these parts.".to_string();

        let mut names = HashMap::new();
        let mut records = HashMap::new();

        names.insert(
            "castle name".to_string(),
            Name {
                keyword: "castle name".to_string(),
                value: "Stonehill".to_string(),
            },
        );
        names.insert(
            "name".to_string(),
            Name {
                keyword: "name".to_string(),
                value: "Joseph the Adventurer".to_string(),
            },
        );
        records.insert(
            "gold".to_string(),
            Record {
                category: String::new(),
                name: "gold".to_string(),
                value: 13,
            },
        );

        let res = parse_story_text(&story, &records, &names);
        assert_eq!(res, expected);
    }
    #[test]
    fn parsing_choices() {
        let choices = vec![Choice {
            text: "Choose".to_string(),
            condition: "con".to_string(),
            result: "res".to_string(),
            test: String::new(),
        }];
        let mut conditions = HashMap::new();
        conditions.insert(
            "con".to_string(),
            Condition {
                comparison: crate::adventure::Comparison::Equal,
                expression_l: "1".to_string(),
                expression_r: "1".to_string(),
                name: "con".to_string(),
            },
        );
        let records = HashMap::new();
        let mut rand = Random::new(69420);

        let res = parse_choices(&choices, &conditions, &records, &mut rand).unwrap();
        for r in res {
            assert!(r.0);
            assert_eq!(r.1, "Choose".to_string());
        }
    }
    #[test]
    fn parsing_choices_expression() {
        let choices = vec![Choice {
            text: "Choose".to_string(),
            condition: "con".to_string(),
            result: "res".to_string(),
            test: String::new(),
        }];
        let mut conditions = HashMap::new();

        let mut rand = Random::new(69420);
        let lv = rand.die(1, 20);
        let rv = rand.die(1, 4);
        rand = Random::new(69420);
        conditions.insert(
            "con".to_string(),
            Condition {
                comparison: crate::adventure::Comparison::Greater,
                expression_l: "1d20".to_string(),
                expression_r: "1d4".to_string(),
                name: "con".to_string(),
            },
        );
        let records = HashMap::new();

        let res = parse_choices(&choices, &conditions, &records, &mut rand).unwrap();
        for r in res {
            assert_eq!(r.0, lv > rv);
            assert_eq!(r.1, "Choose".to_string());
        }
    }
}
