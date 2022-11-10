use std::{
    collections::{HashMap, VecDeque},
    fmt::Display,
    path::PathBuf,
};

use regex::Regex;

use crate::evaluation::{evaluate_and_compare, EvaluationError, Random};

pub const GAME_OVER_KEYWORD: &str = "game over";

/// Describes an error that might have occured during parsing of adventure element
#[derive(Debug)]
pub enum ParsingError {
    ValueNaN(String),
    IncorrectElementCount(String, usize),
    ElementPairMissing(String),
    Invalid(String),
    IncomplatePage(Page),
    MissingRecord(String),
}
/// Holds basic information about adventure, including records, names and path where all the pages can be loaded from
#[derive(Default, Clone)]
pub struct Adventure {
    pub title: String,
    pub description: String,
    pub path: String,
    pub start: String,
    pub records: HashMap<String, Record>,
    pub names: HashMap<String, Name>,
}
/// Represents a numeric value that is tracked throughout an adventure
///
/// It is most useful for branching adventure paths through Tests and Conditions
#[derive(Clone, PartialEq, Debug)]
pub struct Record {
    pub category: String,
    pub name: String,
    pub value: i32,
}
/// Represents a string value that is displayable within adventure page story and title
///
/// It's useful for changing certain words within pages or as a container for titles or names of characters or places that would be typo prone otherwise
#[derive(Clone, PartialEq, Debug)]
pub struct Name {
    pub keyword: String,
    pub value: String,
}
/// Holds both title and story text for an individual page, as well as choices leading to other pages
#[derive(Debug, Default)]
pub struct Page {
    pub title: String,
    pub story: String,
    pub choices: Vec<Choice>,
    pub conditions: HashMap<String, Condition>,
    pub tests: HashMap<String, Test>,
    pub results: HashMap<String, StoryResult>,
}
/// Helper enum for comparing two expressions
#[derive(Debug, Eq, PartialEq, Default)]
pub enum Comparison {
    #[default]
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Equal,
    NotEqual,
}
/// Holds information allowing a story page to transition to another page
///
/// Results can also hold a list of pairs for mutating adventure records and names allowing those to change in reaction to user choice
#[derive(Debug, Default, PartialEq)]
pub struct StoryResult {
    pub name: String,
    pub next_page: String,
    /// Consists of keys that are record or name keywords, and unevaluated expressions as values that represent how the records or names are changed
    pub side_effects: HashMap<String, String>,
}
/// Holds expressions that based on their evaluation and comparison, lead to two different results of a page.
#[derive(Debug, Default, PartialEq)]
pub struct Test {
    pub name: String,
    pub expression_r: String,
    pub comparison: Comparison,
    pub expression_l: String,
    pub success_result: String,
    pub failure_result: String,
}
/// Represents a text available to player as a choice in response to presented story
///
/// The choice have either a test or a result that it points to, allowing progression to a different page
#[derive(Debug, Default, PartialEq)]
pub struct Choice {
    pub text: String,
    pub condition: String,
    pub test: String,
    pub result: String,
}
/// Holds two expressions and comparison type used in determining whatever a choice is available to be chosen by the player
#[derive(Debug, Default, PartialEq)]
pub struct Condition {
    pub name: String,
    pub expression_r: String,
    pub comparison: Comparison,
    pub expression_l: String,
}
// those are for matching tags in Choice during parsing from string so we can figure out which choices should be connected to other elements.
const REGEX_CONDITION_IN_CHOICE: &str = r"\{\s*condition:\s*(\w+(?:\s|\w)*)\s*\}";
const REGEX_TEST_IN_CHOICE: &str = r"\{\s*test:\s*(\w+(?:\s|\w)*)\s*\}";
const REGEX_RESULT_IN_CHOICE: &str = r"\{\s*result:\s*(\w+(?:\s|\w)*)\s*\}";

/// Creates a Regex match for specified keyword
pub fn regex_match_keyword(keyword: &str) -> Result<Regex, regex::Error> {
    regex::Regex::new(&format!(r"\[\s*({})\s*\]", keyword))
}
/// Turns a string into a keyword that can be matched within parts of adventure page text
pub fn create_keyword(keyword: &str) -> String {
    format!("[{}]", keyword)
}
/// Tests if the keyword can be correctly matched in text
pub fn is_keyword_valid(keyword: &str) -> bool {
    if let Ok(r) = regex_match_keyword(keyword) {
        let test = format!("[{}]", keyword);
        return r.is_match(&test);
    }
    false
}

impl Display for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParsingError::ValueNaN(v) => write!(f, "Value is not a number in string {}", v),
            ParsingError::IncorrectElementCount(v, c) => write!(
                f,
                "Incorrect amount of elements in {}, expected {} elements",
                v, c
            ),
            ParsingError::Invalid(v) => write!(f, "Invalid data: {}", v),
            ParsingError::ElementPairMissing(v) => {
                write!(f, "Expected element pair missing in {}", v)
            }
            ParsingError::IncomplatePage(p) => write!(f, "The page is incomplete: {:?}", p),
            ParsingError::MissingRecord(p) => write!(f, "Record {} is missing", p),
        }
    }
}

impl Adventure {
    /// Creates an adventure from text string
    ///
    /// The function goes through text line by line and looks for keywords at start, if it finds a keyword, it will process it.
    /// In case where line doesn't start in a keyword, the line will be added to adventure description if that's the last keyword that was read.
    ///
    /// Note that path can be relative or absolute
    pub fn parse_from_string(text: String, path: String) -> Result<Adventure, ParsingError> {
        let mut adv = Adventure::default();

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
                let rec = Record::parse_from_string(text)?;
                adv.records.insert(rec.name.clone(), rec);
            } else if line.starts_with("name:") {
                flag = 0;
                let text = line.replacen("name:", "", 1);
                let name = Name::parse_from_string(text)?;
                adv.names.insert(name.keyword.clone(), name);
            } else {
                if flag == 1 {
                    adv.description = adv.description + line;
                }
            }
        }
        adv.path = path;

        if adv.is_bare_minimum() {
            return Ok(adv);
        } else {
            return Err(ParsingError::Invalid(text));
        }
    }
    /// Turns the adventure into a string representation that can be either saved to drive or parsed back into adventure
    ///
    /// # Limitations
    /// Function will not serialize adventure path since those are determined during program start when loading the adventures
    pub fn serialize_to_string(&self) -> String {
        let mut ser = format!(
            "title: {}\ndescription: {}\nstart: {}",
            self.title, self.description, self.start
        );
        self.records
            .iter()
            .for_each(|x| ser = format!("{}\nrecord: {}", ser, x.1.serialize_to_string()));
        self.names
            .iter()
            .for_each(|x| ser = format!("{}\nname: {}", ser, x.1.serialize_to_string()));
        ser
    }
    /// Tests if the adventure has bare minimum to be considered as loaded
    pub fn is_bare_minimum(&self) -> bool {
        if self.title.len() < 1 {
            return false;
        }
        if self.path.len() < 1 {
            return false;
        }

        true
    }
    /// Tests if the adventure has bare minimum information to be considered playable
    pub fn is_playable(&self) -> bool {
        if self.start.len() == 0 {
            return false;
        }
        let path = PathBuf::from(&self.path);
        if path.exists() == false {
            return false;
        }
        true
    }
    /// Updates a keyword of a record to a new one
    pub fn update_record(&mut self, old: &str, new: Record) {
        if let Some(_) = self.records.remove(old) {
            self.records.insert(new.name.clone(), new);
        } else {
            println!("Failed to find a record {} to update", old);
        }
    }
    /// Updates a keyword of a name to a new one
    pub fn update_name(&mut self, old: &str, new: Name) {
        if let Some(_) = self.names.remove(old) {
            self.names.insert(new.keyword.clone(), new);
        } else {
            println!("Failed to find a name {} to update", old);
        }
    }
}
/// Replaces a regex matched string slices within source with a new string slice
macro_rules! replace_with_regex {
    ($regex:expr, $source:expr, $new:expr) => {
        if let Some(cap) = $regex.captures(&$source) {
            let mut i = 1;
            let mut buff = $source.clone();
            while let Some(c) = cap.get(i) {
                buff.replace_range(c.range(), $new);
                i += 1;
            }
            $source = buff;
        }
    };
}
impl Page {
    /// Parses string into Page. It will return error if the text isn't valid page
    pub fn parse_from_string(text: String) -> Result<Page, ParsingError> {
        // creating empty page to populate
        let mut page = Page::default();

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
                )?;
                page.choices.push(cho);
            } else if line.starts_with("condition:") {
                story_line = false;

                let con = Condition::parse_from_string(line.replacen("condition:", "", 1))?;

                page.conditions.insert(con.name.clone(), con);
            } else if line.starts_with("test:") {
                story_line = false;

                let test = Test::parse_from_string(line.replacen("test:", "", 1))?;
                page.tests.insert(test.name.clone(), test);
            } else if line.starts_with("result:") {
                story_line = false;

                // failing the page if result doesn't load correctly, like in other cases
                let res = StoryResult::parse_from_string(line.replacen("result:", "", 1))?;
                page.results.insert(res.name.clone(), res);
            } else if story_line {
                // adding a line to story if it's immediately after story keyword and doesn't match any other keywords
                page.story = format!("{}\n{}", page.story, line);
            }
        }
        if page.is_playable() {
            Ok(page)
        } else {
            Err(ParsingError::IncomplatePage(page))
        }
    }
    /// Transforms page into a string representation of it, suitable for saving onto drive or parsing back into a page struct
    pub fn serialize_to_string(&self) -> String {
        let mut ser = format!("title: {}\nstory: {}", self.title, self.story);
        self.choices
            .iter()
            .for_each(|x| ser = format!("{}\nchoice: {}", ser, x.serialize_to_string()));
        self.conditions
            .iter()
            .for_each(|x| ser = format!("{}\ncondition: {}", ser, x.1.serialize_to_string()));
        self.tests
            .iter()
            .for_each(|x| ser = format!("{}\ntest: {}", ser, x.1.serialize_to_string()));
        self.results
            .iter()
            .for_each(|x| ser = format!("{}\nresult: {}", ser, x.1.serialize_to_string()));
        ser
    }
    /// Tests if the page is playable, meaning it has a story text, and a choice that leads somewhere
    pub fn is_playable(&self) -> bool {
        if self.story.len() < 1 {
            return false;
        }
        if self.choices.len() < 1 {
            return false;
        }
        if self.results.len() < 1 {
            for choice in self.choices.iter() {
                if choice.is_game_over() == false {
                    return false;
                }
            }
        }
        true
    }
    /// Tests if provided keyword is present within the page or its subcontents
    ///
    /// The keyword should be a raw text as the function will turn it into a matchable keyword
    pub fn is_keyword_present(&self, keyword: &str) -> bool {
        let regex = regex_match_keyword(keyword);
        if let Err(_) = regex {
            return false;
        }
        let regex = regex.unwrap();
        if regex.is_match(&self.story) {
            return true;
        }
        if regex.is_match(&self.title) {
            return true;
        }
        for choice in self.choices.iter() {
            if choice.is_keyword_present(keyword) {
                return true;
            }
        }
        for condition in self.conditions.iter() {
            if condition.1.is_keyword_present(keyword) {
                return true;
            }
        }
        for test in self.tests.iter() {
            if test.1.is_keyword_present(keyword) {
                return true;
            }
        }
        for result in self.results.iter() {
            if result.1.is_keyword_present(keyword) {
                return true;
            }
        }
        false
    }
    /// Renames all occurances of a keyword within the page and subcomponents to a new string.
    ///
    /// Both strings need to be raw keywords as the function will turn them into matchable keywords
    pub fn rename_keyword(&mut self, old: &str, new: &str) {
        let regex = regex_match_keyword(old);
        if let Err(_) = regex {
            return;
        }
        let regex = regex.unwrap();
        replace_with_regex!(regex, self.story, new);
        replace_with_regex!(regex, self.title, new);
        self.choices
            .iter_mut()
            .for_each(|x| x.rename_keyword(&regex, new));
        self.conditions
            .iter_mut()
            .for_each(|x| x.1.rename_keyword(&regex, new));
        self.tests
            .iter_mut()
            .for_each(|x| x.1.rename_keyword(&regex, new));
        self.results
            .iter_mut()
            .for_each(|x| x.1.rename_keyword(&regex, old, new));
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
    /// Parses string into Choice.
    ///
    /// It requires to be supplied with preconfigured Regex matches which capture name of the matched result
    pub fn parse_from_string(
        mut text: String,
        match_condition: &Regex,
        match_test: &Regex,
        match_result: &Regex,
    ) -> Result<Choice, ParsingError> {
        let mut choice = Choice::default();
        // we use macros here to extract appropriate keywords into their places.
        insert_in_choice!(match_condition, choice.condition, text);
        insert_in_choice!(match_test, choice.test, text);
        insert_in_choice!(match_result, choice.result, text);

        // we finish up by assigning text with keywords extracted and push it into the page
        choice.text = text.trim().to_string();
        if choice.is_valid() {
            Ok(choice)
        } else {
            Err(ParsingError::Invalid(text))
        }
    }
    /// Transforms the choice into a string representation
    fn serialize_to_string(&self) -> String {
        let mut ser = self.text.clone();
        if self.condition.len() > 0 {
            ser += &format!("{{condition: {}}}", self.condition);
        }
        if self.test.len() > 0 {
            ser += &format!("{{test: {}}}", self.test);
        } else if self.result.len() > 0 {
            ser += &format!("{{result: {}}}", self.result);
        } else {
            ser += &format!("{{result: {}}}", GAME_OVER_KEYWORD);
        }

        ser
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
    /// Tests if the choice leads to end of a game
    pub fn is_game_over(&self) -> bool {
        self.result == GAME_OVER_KEYWORD
    }
    /// Tests if the choice is guarded behind a condition
    pub fn has_condition(&self) -> bool {
        self.condition.len() > 0
    }
    /// Tests if the choice contains a keyword within its text
    pub fn is_keyword_present(&self, keyword: &str) -> bool {
        let regex = regex_match_keyword(keyword);
        if let Err(_) = regex {
            return false;
        }
        let regex = regex.unwrap();
        regex.is_match(&self.text)
    }
    /// Renames a keyword within the choice text
    fn rename_keyword(&mut self, regex: &Regex, new: &str) {
        replace_with_regex!(regex, self.text, new);
    }
}
impl From<&str> for Comparison {
    /// Less than or equal is default for anything that doesn't match expected comparisons. Not sure if I should leave it like this or error
    fn from(item: &str) -> Self {
        match item.trim() {
            ">" => Comparison::Greater,
            ">=" => Comparison::GreaterEqual,
            "=" => Comparison::Equal,
            "==" => Comparison::Equal,
            "!" => Comparison::NotEqual,
            "!=" => Comparison::NotEqual,
            "<" => Comparison::Less,
            _ => Comparison::LessEqual,
        }
    }
}
impl From<String> for Comparison {
    fn from(item: String) -> Self {
        Comparison::from(item.as_str())
    }
}
impl Display for Comparison {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Comparison::Greater => write!(f, ">"),
            Comparison::GreaterEqual => write!(f, ">="),
            Comparison::Less => write!(f, "<"),
            Comparison::LessEqual => write!(f, "<="),
            Comparison::Equal => write!(f, "=="),
            Comparison::NotEqual => write!(f, "!="),
        }
    }
}
impl Comparison {
    /// Performs a test between two values according to the comparison type
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
    /// Returns a string suitable to use in FLTK Choice widget
    pub fn as_choice() -> String {
        ">|>=|<|<=|=|!=".to_string()
    }
    /// Converts the comparison to a number usable for indexing values in FLTK Choice widget
    pub fn to_index(&self) -> i32 {
        match self {
            Comparison::Greater => 0,
            Comparison::GreaterEqual => 1,
            Comparison::Less => 2,
            Comparison::LessEqual => 3,
            Comparison::Equal => 4,
            Comparison::NotEqual => 5,
        }
    }
}
impl Condition {
    /// Creates a Condition reading its data from provided string
    ///
    /// # Error
    /// The string needs to have 4 elements divided by ; to be parsed correctly
    pub fn parse_from_string(text: String) -> Result<Condition, ParsingError> {
        // splitting the text into parts. Expected order of data is name, exp right, comparison, exp left. We filter out empty strings
        let args: Vec<&str> = text
            .split(";")
            .map(|x| x.trim())
            .filter(|x| x.len() > 0)
            .collect();

        // function will report error if incorrect amount of data was found.
        if args.len() != 4 {
            return Err(ParsingError::IncorrectElementCount(text, 4));
        }

        // constructing the condition.
        Ok(Condition {
            name: args[0].to_string(),
            expression_l: args[1].to_string(),
            comparison: Comparison::from(args[2]),
            expression_r: args[3].to_string(),
        })
    }
    /// Transforms the Condition into its string representation
    fn serialize_to_string(&self) -> String {
        format!(
            "{};{};{};{}",
            self.name, self.expression_l, self.comparison, self.expression_r
        )
    }
    /// Performs an evaluation on itself, evaluating and comparing both left and right side expressions
    pub fn evaluate(
        &self,
        records: &HashMap<String, Record>,
        rand: &mut Random,
    ) -> Result<bool, EvaluationError> {
        evaluate_and_compare(
            &self.expression_l,
            &self.expression_r,
            &self.comparison,
            records,
            rand,
        )
    }
    /// Tests if a keyword is present within the condition's expressions
    pub fn is_keyword_present(&self, keyword: &str) -> bool {
        let regex = regex_match_keyword(keyword);
        if let Err(_) = regex {
            return false;
        }
        let regex = regex.unwrap();
        if regex.is_match(&self.expression_l) {
            return true;
        }
        regex.is_match(&self.expression_r)
    }
    /// Renames a keyword to a new one within each of condition's expressions.
    ///
    /// Provided string needs to be a raw keyword since the function will turn it into a matchable keyword
    fn rename_keyword(&mut self, regex: &Regex, new: &str) {
        replace_with_regex!(regex, self.expression_l, new);
        replace_with_regex!(regex, self.expression_r, new);
    }
}
impl Test {
    /// Parses a Test out of a string
    ///
    /// # Error
    /// The string needs to use ; as separator and have 6 elements to be parsed into Test components
    pub fn parse_from_string(text: String) -> Result<Test, ParsingError> {
        let args: Vec<&str> = text
            .split(";")
            .map(|x| x.trim())
            .filter(|x| x.len() > 0)
            .collect();

        if args.len() != 6 {
            return Err(ParsingError::IncorrectElementCount(text, 6));
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
    /// Transforms the test into a string representation of it
    fn serialize_to_string(&self) -> String {
        format!(
            "{};{};{};{};{};{}",
            self.name,
            self.expression_l,
            self.comparison,
            self.expression_r,
            self.success_result,
            self.failure_result
        )
    }
    /// Evaluates the expressions within the test and compares them. Then the function returns either a success or failure result name
    ///
    /// # Error
    /// If evaluation fails on either expression, error will be returned instead.
    pub fn evaluate(
        &self,
        records: &HashMap<String, Record>,
        rand: &mut Random,
    ) -> Result<&String, EvaluationError> {
        match evaluate_and_compare(
            &self.expression_l,
            &self.expression_r,
            &self.comparison,
            records,
            rand,
        ) {
            Ok(v) => {
                if v {
                    Ok(&self.success_result)
                } else {
                    Ok(&self.failure_result)
                }
            }
            Err(e) => Err(e),
        }
    }
    /// Tests if a keyword is present in either of expressions of the test
    ///
    /// The string should be a raw keyword, the function will turn it into a matchable keyword
    pub fn is_keyword_present(&self, keyword: &str) -> bool {
        let regex = regex_match_keyword(keyword);
        if let Err(_) = regex {
            return false;
        }
        let regex = regex.unwrap();
        if regex.is_match(&self.expression_l) {
            return true;
        }
        regex.is_match(&self.expression_r)
    }
    /// Renames a keyword in either of expressions to a new one based on provided regex
    fn rename_keyword(&mut self, regex: &Regex, new: &str) {
        replace_with_regex!(regex, self.expression_l, new);
        replace_with_regex!(regex, self.expression_r, new);
    }
}
impl StoryResult {
    /// Parses a string into a StoryResult
    ///
    /// # Error
    /// The string needs to be separated with ; and contain at least 2 elements to be valid
    ///
    /// The third and following elements are pairs of keyword and expression, they need to be in even numbers, otherwise the string is considered not valid.
    pub fn parse_from_string(text: String) -> Result<StoryResult, ParsingError> {
        let mut args: VecDeque<&str> = text
            .split(";")
            .map(|x| x.trim())
            .filter(|x| x.len() > 0)
            .collect();

        if args.len() < 2 {
            return Err(ParsingError::IncorrectElementCount(text, 2));
        }
        let name = args.pop_front().unwrap().to_string();
        let next_page = args.pop_front().unwrap().to_string();
        let mut side_effects = HashMap::new();

        while let Some(ar) = args.pop_front() {
            // if it's not the end that means we are constructing record change
            if let Some(val) = args.pop_front() {
                side_effects.insert(ar.to_string(), val.to_string());
            } else {
                // error because we have keyword but not value
                return Err(ParsingError::ElementPairMissing(text));
            }
        }

        Ok(StoryResult {
            name,
            next_page,
            side_effects,
        })
    }
    /// Transforms the StoryResult into a string representation
    fn serialize_to_string(&self) -> String {
        let mut ser = format!("{};{}", self.name, self.next_page);
        self.side_effects
            .iter()
            .for_each(|x| ser = format!("{};{};{}", ser, x.0, x.1));
        ser
    }
    /// Tests if a keyword is present in any of this StoryResult's side effects
    ///
    /// The provided keyword should be raw, the function will turn it into matchable keyword
    pub fn is_keyword_present(&self, keyword: &str) -> bool {
        let regex = match regex_match_keyword(keyword) {
            Ok(r) => r,
            Err(_) => return false,
        };
        self.side_effects
            .iter()
            .any(|x| regex.is_match(x.0) || regex.is_match(x.1))
    }
    /// Renames a keyword within side effects of the result to a new name
    ///
    /// Keywords should be raw, the function will turn them into matchable keywords
    fn rename_keyword(&mut self, regex: &Regex, old: &str, new: &str) {
        if let Some(v) = self.side_effects.remove(old) {
            self.side_effects.insert(new.to_string(), v);
        }
        self.side_effects
            .iter_mut()
            .for_each(|x| replace_with_regex!(regex, *x.1, new));
    }
}
impl Record {
    /// Creates a record from a text data.
    pub fn parse_from_string(text: String) -> Result<Record, ParsingError> {
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
            }
            2 => {
                name = args[0].to_string();
                if let Ok(n) = args[1].parse() {
                    value = n;
                    category = String::new();
                } else {
                    value = 0;
                    category = args[1].to_string();
                }
            }
            3 => {
                name = args[0].to_string();
                category = args[1].to_string();
                if let Ok(n) = args[2].parse() {
                    value = n;
                } else {
                    return Err(ParsingError::ValueNaN(text));
                }
            }
            _ => return Err(ParsingError::IncorrectElementCount(text, 3)),
        }
        Ok(Record {
            name,
            category,
            value,
        })
    }
    /// Turns the record into a string representation
    fn serialize_to_string(&self) -> String {
        format!("{};{};{}", self.name, self.category, self.value)
    }
    /// Convenience function that turns the record value into string
    pub fn value_as_string(&self) -> String {
        (self.value as i32).to_string()
    }
}
impl Name {
    /// Parses a string into a Name
    ///
    /// The string needs to be separated with ; and have either one or two elements to be valid
    pub fn parse_from_string(text: String) -> Result<Name, ParsingError> {
        let args: Vec<&str> = text
            .split(";")
            .map(|x| x.trim())
            .filter(|x| x.len() > 0)
            .collect();

        let len = args.len();
        if len == 0 || len > 2 {
            return Err(ParsingError::IncorrectElementCount(text, 2));
        }

        Ok(Name {
            keyword: args[0].to_string(),
            value: match len == 2 {
                true => args[1].to_string(),
                false => String::new(),
            },
        })
    }
    /// Turns the name into a string representation
    fn serialize_to_string(&self) -> String {
        format!("{};{}", self.keyword, self.value)
    }
}

#[cfg(test)]
mod tests {

    use std::collections::HashMap;

    use regex::Regex;

    use crate::adventure::Comparison;

    use super::{
        regex_match_keyword, Adventure, Choice, Condition, Name, Page, Record, StoryResult, Test,
    };

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
        assert_eq!(res.next_page, "next scene");
    }
    #[test]
    fn result_record_mod_parse() {
        let data = "proceed; next_scene; strength; 1;".to_string();
        let res = StoryResult::parse_from_string(data).unwrap();
        assert_eq!(res.name, "proceed");
        assert_eq!(res.next_page, "next_scene");
        assert_eq!(res.side_effects.get("strength").unwrap(), "1");
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
    fn capture_keyword() {
        let data = "this is a test string with a [spaced keyword] that should be captured";
        let regex = regex_match_keyword("spaced keyword").unwrap();
        if let Some(cap) = regex.captures(&data) {
            let r = cap.get(1);
            assert_eq!(r.unwrap().as_str(), "spaced keyword");
        } else {
            assert!(false);
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
    #[test]
    fn serializing_adventure_metadata() {
        let a = Adventure {
            title: "test".to_string(),
            description: "this is a test adventure".to_string(),
            start: "start-page".to_string(),
            records: {
                let mut r = HashMap::new();
                r.insert(
                    "first".to_string(),
                    Record {
                        name: "first".to_string(),
                        category: "".to_string(),
                        value: 1,
                    },
                );
                r.insert(
                    "second".to_string(),
                    Record {
                        name: "second".to_string(),
                        category: "".to_string(),
                        value: 4,
                    },
                );
                r
            },
            names: {
                let mut n = HashMap::new();
                n.insert(
                    "hero".to_string(),
                    Name {
                        keyword: "hero".to_string(),
                        value: "Prince Charming".to_string(),
                    },
                );
                n.insert(
                    "vilain".to_string(),
                    Name {
                        keyword: "vilain".to_string(),
                        value: "Evil Witch".to_string(),
                    },
                );
                n
            },
            ..Default::default()
        };

        let serialized = a.serialize_to_string();
        let b = Adventure::parse_from_string(serialized, "path".to_string()).unwrap();
        assert_eq!(a.title, b.title);
        assert_eq!(a.description, b.description);
        assert_eq!(a.start, b.start);
        assert_eq!(a.records.get("first"), b.records.get("first"));
        assert_eq!(a.records.get("second"), b.records.get("second"));
        assert_eq!(a.names.get("hero"), b.names.get("hero"));
        assert_eq!(a.names.get("vilain"), b.names.get("vilain"));
    }
    #[test]
    fn serializing_page() {
        let a = Page {
            title: "test title".to_string(),
            story: "this is a test story".to_string(),
            choices: {
                vec![
                    Choice {
                        text: "Default choice".to_string(),
                        result: "game over".to_string(),
                        ..Default::default()
                    },
                    Choice {
                        text: "Conditioned choice".to_string(),
                        condition: "con".to_string(),
                        result: "game over".to_string(),
                        ..Default::default()
                    },
                    Choice {
                        text: "Testing Choice".to_string(),
                        test: "test".to_string(),
                        ..Default::default()
                    },
                    Choice {
                        text: "Resulting choice".to_string(),
                        result: "result".to_string(),
                        ..Default::default()
                    },
                ]
            },
            conditions: {
                let mut c = HashMap::new();
                c.insert(
                    "con".to_string(),
                    Condition {
                        name: "con".to_string(),
                        comparison: Comparison::Greater,
                        expression_l: "1d6".to_string(),
                        expression_r: "2".to_string(),
                    },
                );
                c
            },
            tests: {
                let mut t = HashMap::new();
                t.insert(
                    "test".to_string(),
                    Test {
                        name: "test".to_string(),
                        comparison: Comparison::Greater,
                        expression_l: "1d20".to_string(),
                        expression_r: "10".to_string(),
                        success_result: "result".to_string(),
                        failure_result: "failure".to_string(),
                    },
                );
                t
            },
            results: {
                let mut r = HashMap::new();
                r.insert(
                    "result".to_string(),
                    StoryResult {
                        name: "result".to_string(),
                        next_page: "next".to_string(),
                        side_effects: {
                            let mut se = HashMap::new();
                            se.insert("record".to_string(), "4".to_string());
                            se
                        },
                    },
                );
                r.insert(
                    "failure".to_string(),
                    StoryResult {
                        name: "failure".to_string(),
                        next_page: "loss".to_string(),
                        side_effects: HashMap::new(),
                    },
                );
                r
            },
        };

        let serialized = a.serialize_to_string();
        let b = Page::parse_from_string(serialized).unwrap();
        assert_eq!(a.title, b.title);
        assert_eq!(a.story, b.story);
        assert_eq!(a.choices.len(), b.choices.len());
        a.choices
            .iter()
            .enumerate()
            .for_each(|x| assert_eq!(x.1, b.choices.get(x.0).unwrap()));
        assert_eq!(a.conditions.len(), b.conditions.len());
        a.conditions
            .iter()
            .for_each(|x| assert_eq!(x.1, b.conditions.get(x.0).unwrap()));
        assert_eq!(a.tests.len(), b.tests.len());
        a.tests
            .iter()
            .for_each(|x| assert_eq!(x.1, b.tests.get(x.0).unwrap()));
        assert_eq!(a.results.len(), b.results.len());
        a.results
            .iter()
            .for_each(|x| assert_eq!(x.1, b.results.get(x.0).unwrap()));
    }
}
