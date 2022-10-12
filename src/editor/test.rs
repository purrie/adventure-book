use std::{cell::RefCell, rc::Rc, collections::HashMap};

use fltk::{prelude::*, browser::SelectBrowser, frame::Frame, text::{TextEditor, TextBuffer}, draw::Rect, group::Group, app, button::Button, image::SvgImage};

use crate::{dialog::{ask_to_confirm, ask_for_text}, file::signal_error, adventure::{Test, Comparison, StoryResult}, icons::{GEAR_ICON, BIN_ICON}, editor::variables::variable_receiver};

use super::{EditorWindow, emit, Event};


pub fn save(editor: &mut EditorWindow, test: Option<String>) {
    if let Some(page) = editor.pages.get_mut(&editor.current_page) {
        let test = match test {
            Some(t) => t,
            None => editor.page_editor.tests.selected(),
        };
        if let Some(t) = page.tests.get_mut(&test) {
            editor.page_editor.tests.save(t);
        }
    }
}
pub fn load(editor: &mut EditorWindow, test: String) {
    if let Some(page) = editor.pages.get_mut(&editor.current_page) {
        if let Some(test) = page.tests.get(&test) {
            editor.page_editor.tests.load(test);
        }
    }
}
pub fn rename(editor: &mut EditorWindow) {
    if let Some(page) = editor.pages.get_mut(&editor.current_page) {
        let selected = editor.page_editor.tests.selected();
        let name = match ask_for_text(&format!("Insert new name for {} Test", &selected)) {
            Some(n) if n.len() > 0 => n,
            _ => return,
        };

        if let Some(test) = page.tests.get_mut(&selected) {
            for choice in page.choices.iter_mut() {
                if choice.test == test.name {
                    choice.test = name.clone();
                }
            }
            editor.page_editor.tests.rename(name.clone());
            test.name = name;
        }
    }
}
pub fn add(editor: &mut EditorWindow) {
    if let Some(page) = editor.pages.get_mut(&editor.current_page) {
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
            expression_l: "0".to_string(),
            expression_r: "0".to_string(),
            ..Default::default()
        };
        editor.page_editor.tests.add(&name);
        editor.page_editor.tests.load(&test);
        page.tests.insert(name, test);
    }
}
pub fn remove(editor: &mut EditorWindow) {
    if let Some(page) = editor.pages.get_mut(&editor.current_page) {
        let selected = editor.page_editor.tests.selected();
        if page.tests.contains_key(&selected) == false {
            return;
        }
        for choice in page.choices.iter() {
            if choice.test == selected {
                signal_error!(
                    "Cannot remove Test {} because it's used in one or more of Page's Choices",
                    selected
                );
                return;
            }
        }
        if ask_to_confirm(&format!(
            "Are you sure you want to delete {} Test?",
            &selected
        )) {
            page.tests.remove(&selected);
            editor.page_editor.tests.populate(&page.tests, &page.results);
        }
    }
}

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
    selected: Rc<RefCell<String>>,
}

impl TestEditor {
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
        let selected = Rc::new(RefCell::new(String::new()));

        add.emit(sender.clone(), emit!(Event::AddTest));
        ren.emit(sender.clone(), emit!(Event::RenameTest));
        rem.emit(sender.clone(), emit!(Event::RemoveTest));
        selector.set_callback({
            let selected = Rc::clone(&selected);
            move |x| {
                let mut s = selected.borrow_mut();
                if s.len() > 0 {
                    sender.send(emit!(Event::SaveTest(Some(s.clone()))));
                }
                if let Some(new) = x.selected_text() {
                    if *s == new {
                        return;
                    }
                    *s = new;
                    sender.send(emit!(Event::LoadTest(s.clone())));
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
            selected,
        }
    }
    fn load(&mut self, test: &Test) {
        self.name.set_label(&test.name);
        let mut s = self.selected.borrow_mut();
        *s = test.name.clone();
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
    fn save(&self, test: &mut Test) {
        if let Some(succ) = self.success.choice() {
            if let Some(fail) = self.failure.choice() {
                test.comparison = Comparison::from(self.comparison.choice().unwrap());
                test.expression_l = self.expression_left.buffer().unwrap().text();
                test.expression_r = self.expression_right.buffer().unwrap().text();
                test.success_result = succ;
                test.failure_result = fail;
                return;
            }
        }
        signal_error!("A Test needs to have all of its components before it can be saved");
    }
    fn add(&mut self, entry: &str) {
        self.selector.add(entry);
    }
    fn rename(&mut self, new: String) {
        let mut i = 1;
        let mut old = self.selected.borrow_mut();
        while let Some(entry) = self.selector.text(i) {
            if entry == *old {
                self.selector.set_text(i, &new);
                self.name.set_label(&new);
                *old = new;
                return;
            }
            i += 1;
        }
    }
    pub fn populate(&mut self, tests: &HashMap<String, Test>, results: &HashMap<String, StoryResult>) {
        let mut set = true;
        self.success.set_value(-1);
        self.failure.set_value(-1);
        self.success.clear();
        self.failure.clear();
        self.selector.clear();
        for result in results.iter() {
            self.success.add_choice(result.0);
            self.failure.add_choice(result.0);
        }
        for test in tests.iter() {
            self.selector.add(test.0);
            if set {
                self.load(test.1);
                set = false;
            }
        }
        if set {
            self.clear_selection();
        }
    }
    fn clear_selection(&mut self) {
        self.expression_left.buffer().unwrap().set_text("");
        self.expression_right.buffer().unwrap().set_text("");
        self.name.set_label("");
        self.comparison.set_value(0);
        self.success.set_value(-1);
        self.success.clear();
        self.failure.set_value(-1);
        self.failure.clear();
        *self.selected.borrow_mut() = String::new();
    }
    fn selected(&self) -> String {
        let s = self.selected.borrow();
        s.clone()
    }
}
