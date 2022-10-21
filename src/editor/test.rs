use std::collections::HashMap;

use fltk::{
    app,
    browser::SelectBrowser,
    button::Button,
    draw::Rect,
    frame::Frame,
    group::Group,
    image::SvgImage,
    prelude::*,
    text::{TextBuffer, TextEditor},
};

use crate::{
    adventure::{Comparison, Page, StoryResult, Test},
    dialog::{ask_for_text, ask_to_confirm},
    editor::variables::variable_receiver,
    file::signal_error,
    icons::{BIN_ICON, GEAR_ICON},
    widgets::find_item,
};

use super::{emit, Event};

/// Widgets for editing tests
///
/// Lists tests in page by name
/// Has widgets to customize two expressions and their comparison
/// It provides drop downs to fill success and failure results of the test
pub struct TestEditor {
    selector: SelectBrowser,
    name: Frame,
    expression_left: TextEditor,
    expression_right: TextEditor,
    comparison: fltk::menu::Choice,
    success: fltk::menu::Choice,
    failure: fltk::menu::Choice,
}

impl TestEditor {
    /// Creates UI for editing tests of a page
    pub fn new(area: Rect) -> Self {
        let group = Group::new(area.x, area.y, area.w, area.h, "Tests");

        let font_size = app::font_size();

        let x_selector = area.x;
        let y_selector = area.y;
        let w_selector = area.w / 3;
        let h_selector = area.h - font_size;

        let y_butt = y_selector + h_selector;
        let w_butt = font_size;
        let h_butt = w_butt;

        let x_add = x_selector;
        let x_ren = x_add + w_butt;
        let x_rem = x_selector + w_selector - w_butt;

        let column_margin = 20;
        let x_second_column = x_selector + w_selector + column_margin;
        let w_second_column = area.w - w_selector - column_margin * 2;
        let h_line = font_size + font_size / 2;

        let y_name = y_selector + font_size;
        let y_exp = y_name + h_line * 2;
        let y_comp = y_exp + h_line * 2;
        let y_exp2 = y_comp + h_line * 2;
        let y_result_success = y_exp2 + h_line * 3;
        let y_result_failure = y_result_success + h_line * 2;

        let x_comp = x_second_column + w_second_column / 4;
        let w_comp = w_second_column / 2;

        let mut selector =
            SelectBrowser::new(x_selector, y_selector, w_selector, h_selector, "Tests");

        let mut add = Button::new(x_add, y_butt, w_butt, h_butt, "@+");
        let mut ren = Button::new(x_ren, y_butt, w_butt, h_butt, None);
        let mut rem = Button::new(x_rem, y_butt, w_butt, h_butt, None);

        let name = Frame::new(x_second_column, y_name, w_second_column, h_line, "Name");
        let mut expression_left = TextEditor::new(
            x_second_column,
            y_exp,
            w_second_column,
            h_line,
            "Left side expression",
        );
        let mut expression_right = TextEditor::new(
            x_second_column,
            y_exp2,
            w_second_column,
            h_line,
            "Right side expression",
        );
        let mut comparison = fltk::menu::Choice::new(x_comp, y_comp, w_comp, h_line, None);
        Frame::new(
            x_second_column,
            y_result_success - font_size,
            w_second_column,
            h_line,
            "On Success",
        );
        let success = fltk::menu::Choice::new(
            x_second_column,
            y_result_success,
            w_second_column,
            h_line,
            None,
        );
        Frame::new(
            x_second_column,
            y_result_failure - font_size,
            w_second_column,
            h_line,
            "On Failure",
        );
        let failure = fltk::menu::Choice::new(
            x_second_column,
            y_result_failure,
            w_second_column,
            h_line,
            None,
        );
        group.end();

        let (sender, _r) = app::channel();

        add.emit(sender.clone(), emit!(Event::AddTest));
        ren.emit(sender.clone(), emit!(Event::RenameTest));
        rem.emit(sender.clone(), emit!(Event::RemoveTest));
        selector.set_callback({
            let mut selected = 0;
            move |x| {
                let new = x.value();
                if selected != new {
                    if selected > 0 {
                        if let Some(test) = x.text(selected) {
                            sender.send(emit!(Event::SaveTest(Some(test))));
                        }
                    }
                    if let Some(new) = x.selected_text() {
                        sender.send(emit!(Event::LoadTest(new)));
                    }
                    selected = new;
                }
            }
        });

        let mut gear = SvgImage::from_data(GEAR_ICON).unwrap();
        let mut bin = SvgImage::from_data(BIN_ICON).unwrap();
        gear.scale(font_size, font_size, false, true);
        bin.scale(font_size, font_size, false, true);
        ren.set_image(Some(gear));
        rem.set_image(Some(bin));

        expression_left.set_buffer(TextBuffer::default());
        expression_right.set_buffer(TextBuffer::default());
        comparison.add_choice(&Comparison::as_choice());
        comparison.set_value(0);

        variable_receiver!(expression_left);
        variable_receiver!(expression_right);

        Self {
            selector,
            name,
            expression_left,
            expression_right,
            comparison,
            success,
            failure,
        }
    }
    /// Loads provided test into UI
    fn load_ui(&mut self, test: &Test) {
        self.name.set_label(&test.name);
        if let Some(i) = find_item(&self.selector, &test.name) {
            self.selector.select(i);
            self.selector.do_callback();
        }
        self.expression_left
            .buffer()
            .as_mut()
            .unwrap()
            .set_text(&test.expression_l);
        self.expression_right
            .buffer()
            .as_mut()
            .unwrap()
            .set_text(&test.expression_r);
        self.comparison.set_value(test.comparison.to_index());
        let mut i = 0;
        while let Some(choice) = self.success.text(i) {
            if choice == test.success_result {
                self.success.set_value(i);
                break;
            }
            i += 1;
        }
        i = 0;
        while let Some(choice) = self.failure.text(i) {
            if choice == test.failure_result {
                self.failure.set_value(i);
            }
            i += 1;
        }
    }
    /// Fills the UI with data to edit the tests
    pub fn populate(
        &mut self,
        tests: &HashMap<String, Test>,
        results: &HashMap<String, StoryResult>,
    ) {
        let mut set = true;
        self.success.set_value(-1);
        self.failure.set_value(-1);
        self.success.clear();
        self.failure.clear();
        self.selector.clear();
        self.selector.do_callback();
        for result in results.iter() {
            self.success.add_choice(result.0);
            self.failure.add_choice(result.0);
        }
        for test in tests.iter() {
            self.selector.add(test.0);
            if set {
                self.load_ui(test.1);
                set = false;
            }
        }
        if set {
            // TODO hide the UI if there is nothing to edit
            self.expression_left.buffer().unwrap().set_text("");
            self.expression_right.buffer().unwrap().set_text("");
            self.name.set_label("");
            self.comparison.set_value(0);
            self.success.set_value(-1);
            self.success.clear();
            self.failure.set_value(-1);
            self.failure.clear();
        }
    }
    /// Returns text of currently selected item, or None if nothing is selected
    fn selected(&self) -> Option<String> {
        self.selector.selected_text()
    }
    /// Event response that saves the test into the page collection
    ///
    /// Test to save can be specified by name, or if it is None, currently selected test will be saved
    pub fn save(&self, tests: &mut HashMap<String, Test>, test: Option<String>) {
        let test = match test {
            Some(t) => t,
            None => match self.selected() {
                Some(t) => t,
                None => {
                    return;
                }
            },
        };
        if let Some(t) = tests.get_mut(&test) {
            if let Some(succ) = self.success.choice() {
                if let Some(fail) = self.failure.choice() {
                    t.comparison = Comparison::from(self.comparison.choice().unwrap());
                    t.expression_l = self.expression_left.buffer().unwrap().text();
                    t.expression_r = self.expression_right.buffer().unwrap().text();
                    t.success_result = succ;
                    t.failure_result = fail;
                    return;
                }
            }
            signal_error!("A Test needs to have all of its components before it can be saved");
        }
    }
    /// Event response that loads a test by name into UI
    pub fn load(&mut self, tests: &mut HashMap<String, Test>, test: String) {
        if let Some(test) = tests.get(&test) {
            self.load_ui(test);
        }
    }
    /// Event response renaming currently selected test
    ///
    /// It also updates the test in choices in the page
    pub fn rename(&mut self, page: &mut Page) {
        let selected = match self.selected() {
            Some(s) => s,
            None => {
                println!("Error: Could not rename a test. No test selected");
                return;
            }
        };
        let name = match ask_for_text(&format!("Insert new name for {} Test", &selected)) {
            Some(n) if n.len() > 0 => n,
            _ => return,
        };

        if let Some(mut test) = page.tests.remove(&selected) {
            // go through all the choices and update the test name
            page.choices
                .iter_mut()
                .filter(|x| x.test == selected)
                .for_each(|x| x.test = name.clone());

            let i = self.selector.value();
            self.selector.set_text(i, &name);
            test.name = name.clone();
            page.tests.insert(name, test);
        }
    }
    /// Event response that adds a new test to the page
    ///
    /// It will fail if there isn't at least 2 results present in the page
    pub fn add(&mut self, page: &mut Page) {
        if page.results.len() < 2 {
            signal_error!("You need to add at least 2 results before you can add a test");
            return;
        }
        let name = match ask_for_text("Insert name for new Test") {
            Some(n) if n.len() > 0 => n,
            _ => return,
        };
        if page.tests.contains_key(&name) {
            signal_error!("Cannot add {} because it already exists", name);
            return;
        }
        let test = Test {
            name: name.clone(),
            expression_l: "1d20".to_string(),
            expression_r: "10".to_string(),
            comparison: Comparison::Greater,
            ..Default::default()
        };
        self.selector.add(&name);
        self.load_ui(&test);
        page.tests.insert(name, test);
    }
    /// Event response that removes a selected test from the page
    ///
    /// It fails and shows error to an user if the test is used in a choice
    pub fn remove(&mut self, page: &mut Page) {
        let selected = match self.selected() {
            Some(s) => s,
            None => {
                println!("Error: Tried to remove selected test but found no selection");
                return;
            }
        };
        if page.tests.contains_key(&selected) == false {
            return;
        }
        if page.choices.iter().any(|x| x.test == selected) {
            signal_error!(
                "Cannot remove Test {} because it's used in one or more of Page's Choices",
                selected
            );
            return;
        }

        if ask_to_confirm(&format!(
            "Are you sure you want to delete {} Test?",
            &selected
        )) {
            page.tests.remove(&selected);
            self.populate(&page.tests, &page.results);
        }
    }
}
