use std::collections::HashMap;

use regex::Regex;

use crate::evaluation::{evaluate_and_compare, Random};

#[derive(Clone)]
pub struct Adventure {
    pub title: String,
    pub description: String,
    pub path: String,
    pub start: String,
    pub records: HashMap<String, Record>,
    pub names: HashMap<String, Name>,
}
#[derive(Clone)]
pub struct Record {
    pub category: String,
    pub name: String,
    pub value: i32,
}
#[derive(Clone)]
pub struct Name {
    pub keyword: String,
    pub value: String,
}
pub struct Page {
    pub title: String,
    pub story: String,
    pub choices: Vec<Choice>,
    pub conditions: HashMap<String, Condition>,
    pub tests: HashMap<String, Test>,
    pub results: HashMap<String, StoryResult>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Comparison {
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Equal,
    NotEqual,
}
pub struct StoryResult {
    pub name: String,
    pub expression: String,
}
pub struct Test {
    pub name: String,
    pub expression_r: String,
    pub comparison: Comparison,
    pub expression_l: String,
    pub success_result: String,
    pub failure_result: String,
}
pub struct Choice {
    pub text: String,
    pub condition: String,
    pub test: String,
    pub result: String,
}
pub struct Condition {
    pub name: String,
    pub expression_r: String,
    pub comparison: Comparison,
    pub expression_l: String,
}
// those are for matching tags in choice so we can figure out which choices should be connected to other elements.
const REGEX_CONDITION_IN_CHOICE: &str = r"\{\s*condition:\s*(\w+(?:\s|\w)*)\s*\}";
const REGEX_TEST_IN_CHOICE: &str = r"\{\s*test:\s*(\w+(?:\s|\w)*)\s*\}";
const REGEX_RESULT_IN_CHOICE: &str = r"\{\s*result:\s*(\w+(?:\s|\w)*)\s*\}";

impl Adventure {
    /// Creates a new empty adventure data
    pub fn new() -> Adventure {
        Adventure {
            title: String::new(),
            description: String::new(),
            path: String::new(),
            start: String::new(),
            records: HashMap::<String, Record>::new(),
            names: HashMap::<String, Name>::new(),
        }
    }
    /// Creates an adventure from text string
    ///
    /// The function goes through text line by line and looks for keywords at start, if it finds a keyword, it will process it.
    /// In case where line doesn't start in a keyword, the line will be added to adventure description if that's the last keyword that was read.
    ///
    /// Note that path can be relative or absolute
    pub fn parse_from_string(text: String, path: String) -> Result<Adventure, ()> {
        let mut adv = Adventure::new();

        let lines = text.lines();
        let mut flag = 0;
        for line in lines {
            if line.starts_with("title:") {
                flag = 0;
                adv.title = line.replacen("title:", "", 1).trim().to_string();
            } else if line.starts_with("description:") {
                flag = 1;
                adv.description = line.replacen("description:", "", 1).trim().to_string();
            } else if line.starts_with("start:") {
                flag = 0;
                adv.start = line.replacen("start:", "", 1).trim().to_string();
            } else if line.starts_with("record:") {
                flag = 0;
                let text = line.replacen("record:", "", 1);
                let rec = Record::parse_from_string(text);

                if let Ok(r) = rec {
                    adv.records.insert(r.name.clone(), r);
                } else {
                    return Err(());
                }
            } else if line.starts_with("name:") {
                flag = 0;
                let text = line.replacen("record:", "", 1);
                if let Ok(name) = Name::parse_from_string(text) {
                    adv.names.insert(name.keyword.clone(), name);
                } else {
                    return Err(());
                }
            } else {
                if flag == 1 {
                    adv.description = adv.description + line;
                }
            }
        }
        adv.path = path;

        if adv.is_valid() {
            return Ok(adv);
        } else {
            return Err(());
        }
    }
    pub fn is_valid(&self) -> bool {
        if self.title.len() < 1 {
            return false;
        }
        if self.description.len() < 1 {
            return false;
        }
        if self.path.len() < 1 {
            return false;
        }
        if self.start.len() < 1 {
            return false;
        }

        true
    }
}
impl Page {
    /// Creates an empty page
    pub fn new() -> Page {
        Page {
            title: String::new(),
            story: String::new(),
            choices: Vec::<Choice>::new(),
            conditions: HashMap::<String, Condition>::new(),
            tests: HashMap::<String, Test>::new(),
            results: HashMap::<String, StoryResult>::new(),
        }
    }

    /// Parses string into Page. It will return error if the text isn't valid page
    pub fn parse_from_string(text: String) -> Result<Page, ()> {
        // creating empty page to populate
        let mut page = Page::new();

        // next we break the text into lines and create regex lookups to match and connect parts of the page
        let lines = text.lines();

        let match_condition = Regex::new(REGEX_CONDITION_IN_CHOICE).unwrap();
        let match_test = Regex::new(REGEX_TEST_IN_CHOICE).unwrap();
        let match_result = Regex::new(REGEX_RESULT_IN_CHOICE).unwrap();

        let mut story_line = false;
        for line in lines {
            // the flag marks if we're at a story line, this allows story lines to be broken up into multiple lines later on

            if line.starts_with("title:") {
                // matching title by keyword
                story_line = false;
                page.title = line.replacen("title:", "", 1).trim().to_string();
            } else if line.starts_with("story:") {
                // same with the story, we set the flag to 1 here to signify that any following line that doesn't match any keyword can be added to story
                story_line = true;
                page.story = line.replacen("story:", "", 1).trim().to_string();
            } else if line.starts_with("choice:") {
                story_line = false;
                // Reading choice from the line
                let cho = Choice::parse_from_string(
                    line.replacen("choice:", "", 1),
                    &match_condition,
                    &match_test,
                    &match_result,
                );

                // if read succeeds, we push the choice in
                if let Ok(c) = cho {
                    page.choices.push(c);
                } else {
                    return Err(());
                }
            } else if line.starts_with("condition:") {
                story_line = false;

                let con = Condition::parse_from_string(line.replacen("condition:", "", 1));

                // reading condition data can fail, in that case we fail the page
                if let Ok(c) = con {
                    page.conditions.insert(c.name.clone(), c);
                } else {
                    return Err(());
                }
            } else if line.starts_with("test:") {
                story_line = false;

                // we fail the page if the test doesn't load in correctly
                let test = Test::parse_from_string(line.replacen("test:", "", 1));
                if let Ok(t) = test {
                    page.tests.insert(t.name.clone(), t);
                } else {
                    return Err(());
                }
            } else if line.starts_with("result:") {
                story_line = false;

                // failing the page if result doesn't load correctly, like in other cases
                let res = StoryResult::parse_from_string(line.replacen("result:", "", 1));
                if let Ok(r) = res {
                    page.results.insert(r.name.clone(), r);
                } else {
                    return Err(());
                }
            } else if story_line {
                // adding a line to story if it's immediately after story keyword and doesn't match any other keywords
                page.story += line;
            }
        }
        if page.is_valid() {
            Ok(page)
        } else {
            Err(())
        }
    }
    pub fn is_valid(&self) -> bool {
        if self.story.len() < 1 {
            return false;
        }
        if self.choices.len() < 1 {
            return false;
        }
        if self.results.len() < 1 {
            return false;
        }
        true
    }
}

/// macro that extracts keywords from choice text
macro_rules! insert_in_choice {
    ($reg:ident, $target:expr, $source:ident) => {
        // we start by capturing the keyword through regex
        if let Some(c) = $reg.captures(&$source) {
            // we have two matches here, first is the whole match and second is just the name of matched keyword
            let whole = c.get(0).unwrap();
            let name = c.get(1).unwrap();
            // we stuff the name into provided target
            $target = name.as_str().trim().to_string();
            // and then remove the whole keyword from source text
            $source.replace_range(whole.range(), "");
        }
    };
}

impl Choice {
    pub fn new() -> Choice {
        Choice {
            text: String::new(),
            condition: String::new(),
            test: String::new(),
            result: String::new(),
        }
    }
    /// Parses string into Choice.
    ///
    /// It requires to be supplied with preconfigured Regex matches which capture name of the matched result
    pub fn parse_from_string(
        mut text: String,
        match_condition: &Regex,
        match_test: &Regex,
        match_result: &Regex,
    ) -> Result<Choice, ()> {
        let mut choice = Choice::new();
        // we use macros here to extract appropriate keywords into their places.
        insert_in_choice!(match_condition, choice.condition, text);
        insert_in_choice!(match_test, choice.test, text);
        insert_in_choice!(match_result, choice.result, text);

        // we finish up by assigning text with keywords extracted and push it into the page
        choice.text = text.trim().to_string();
        if choice.is_valid() {
            Ok(choice)
        } else {
            Err(())
        }
    }
    /// Tests if this choice is valid
    ///
    /// It will return flase if it doesn't have test or result name
    /// or if it has both
    pub fn is_valid(&self) -> bool {
        if self.text.len() < 1 {
            return false;
        }
        if self.test.len() == 0 && self.result.len() == 0 {
            return false;
        }
        if self.test.len() > 0 && self.result.len() > 0 {
            return false;
        }
        true
    }
    /// Tests if this choice always leads to the same result or not
    ///
    /// Will return false if it leads to a test instead
    pub fn is_constant(&self) -> bool {
        self.result.len() > 0
    }
    pub fn has_condition(&self) -> bool {
        self.condition.len() > 0
    }
}
impl From<&str> for Comparison {
    /// Less than or equal is default for anything that doesn't match expected comparisons. Not sure if I should leave it like this or error
    fn from(item: &str) -> Self {
        match item.trim() {
            ">" => Comparison::Greater,
            ">=" => Comparison::GreaterEqual,
            "=" => Comparison::Equal,
            "!" => Comparison::NotEqual,
            "<" => Comparison::Less,
            _ => Comparison::LessEqual,
        }
    }
}
impl Comparison {
    pub fn compare(&self, lhv: i32, rhv: i32) -> bool {
        match self {
            Comparison::Greater => lhv > rhv,
            Comparison::GreaterEqual => lhv >= rhv,
            Comparison::Less => lhv < rhv,
            Comparison::LessEqual => lhv <= rhv,
            Comparison::Equal => lhv == rhv,
            Comparison::NotEqual => lhv != rhv,
        }
    }
}
impl Condition {
    pub fn parse_from_string(text: String) -> Result<Condition, ()> {
        // splitting the text into parts. Expected order of data is name, exp right, comparison, exp left. We filter out empty strings
        let args: Vec<&str> = text
            .split(";")
            .map(|x| x.trim())
            .filter(|x| x.len() > 0)
            .collect();

        // function will report error if incorrect amount of data was found.
        if args.len() != 4 {
            return Err(());
        }

        // constructing the condition.
        Ok(Condition {
            name: args[0].to_string(),
            expression_l: args[1].to_string(),
            comparison: Comparison::from(args[2]),
            expression_r: args[3].to_string(),
        })
    }
    pub fn evaluate(&self, records: &HashMap<String, Record>, rand: &mut Random) -> bool {
        if let Ok(ok) = evaluate_and_compare(
            &self.expression_l,
            &self.expression_r,
            &self.comparison,
            records,
            rand,
        ) {
            ok
        } else {
            // TODO probably some error handling should be in order here
            false
        }
    }
}
impl Test {
    pub fn parse_from_string(text: String) -> Result<Test, ()> {
        let args: Vec<&str> = text
            .split(";")
            .map(|x| x.trim())
            .filter(|x| x.len() > 0)
            .collect();

        if args.len() != 6 {
            return Err(());
        }

        Ok(Test {
            name: args[0].to_string(),
            expression_l: args[1].to_string(),
            comparison: Comparison::from(args[2]),
            expression_r: args[3].to_string(),
            success_result: args[4].to_string(),
            failure_result: args[5].to_string(),
        })
    }
    pub fn evaluate(&self, records: &HashMap<String, Record>, rand: &mut Random) -> &String {
        if let Ok(ok) = evaluate_and_compare(
            &self.expression_l,
            &self.expression_r,
            &self.comparison,
            records,
            rand,
        ) {
            if ok {
                &self.success_result
            } else {
                &self.failure_result
            }
        } else {
            // TODO do proper error handling
            panic!("Invalid evaluation of test {}", self.name);
        }
    }
}
impl StoryResult {
    pub fn parse_from_string(text: String) -> Result<StoryResult, ()> {
        let args: Vec<&str> = text
            .splitn(2, ";")
            .map(|x| x.trim())
            .filter(|x| x.len() > 0)
            .collect();

        if args.len() != 2 {
            return Err(());
        }

        Ok(StoryResult {
            name: args[0].to_string(),
            expression: args[1].to_string(),
        })
    }
}
impl Record {
    /// Creates a record from a text data.
    pub fn parse_from_string(text: String) -> Result<Record, ()> {
        let args: Vec<&str> = text
            .split(";")
            .map(|x| x.trim())
            .filter(|x| x.len() > 0)
            .collect();

        let len = args.len();
        let name;
        let category;
        let value;
        match len {
            1 => {
                name = args[0].to_string();
                category = String::new();
                value = 0;
            },
            2 => {
                name = args[0].to_string();
                if let Ok(n) = args[1].parse() {
                    value = n;
                    category = String::new();
                } else {
                    value = 0;
                    category = args[1].to_string();
                }
            },
            3 => {
                name = args[0].to_string();
                category = args[1].to_string();
                if let Ok(n) = args[2].parse() {
                    value = n;
                } else {
                    return Err(());
                }
            },
            _ => return Err(()),
        }
        Ok(Record {
            name,
            category,
            value,
        })
    }
    pub fn value_as_string(&self) -> String {
        (self.value as i32).to_string()
    }
}
impl Name {
    pub fn parse_from_string(text: String) -> Result<Name, ()> {
        let args: Vec<&str> = text
            .split(";")
            .map(|x| x.trim())
            .filter(|x| x.len() > 0)
            .collect();

        let len = args.len();
        if len == 0 || len > 2 {
            return Err(());
        }

        Ok(Name {
            keyword: args[0].to_string(),
            value: match len == 2 {
                true => args[1].to_string(),
                false => String::new(),
            },
        })
    }
}

#[cfg(test)]
mod tests {

    use regex::Regex;

    use crate::adventure::Comparison;

    use super::{Adventure, Choice, Condition, Page, Record, StoryResult, Test};

    #[test]
    fn record_parse() {
        let data = "strength; attributes;".to_string();
        let rec = Record::parse_from_string(data).unwrap();
        assert_eq!(rec.name, "strength");
        assert_eq!(rec.category, "attributes");
    }
    #[test]
    fn result_parse() {
        let data = "proceed; next scene".to_string();
        let res = StoryResult::parse_from_string(data).unwrap();
        assert_eq!(res.name, "proceed");
        assert_eq!(res.expression, "next scene");
    }
    #[test]
    fn result_record_mod_parse() {
        let data = "proceed; strength; 1; next_scene;".to_string();
        let res = StoryResult::parse_from_string(data).unwrap();
        assert_eq!(res.name, "proceed");
        assert_eq!(res.expression, "strength; 1; next_scene;");
    }
    #[test]
    fn test_parse() {
        let data = "bravery; 1d20; <=; [confidence]; proceed; cowardness;".to_string();
        let t = Test::parse_from_string(data).unwrap();
        assert_eq!(t.name, "bravery");
        assert_eq!(t.expression_l, "1d20");
        assert_eq!(t.expression_r, "[confidence]");
        assert_eq!(t.comparison, Comparison::LessEqual);
        assert_eq!(t.success_result, "proceed");
        assert_eq!(t.failure_result, "cowardness");
    }
    #[test]
    fn condition_parse() {
        let data = "wealth; [wealth]; >=; 1d100+15;".to_string();
        let con = Condition::parse_from_string(data).unwrap();
        assert_eq!(con.name, "wealth");
        assert_eq!(con.comparison, Comparison::GreaterEqual);
        assert_eq!(con.expression_l, "[wealth]");
        assert_eq!(con.expression_r, "1d100+15");
    }
    #[test]
    fn comparison_conversion() {
        let mut comp: Comparison = ">".into();
        assert_eq!(comp, Comparison::Greater);
        comp = ">=".into();
        assert_eq!(comp, Comparison::GreaterEqual);
        comp = "=".into();
        assert_eq!(comp, Comparison::Equal);
        comp = "!".into();
        assert_eq!(comp, Comparison::NotEqual);
        comp = "<".into();
        assert_eq!(comp, Comparison::Less);
        comp = "<=".into();
        assert_eq!(comp, Comparison::LessEqual);
    }
    // TODO write test that would make parsing double links in choices invalid, like having two results for example
    #[test]
    fn choice_parse_condition_result() {
        let data = "Do something brave! {condition: brave} {result: proceed}".to_string();
        let match_condition = Regex::new(super::REGEX_CONDITION_IN_CHOICE).unwrap();
        let match_test = Regex::new(super::REGEX_TEST_IN_CHOICE).unwrap();
        let match_result = Regex::new(super::REGEX_RESULT_IN_CHOICE).unwrap();
        let cho =
            Choice::parse_from_string(data, &match_condition, &match_test, &match_result).unwrap();
        assert_eq!(cho.text, "Do something brave!");
        assert_eq!(cho.test, "");
        assert_eq!(cho.condition, "brave");
        assert_eq!(cho.result, "proceed");
    }
    #[test]
    fn choice_parse_test() {
        let data = "Do something brave! { test: bravery }".to_string();
        let match_condition = Regex::new(super::REGEX_CONDITION_IN_CHOICE).unwrap();
        let match_test = Regex::new(super::REGEX_TEST_IN_CHOICE).unwrap();
        let match_result = Regex::new(super::REGEX_RESULT_IN_CHOICE).unwrap();
        let cho =
            Choice::parse_from_string(data, &match_condition, &match_test, &match_result).unwrap();
        assert_eq!(cho.text, "Do something brave!");
        assert_eq!(cho.test, "bravery");
        assert_eq!(cho.condition, "");
        assert_eq!(cho.result, "");
    }
    #[test]
    fn choice_parse() {
        let data = "Do something brave! { result: proceed }".to_string();
        let match_condition = Regex::new(super::REGEX_CONDITION_IN_CHOICE).unwrap();
        let match_test = Regex::new(super::REGEX_TEST_IN_CHOICE).unwrap();
        let match_result = Regex::new(super::REGEX_RESULT_IN_CHOICE).unwrap();
        let cho =
            Choice::parse_from_string(data, &match_condition, &match_test, &match_result).unwrap();
        assert_eq!(cho.text, "Do something brave!");
        assert_eq!(cho.test, "");
        assert_eq!(cho.condition, "");
        assert_eq!(cho.result, "proceed");
    }
    #[test]
    fn choice_valid() {
        let mut cho = Choice {
            text: String::from("Do something brave!"),
            condition: String::new(),
            result: String::from("Proceed"),
            test: String::new(),
        };
        assert!(cho.is_valid());
        cho.result = String::new();
        cho.test = String::from("bravery");
        assert!(cho.is_valid());
    }
    #[test]
    fn choice_invalid() {
        let mut cho = Choice {
            text: String::from("Do something brave!"),
            condition: String::new(),
            result: String::new(),
            test: String::new(),
        };
        assert!(!cho.is_valid());
        cho.result = String::from("proceed");
        cho.test = String::from("bravery");
        assert!(!cho.is_valid());
    }
    #[test]
    fn page_parse() {
        let data = "title: At the Castle Ruins
story: [name] arrived at the ruined castle where the fabled dragon has kidnapped the princess to. The air is stale, filled with stench of sulfour and roars of the beast can be heard in the distance.
choice: Proceed through the gate! {test: bravery}
choice: Run away! {result: coward}
choice: Prepare stuffed animal spiked with poison. {condition: animal}{result: victory}
condition: animal; [stuffed animals]; >; 1;
test: bravery; [confidence]; >=; 1d20; victory; coward;
result: victory; victory_scene
result: coward; confidence; -1; coward_scene;".to_string();
        let page = Page::parse_from_string(data).unwrap();

        assert_eq!(page.title, "At the Castle Ruins");
        assert_eq!(page.story, "[name] arrived at the ruined castle where the fabled dragon has kidnapped the princess to. The air is stale, filled with stench of sulfour and roars of the beast can be heard in the distance.");
        assert_eq!(page.choices.len(), 3);
        assert_eq!(page.conditions.len(), 1);
        assert_eq!(page.tests.len(), 1);
        assert_eq!(page.results.len(), 2);

        for choice in page.choices {
            assert!(choice.is_valid());
        }
    }
    #[test]
    fn adventure_parse() {
        let data = "title: Damsel in Distress
description: This is a story about a knight who faces a dragon to save the princess
start: at_the_castle_ruins
record: confidence; attributes;
record: stuffed animals; resources;"
            .to_string();
        let adventure = Adventure::parse_from_string(data, "damsel".to_string()).unwrap();

        assert_eq!(adventure.title, "Damsel in Distress");
        assert_eq!(
            adventure.description,
            "This is a story about a knight who faces a dragon to save the princess"
        );
        assert_eq!(adventure.start, "at_the_castle_ruins");
        assert_eq!(adventure.records.len(), 2);

        let con = adventure.records.get("confidence").unwrap();
        let stuff = adventure.records.get("stuffed animals").unwrap();
        assert_eq!(con.name, "confidence");
        assert_eq!(con.category, "attributes");
        assert_eq!(stuff.name, "stuffed animals");
        assert_eq!(stuff.category, "resources");
    }
    #[test]
    fn comparison_greater() {
        assert!(Comparison::Greater.compare(20, 10));
    }
    #[test]
    fn comparison_greater_equal() {
        assert!(Comparison::GreaterEqual.compare(10, 10));
    }
    #[test]
    fn comparison_less() {
        assert!(Comparison::Less.compare(10, 20));
    }
    #[test]
    fn comparison_less_equal() {
        assert!(Comparison::LessEqual.compare(10, 10));
    }
    #[test]
    fn comparison_equal() {
        assert!(Comparison::Equal.compare(10, 10));
    }
    #[test]
    fn comparison_not_equal() {
        assert!(Comparison::NotEqual.compare(10, 20));
    }
}
