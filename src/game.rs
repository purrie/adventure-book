use std::{collections::HashMap, fmt::Display};

use crate::{
    adventure::{Adventure, Choice, Condition, Name, Page, ParsingError, Record},
    evaluation::{EvaluationError, Random},
    file::{read_page, FileError},
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
) -> Result<Page, GameError> {
    let page = match read_page(&adventure.path, page_name) {
        Ok(p) => p,
        Err(e) => return Err(GameError::FileError(e)),
    };
    let story = parse_keywords(&page.story, &adventure.records, &adventure.names)?;
    let choices = parse_choices(
        &page.choices,
        &page.conditions,
        &adventure.records,
        &adventure.names,
        rand,
    )?;

    main_window.game_window.fill_choices(choices);
    main_window.game_window.fill_records(&adventure.records);
    main_window.game_window.display_story(&page.title, story);
    Ok(page)
}
/// Parses supplied text and returns string with tags replaced with their values as found in records and names maps
fn parse_keywords(
    story_text: &String,
    records: &HashMap<String, Record>,
    names: &HashMap<String, Name>,
) -> Result<String, GameError> {
    let reg = Regex::new(r"\[\s*(\w+(?:\s|\w)*)\]").unwrap();

    let mut res = story_text.clone();
    while let Some(caps) = reg.captures(&res) {
        let whole = caps.get(0).unwrap();
        let name = caps.get(1).unwrap();
        if let Some(rec) = records.get(name.as_str()) {
            res.replace_range(whole.range(), &rec.value_as_string());
        } else if let Some(name) = names.get(name.as_str()) {
            res.replace_range(whole.range(), &name.value);
        } else {
            return Err(GameError::ParsingError(ParsingError::MissingRecord(
                name.as_str().to_string(),
            )));
        }
    }
    Ok(res)
}

/// Parses choices for availability and keywords
///
/// The function tests if the choice is available based on its condition.
/// Then it evaluates all keywords found within the choice text
///
/// # Error
///
/// The function will result in error if any condition evaluation results in an error
///
/// The function will result in error if the condition a choice is set to isn't present in the conditions hashmap
///
/// The function will also fail if parsing keywords in choice text fails
fn parse_choices(
    choices: &Vec<Choice>,
    conditions: &HashMap<String, Condition>,
    records: &HashMap<String, Record>,
    names: &HashMap<String, Name>,
    rand: &mut Random,
) -> Result<Vec<(bool, String)>, GameError> {
    let mut res = Vec::new();
    for choice in choices.iter() {
        let enabled;
        if choice.has_condition() {
            if let Some(con) = conditions.get(&choice.condition) {
                match con.evaluate(records, rand) {
                    Ok(v) => enabled = v,
                    Err(e) => return Err(GameError::EvaluationError(e)),
                }
            } else {
                return Err(GameError::ConditionNotFound(choice.condition.clone()));
            }
        } else {
            enabled = true;
        }
        let text = parse_keywords(&choice.text, records, names)?;
        res.push((enabled, text));
    }

    Ok(res)
}

#[derive(Debug)]
pub enum GameError {
    EvaluationError(EvaluationError),
    ParsingError(ParsingError),
    FileError(FileError),
    ConditionNotFound(String),
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
    Editor(crate::editor::Event),
}

impl Display for GameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameError::EvaluationError(e) => write!(f, "{}", e),
            GameError::ParsingError(e) => write!(f, "{}", e),
            GameError::FileError(e) => write!(f, "{}", e),
            GameError::ConditionNotFound(e) => {
                write!(f, "Condition {} have not been found in the page", e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        adventure::{Choice, Condition, Name, Record},
        evaluation::Random,
    };

    use super::{parse_choices, parse_keywords};

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

        let res = parse_keywords(&story, &records, &names).unwrap();
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
        let names = HashMap::new();
        let records = HashMap::new();
        let mut rand = Random::new(69420);

        let res = parse_choices(&choices, &conditions, &records, &names, &mut rand).unwrap();
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
        let names = HashMap::new();

        let res = parse_choices(&choices, &conditions, &records, &names, &mut rand).unwrap();
        for r in res {
            assert_eq!(r.0, lv > rv);
            assert_eq!(r.1, "Choose".to_string());
        }
    }
}
