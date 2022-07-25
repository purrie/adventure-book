use regex::Regex;

pub struct Adventure {
    pub title: String,
    pub description: String,
    pub path: String,
    pub start: String,
    pub records: Vec<Record>,
    pub names: Vec<Name>,
}
pub struct Record {
    pub category: String,
    pub name: String,
    pub value: f64,
}
pub struct Name {
    pub keyword: String,
    pub name: String,
}
pub struct Page {
    pub title: String,
    pub story: String,
    pub choices: Vec<Choice>,
    pub conditions: Vec<Condition>,
    pub tests: Vec<Test>,
    pub results: Vec<StoryResult>,
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
            records: Vec::<Record>::new(),
            names: Vec::<Name>::new(),
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
                    adv.records.push(r);
                } else {
                    return Err(());
                }
            } else if line.starts_with("name:") {
                flag = 0;
                let text = line.replacen("record:", "", 1);
                if let Ok(name) = Name::parse_from_string(text) {
                    adv.names.push(name);
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
            conditions: Vec::<Condition>::new(),
            tests: Vec::<Test>::new(),
            results: Vec::<StoryResult>::new(),
        }
    }

    /// Parses string into Page. It will return error if the text isn't valid page
    pub fn parse_from_string(text: String) -> Result<Page, ()> {
        // creating empty page to populate
        let mut page = Page::new();
        let flag = 0;

        // next we break the text into lines and create regex lookups to match and connect parts of the page
        let lines = text.lines();

        let match_condition = Regex::new(REGEX_CONDITION_IN_CHOICE).unwrap();
        let match_test = Regex::new(REGEX_TEST_IN_CHOICE).unwrap();
        let match_result = Regex::new(REGEX_RESULT_IN_CHOICE).unwrap();

        for line in lines {
            // the flag marks if we're at a story line, this allows story lines to be broken up into multiple lines later on
            let mut flag = 0;

            if line.starts_with("title:") {
                // matching title by keyword
                flag = 0;
                page.title = line.replacen("title:", "", 1).trim().to_string();
            } else if line.starts_with("story:") {
                // same with the story, we set the flag to 1 here to signify that any following line that doesn't match any keyword can be added to story
                flag = 1;
                page.story = line.replacen("story:", "", 1).trim().to_string();
            } else if line.starts_with("choice:") {
                flag = 0;
                // first we get the actual story text
                let cho = Choice::parse_from_string(
                    line.replacen("choice:", "", 1),
                    &match_condition,
                    &match_test,
                    &match_result,
                );

                if let Ok(c) = cho {
                    page.choices.push(c);
                } else {
                    return Err(());
                }
            } else if line.starts_with("condition:") {
                flag = 0;

                let con = Condition::parse_from_string(line.replacen("condition:", "", 1));

                // reading condition data can fail, in that case we fail the page
                if let Ok(c) = con {
                    page.conditions.push(c);
                } else {
                    return Err(());
                }
            } else if line.starts_with("test:") {
                flag = 0;

                // we fail the page if the test doesn't load in correctly
                let test = Test::parse_from_string(line.replacen("test:", "", 1));
                if let Ok(t) = test {
                    page.tests.push(t);
                } else {
                    return Err(());
                }
            } else if line.starts_with("result:") {
                flag = 0;

                // failing the page if result doesn't load correctly, like in other cases
                let res = StoryResult::parse_from_string(line.replacen("result:", "", 1));
                if let Ok(r) = res {
                    page.results.push(r);
                } else {
                    return Err(());
                }
            } else if flag == 1 {
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
impl Condition {
    pub fn new() -> Condition {
        Condition {
            name: String::new(),
            expression_r: String::new(),
            comparison: Comparison::Less,
            expression_l: String::new(),
        }
    }
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
}
impl Test {
    pub fn new() -> Test {
        Test {
            name: String::new(),
            expression_r: String::new(),
            comparison: Comparison::Less,
            expression_l: String::new(),
            success_result: String::new(),
            failure_result: String::new(),
        }
    }
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
}
impl StoryResult {
    pub fn new() -> StoryResult {
        StoryResult {
            name: String::new(),
            expression: String::new(),
        }
    }
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
    pub fn new() -> Record {
        Record {
            category: String::new(),
            name: String::new(),
            value: 0.0,
        }
    }
    /// Creates a record from a text data.
    pub fn parse_from_string(text: String) -> Result<Record, ()> {
        let args: Vec<&str> = text
            .split(";")
            .map(|x| x.trim())
            .filter(|x| x.len() > 0)
            .collect();

        let len = args.len();
        if len == 0 || len > 2 {
            return Err(());
        }

        Ok(Record {
            name: args[0].to_string(),
            category: match len == 2 {
                true => args[1].to_string(),
                false => String::new(),
            },
            value: 0.0,
        })
    }
}
impl Name {
    pub fn new() -> Name {
        Name {
            keyword: String::new(),
            name: String::new(),
        }
    }
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
            name: match len == 2 {
                true => args[1].to_string(),
                false => String::new(),
            }
        })
    }
}

#[cfg(test)]
mod tests {

    use regex::Regex;

    use crate::adventure::Comparison;

    use super::{Choice, Condition, Record, StoryResult, Test, Page, Adventure};

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
record: stuffed animals; resources;".to_string();
        let adventure = Adventure::parse_from_string(data, "damsel".to_string()).unwrap();

        assert_eq!(adventure.title, "Damsel in Distress");
        assert_eq!(adventure.description, "This is a story about a knight who faces a dragon to save the princess");
        assert_eq!(adventure.start, "at_the_castle_ruins");
        assert_eq!(adventure.records.len(), 2);

        assert_eq!(adventure.records[0].name, "confidence");
        assert_eq!(adventure.records[0].category, "attributes");
        assert_eq!(adventure.records[1].name, "stuffed animals");
        assert_eq!(adventure.records[1].category, "resources");
    }
}
