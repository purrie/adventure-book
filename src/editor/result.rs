use std::collections::HashMap;

use fltk::{
    app,
    browser::SelectBrowser,
    button::Button,
    draw::Rect,
    enums::Color,
    frame::Frame,
    group::Group,
    image::SvgImage,
    prelude::*,
    text::{TextBuffer, TextEditor},
};

use crate::{
    adventure::{Adventure, Name, Page, Record, StoryResult},
    dialog::{ask_for_choice, ask_for_text, ask_to_confirm},
    evaluation::{evaluate_expression, Random},
    file::signal_error,
    icons::{BIN_ICON, GEAR_ICON},
};

use super::{emit, Event};

/// Widgets for customizing results of the page
///
/// Lists available results for the page
/// It will give a drop down for choosing the next page
/// It will give a growing field for adding changes to records or names
pub struct ResultEditor {
    selector_results: SelectBrowser,
    selector_effects: SelectBrowser,
    name: Frame,
    effect: Frame,
    next_page: fltk::menu::Choice,
    effect_value: TextEditor,
    next_page_label: Frame,
    butt_rec: Button,
    butt_nam: Button,
}

impl ResultEditor {
    /// Creates UI for result editor
    pub fn new(area: Rect) -> Self {
        let group = Group::new(area.x, area.y, area.w, area.h, "Results");

        let font_size = app::font_size();

        let x_column_1 = area.x;
        let w_column_1 = area.w / 3;

        let margin = 20;
        let x_column_2 = x_column_1 + w_column_1 + margin;
        let w_column_2 = area.w - w_column_1 - margin * 2;

        // subcolumns in second column
        let margin2 = 5;
        let x_column_3 = x_column_2 + margin2;
        let w_column_3 = w_column_2 / 2 - margin2 * 2;
        let x_column_4 = x_column_3 + w_column_3 + margin2 * 2;

        let h_line = font_size + font_size / 2;

        // vertical result selector coords
        let y_results = area.y;
        let h_result = area.h / 2 - font_size;

        // vertical side effect selector coords
        let y_mods = y_results + h_result + font_size;
        let h_mods = area.h - h_result - font_size * 2;

        // result manipulation widgets
        let y_name = y_results + font_size;
        let y_page = y_name + h_line * 2;

        // controls for selector buttons
        let y_butt_result = y_results + h_result;
        let y_butt_mod = y_mods + h_mods;
        let w_butt = font_size;
        let h_butt = w_butt;

        let x_add = x_column_1;
        let x_ren = x_add + w_butt;
        let x_rem = x_column_1 + w_column_1 - w_butt;

        // controls for side effect second column
        let y_effect = y_results + h_result + h_line;
        let y_butt = y_effect + h_line * 2;
        let y_exp = y_butt + h_line * 2;

        let mut select_result =
            SelectBrowser::new(x_column_1, y_results, w_column_1, h_result, "Results");
        let mut select_mod =
            SelectBrowser::new(x_column_1, y_mods, w_column_1, h_mods, "Modifications");

        let mut butt_add_result = Button::new(x_add, y_butt_result, w_butt, h_butt, "@+");
        let mut butt_ren_result = Button::new(x_ren, y_butt_result, w_butt, h_butt, None);
        let mut butt_rem_result = Button::new(x_rem, y_butt_result, w_butt, h_butt, None);
        let mut butt_rem_effect = Button::new(x_rem, y_butt_mod, w_butt, h_butt, None); // no add or rename because the names are constant and you add in other controls

        let name = Frame::new(x_column_2, y_name, w_column_2, h_line, "Name");
        let next_page_label = Frame::new(
            x_column_2,
            y_page - font_size,
            w_column_2,
            h_line,
            "Next Page",
        );
        let next_page = fltk::menu::Choice::new(x_column_2, y_page, w_column_2, h_line, None);

        let effect = Frame::new(x_column_2, y_effect, w_column_2, h_line, None);
        let mut butt_rec = Button::new(x_column_3, y_butt, w_column_3, h_line, "Add Record");
        let mut butt_nam = Button::new(x_column_4, y_butt, w_column_3, h_line, "Add Name");
        let mut expression =
            TextEditor::new(x_column_2, y_exp, w_column_2, h_line, "Value expression");

        group.end();

        let (sender, _r) = app::channel();

        select_result.set_callback({
            let sender = sender.clone();
            let mut old_result: Option<String> = None;
            move |x| {
                if let Some(text) = x.selected_text() {
                    if let Some(old) = &old_result {
                        if old == &text {
                            return;
                        }
                        sender.send(emit!(Event::SaveResult(Some(old.clone()))));
                    }
                    old_result = Some(text.clone());
                    sender.send(emit!(Event::LoadResult(text)));
                } else {
                    old_result = None;
                }
            }
        });
        select_mod.set_callback({
            let sender = sender.clone();
            let mut old_result: Option<String> = None;
            move |x| {
                if let Some(text) = x.selected_text() {
                    if let Some(old) = &old_result {
                        if old == &text {
                            return;
                        }
                        sender.send(emit!(Event::SaveSideEffect(Some(old.clone()))))
                    }
                    old_result = Some(text.clone());
                    sender.send(emit!(Event::LoadSideEffect(text)));
                } else {
                    old_result = None;
                }
            }
        });
        select_result.set_selection_color(Color::Blue);
        select_mod.set_selection_color(Color::Blue);
        butt_add_result.set_callback({
            let sender = sender.clone();
            move |_| {
                sender.send(emit!(Event::SaveResult(None)));
                sender.send(emit!(Event::AddResult));
            }
        });
        butt_ren_result.emit(sender.clone(), emit!(Event::RenameResult));
        butt_rem_result.emit(sender.clone(), emit!(Event::RemoveResult));
        butt_rem_effect.emit(sender.clone(), emit!(Event::RemoveSideEffect));
        butt_rec.set_callback({
            let sender = sender.clone();
            move |_| {
                sender.send(emit!(Event::SaveSideEffect(None)));
                sender.send(emit!(Event::AddSideEffectRecord));
            }
        });
        butt_nam.set_callback({
            move |_| {
                sender.send(emit!(Event::SaveSideEffect(None)));
                sender.send(emit!(Event::AddSideEffectName));
            }
        });
        expression.set_buffer(TextBuffer::default());

        let mut gear = SvgImage::from_data(GEAR_ICON).unwrap();
        let mut bin = SvgImage::from_data(BIN_ICON).unwrap();
        gear.scale(w_butt, h_butt, false, true);
        bin.scale(w_butt, h_butt, false, true);

        butt_ren_result.set_image(Some(gear));
        butt_rem_result.set_image(Some(bin.clone()));
        butt_rem_effect.set_image(Some(bin));

        Self {
            selector_results: select_result,
            selector_effects: select_mod,
            name,
            effect,
            next_page,
            next_page_label,
            butt_rec,
            butt_nam,
            effect_value: expression,
        }
    }
    /// Returns selected result or None if the list is empty or there's nothing selected
    fn selected_result(&self) -> Option<String> {
        self.selector_results.selected_text()
    }
    /// Returns name of selected side effect or None if the list is empty or nothing is selected
    fn selected_side_effect(&self) -> Option<String> {
        self.selector_effects.selected_text()
    }
    /// Tests validity of currently selected side effect value, showing errors to user if those happen, and returning the value if they don't
    fn evaluate_correct_side_effect_value(
        &self,
        is_record: bool,
        res: &str,
        se: Option<String>,
        records: &HashMap<String, Record>,
    ) -> Option<String> {
        let se = match se {
            None => match self.selected_side_effect() {
                Some(s) => s,
                None => return None,
            },
            Some(s) => s,
        };
        let value = self.effect_value.buffer().unwrap().text();
        match value.trim().to_string() {
            x if is_record && x.len() == 0 => {
                signal_error!(
                    "Warning! A record cannot be empty, expression for {} in {} will be set to 1",
                    &se,
                    res
                );
                Some("1".to_string())
            }
            x if is_record && x == "0" => {
                signal_error!("Warning! A record cannot be equal to 0 as it would introduce no change, expression for {} in {} will be set to 1", &se, res);
                Some("1".to_string())
            }
            x if is_record => {
                let mut r = Random::new(69);
                match evaluate_expression(&x, records, &mut r) {
                    Ok(_) => Some(x),
                    Err(er) => match &er {
                        crate::evaluation::EvaluationError::DivisionByZero => {
                            signal_error!("Warning! Evaluation of {} in {} resulted in division by zero error. Saving process will proceed normally, as this may be a false alert caused by default record value.",
                                          &se, &res);
                            Some(x)
                        }
                        crate::evaluation::EvaluationError::NotANumber(_) => {
                            signal_error!("Warning! Expression of {} is invalid. {}", &se, er);
                            None
                        }
                        crate::evaluation::EvaluationError::InvalidDieExpression(_) => {
                            signal_error!("Warning! Expression of {} is invalid. {}", &se, er);
                            None
                        }
                        crate::evaluation::EvaluationError::MissingDicePoolEvaluator(_) => {
                            signal_error!("Warning! Expression of {} is invalid. {}", &se, er);
                            None
                        }
                    },
                }
            }
            x => Some(x),
        }
    }
    /// Adds a line to the side effect selector. It handles the whole process.
    fn add_side_effect(&mut self, name: &str) {
        self.selector_effects.add(name);
        self.selector_effects.select(self.selector_effects.size());
        self.selector_effects.do_callback();
    }
    /// Loads result into the editor
    fn load_result(&mut self, res: &StoryResult) {
        let mut i = 0;
        while let Some(text) = self.next_page.text(i) {
            if text == res.next_page {
                self.next_page.set_value(i);
                i = 0;
                break;
            }
            i += 1;
        }
        if i > 0 {
            self.next_page.set_value(-1);
        }
        i = 1;
        while let Some(text) = self.selector_results.text(i) {
            if text == res.name {
                self.selector_results.select(i);
                self.selector_results.do_callback();
                break;
            }
            i += 1;
        }
        self.name.set_label(&res.name);
        self.populate_side_effects(res);
        self.show_controls();
    }
    /// Fills out the editor with story result data and selects the first element if present
    fn populate_results(&mut self, res: &HashMap<String, StoryResult>) {
        self.selector_results.clear();
        self.selector_results.do_callback();
        self.selector_effects.clear();
        self.selector_effects.do_callback();
        let mut set = true;
        for r in res.iter() {
            self.selector_results.add(r.0);
            if set {
                self.load_result(r.1);
                self.selector_results.select(1);
                set = false;
            }
        }
        if set {
            self.hide_controls();
        }
    }
    /// Fills the next page choice dropdown with page names
    fn populate_pages(&mut self, pages: &HashMap<String, Page>) {
        self.next_page.clear();
        let mut keys: Vec<&String> = pages.keys().collect();
        keys.sort();
        keys.iter().for_each(|x| self.next_page.add_choice(x));
    }
    /// populates side effect editor
    fn populate_side_effects(&mut self, se: &StoryResult) {
        self.selector_effects.clear();
        self.selector_effects.do_callback();
        let mut set = true;
        for e in se.side_effects.iter() {
            self.selector_effects.add(e.0);
            if set {
                self.selector_effects.select(1);
                self.selector_effects.do_callback();
                set = false;
                self.show_effects();
            }
        }
        if set {
            self.hide_effects();
        }
    }
    fn show_controls(&mut self) {
        self.name.show();
        self.next_page_label.show();
        self.next_page.show();
        self.butt_nam.show();
        self.butt_rec.show();
    }
    fn hide_controls(&mut self) {
        self.name.hide();
        self.next_page_label.hide();
        self.next_page.hide();
        self.butt_nam.hide();
        self.butt_rec.hide();
        self.effect.hide();
        self.effect_value.hide();
    }
    fn show_effects(&mut self) {
        self.effect.show();
        self.effect_value.show();
    }
    fn hide_effects(&mut self) {
        self.effect.hide();
        self.effect_value.hide();
    }
    /// tests if a side effect already exists in the story result
    fn contains_side_effect(&self, name: &str) -> bool {
        let mut i = 1;
        while let Some(s) = self.selector_effects.text(i) {
            if s == name {
                return true;
            }
            i += 1;
        }
        false
    }
    /// Tests if there is at least one side effect in the story result
    fn has_side_effects(&self) -> bool {
        self.selector_effects.size() > 0
    }
    /// Populates result UI with pages and results
    ///
    /// This function ensures the loading is performed in correct order
    /// First pages, then results, otherwise fltk causes segfault when trying to access dropdowns
    pub fn populate(
        &mut self,
        results: &HashMap<String, StoryResult>,
        pages: &HashMap<String, Page>,
    ) {
        self.populate_pages(pages);
        self.populate_results(results);
    }
    /// Event response to load a result into editor by name
    pub fn load(&mut self, results: &HashMap<String, StoryResult>, res: String) {
        if let Some(r) = results.get(&res) {
            self.load_result(&r);
        }
    }
    /// Event response to save specified result from the editor into the collection
    ///
    /// If no res is provided, then it will save currently selected result
    pub fn save(
        &self,
        results: &mut HashMap<String, StoryResult>,
        res: Option<String>,
        adventure: &Adventure,
    ) {
        let selected = match res {
            Some(s) => s,
            None => match self.selected_result() {
                Some(s) => s,
                None => return,
            },
        };
        if let Some(result) = results.get_mut(&selected) {
            // saving the main part of the result
            if let Some(sel) = self.next_page.choice() {
                result.next_page = sel;
            } else {
                signal_error!(
                    "Cannot save Result {} because next page is not chosen",
                    selected
                );
            }

            // this part saves currently selected side effect
            // or if there's none, exit early
            let se = match self.selected_side_effect() {
                Some(s) => s,
                None => {
                    println!("No side effect selected to save");
                    return;
                }
            };
            let is_record = adventure.records.contains_key(&se);
            let value = match self.evaluate_correct_side_effect_value(
                is_record,
                &result.name,
                Some(se.clone()),
                &adventure.records,
            ) {
                Some(x) => x,
                None => {
                    println!(
                        "Save error: couldn't evaluate value of the side effect {} in {}",
                        se, result.name
                    );
                    return;
                }
            };
            result.side_effects.insert(se, value);
        }
    }
    /// Event response that adds a new result into the page collection
    pub fn add(&mut self, results: &mut HashMap<String, StoryResult>) {
        let name = match ask_for_text("Enter name for a new result") {
            Some(n) if n.len() > 0 => n,
            _ => return,
        };
        if results.contains_key(&name) {
            signal_error!("Result with name '{}' already exists", name);
            return;
        }
        let res = StoryResult {
            name: name.clone(),
            ..Default::default()
        };
        self.selector_results.add(&res.name);
        self.selector_results.select(self.selector_results.size());
        self.selector_results.do_callback();
        results.insert(name, res);
    }
    /// Event response that removes currently selected result from the page
    pub fn remove(&mut self, page: &mut Page) {
        let selected = match self.selected_result() {
            Some(s) => s,
            None => return,
        };
        // check if the result is used somewhere
        if page.choices.iter().any(|x| x.result == selected) {
            signal_error!(
                "Result {} is used in a player choice! Cannot remove used result",
                selected
            );
            return;
        }
        if page
            .tests
            .iter()
            .any(|x| x.1.success_result == selected || x.1.failure_result == selected)
        {
            signal_error!(
                "Result {} is used in a test! Cannot remove used result",
                selected
            );
            return;
        }
        if ask_to_confirm(&format!("Are you sure you want to remove {}?", &selected)) {
            page.results.remove(&selected);
            // no need to call populate_side_effects as it is expected of populate_results to do it
            self.populate_results(&page.results);
        }
    }
    /// Event response that renames currently selected result
    ///
    /// It will also update its name when used in choices and tests
    pub fn rename(&mut self, page: &mut Page) {
        let selected = match self.selected_result() {
            Some(s) => s,
            None => return,
        };
        let name = match ask_for_text(&format!("Input a new name for {}", &selected)) {
            Some(n) if n.len() > 0 => n,
            _ => return,
        };

        if let Some(mut res) = page.results.remove(&selected) {
            // updating the name in other parts of the page
            page.choices
                .iter_mut()
                .filter(|x| x.result == selected)
                .for_each(|x| x.result = name.clone());
            for el in page.tests.iter_mut() {
                if el.1.success_result == selected {
                    el.1.success_result = name.clone();
                }
                if el.1.failure_result == selected {
                    el.1.failure_result = name.clone();
                }
            }
            let sel = self.selector_results.value();
            self.selector_results.set_text(sel, &name);
            self.name.set_label(&name);
            res.name = name.clone();
            page.results.insert(name, res);
        }
    }
    /// Event response that saves effect data from UI into specified side effect
    ///
    /// If no name is provided then it will save into currently selected side effect
    pub fn save_effect(
        &mut self,
        results: &mut HashMap<String, StoryResult>,
        adventure: &Adventure,
        se: Option<String>,
    ) {
        if self.has_side_effects() == false {
            println!("Error: Tried to save side effect when none is present in the result");
            return;
        }
        // grabbing result
        let res = match self.selected_result() {
            Some(r) => match results.get_mut(&r) {
                Some(r) => r,
                None => {
                    println!("Save error: Couldn't find selected result");
                    return;
                }
            },
            None => {
                println!("Save error: Automatic match for result selection not found");
                return;
            }
        };
        // grabbing side effect from provided name
        let se = match se {
            Some(s) => s,
            None => match self.selected_side_effect() {
                Some(s) => s,
                None => {
                    println!("Save error: Couldn't find side effect");
                    return;
                }
            },
        };
        let is_record = adventure.records.contains_key(&se);
        let value = match self.evaluate_correct_side_effect_value(
            is_record,
            &res.name,
            Some(se.clone()),
            &adventure.records,
        ) {
            Some(x) => x,
            None => {
                println!(
                    "Save error: couldn't evaluate value of the side effect {}",
                    se
                );
                return;
            }
        };
        res.side_effects.insert(se, value);
    }
    /// Event response that loads a side effect by name into the UI
    pub fn load_effect(&mut self, results: &HashMap<String, StoryResult>, se: String) {
        let selected = match self.selected_result() {
            Some(res) => match results.get(&res) {
                Some(r) => r,
                None => {
                    println!("SideEffect Load error: selected result isn't in the list");
                    return;
                }
            },
            None => {
                println!("SideEffect Load error: no selected result");
                return;
            }
        };
        if let Some(v) = selected.side_effects.get(&se) {
            let different_se;
            // testing if the selection is correct
            if let Some(t) = self.selector_effects.selected_text() {
                different_se = t != se;
            } else {
                different_se = true;
            }
            // setting the selection to correct number if it isn't
            if different_se {
                let mut i = 1;
                while let Some(v) = self.selector_effects.text(i) {
                    if v == se {
                        self.selector_effects.select(i);
                        self.selector_effects.do_callback();
                    }
                    i += 1;
                }
            }
            self.effect.set_label(&se);
            self.effect_value.buffer().unwrap().set_text(v);
            self.show_effects();
        } else {
            println!("SideEffect Load error: couldn't find the effect to load");
        }
    }
    /// Event response that adds a new record side effect to currently selected result
    pub fn add_record(
        &mut self,
        results: &mut HashMap<String, StoryResult>,
        records: &HashMap<String, Record>,
    ) {
        let r = match self.selected_result() {
            Some(r) => match results.get_mut(&r) {
                Some(r) => r,
                None => {
                    println!("Add Record error: Couldn't find selected result");
                    return;
                }
            },
            None => {
                println!("Add Record error: Automatic match for result selection not found");
                return;
            }
        };
        if let Some(choice) = ask_for_choice(
            "Select Record to add",
            records
                .iter()
                .filter(|x| !self.contains_side_effect(x.0))
                .map(|x| x.0),
        ) {
            self.add_side_effect(&choice.1);
            r.side_effects.insert(choice.1, "1".to_string());
        }
    }
    /// Event response that adds a new name side effect to currently selected result
    pub fn add_name(
        &mut self,
        results: &mut HashMap<String, StoryResult>,
        names: &HashMap<String, Name>,
    ) {
        let r = match self.selected_result() {
            Some(r) => match results.get_mut(&r) {
                Some(r) => r,
                None => {
                    println!("Add Record error: Couldn't find selected result");
                    return;
                }
            },
            None => {
                println!("Add Record error: Automatic match for result selection not found");
                return;
            }
        };
        if let Some(choice) = ask_for_choice(
            "Select Name to add",
            names
                .iter()
                .filter(|x| !self.contains_side_effect(x.0))
                .map(|x| x.0),
        ) {
            let defval = format!("[{}]", &choice.1);
            self.add_side_effect(&choice.1);
            r.side_effects.insert(choice.1, defval);
        }
    }
    /// Event response that removes currently selected side effect from the result
    pub fn remove_effect(&mut self, results: &mut HashMap<String, StoryResult>) {
        let selected = match self.selected_side_effect() {
            Some(s) => s,
            None => return,
        };
        let res = match self.selected_result() {
            Some(s) => match results.get_mut(&s) {
                Some(r) => r,
                None => return,
            },
            None => return,
        };
        if ask_to_confirm(&format!(
            "Are you sure you want to remove {} side effect from {} result?",
            &selected, &res.name
        )) {
            res.side_effects.remove(&selected);
            self.populate_side_effects(&res);
        }
    }
}
