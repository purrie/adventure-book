use std::{collections::HashMap, rc::Rc, cell::RefCell};

use fltk::{prelude::*, text::{TextBuffer, TextEditor}, app, browser::SelectBrowser, button::Button, frame::Frame, draw::Rect, group::Group, image::SvgImage};

use crate::{
    adventure::{Condition, Comparison},
    dialog::{ask_for_text, ask_to_confirm},
    file::signal_error, editor::variables::variable_receiver,
};

use super::{EditorWindow, emit, Event};

// TODO move functions to struct impl

pub fn save(editor: &mut EditorWindow, cond: Option<String>) {
    if let Some(page) = editor.pages.get_mut(&editor.current_page) {
        let cond = match cond {
            Some(s) => s,
            None => editor.page_editor.conditions.selected(),
        };
        if let Some(con) = page.conditions.get_mut(&cond) {
            editor.page_editor.conditions.save(con);
        }
    }
}
pub fn load(editor: &mut EditorWindow, cond: String) {
    if let Some(page) = editor.pages.get(&editor.current_page) {
        if let Some(con) = page.conditions.get(&cond) {
            editor.page_editor.conditions.load(con);
        }
    }
}
pub fn rename(editor: &mut EditorWindow) {
    if let Some(page) = editor.pages.get_mut(&editor.current_page) {
        let selected = editor.page_editor.conditions.selected();
        let name = match ask_for_text(&format!("Insert new name for {} Condition", &selected)) {
            Some(n) if n.len() > 0 => n,
            _ => return,
        };

        if let Some(cond) = page.conditions.get_mut(&selected) {
            // renaming the condition in choices
            for choice in page.choices.iter_mut() {
                if choice.condition == cond.name {
                    choice.condition = name.clone();
                }
            }
            editor.page_editor.conditions.rename(&selected, &name);
            cond.name = name;
        }
    }
}
pub fn add(editor: &mut EditorWindow) {
    let name = match ask_for_text("Insert name for the new Condition") {
        Some(n) => n,
        _ => return,
    };
    if let Some(page) = editor.pages.get_mut(&editor.current_page) {
        if let Some(_cond) = page.conditions.get(&name) {
            signal_error!("Cannot add {} because it already exists!", name);
            return;
        }
        let cond = Condition {
            name: name.clone(),
            expression_l: "0".to_string(),
            expression_r: "0".to_string(),
            ..Default::default()
        };
        editor.page_editor.conditions.add(&name);
        editor.page_editor.conditions.load(&cond);
        page.conditions.insert(name, cond);
    }
}
pub fn remove(editor: &mut EditorWindow) {
    if let Some(page) = editor.pages.get_mut(&editor.current_page) {
        let selected = editor.page_editor.conditions.selected();
        if page.conditions.contains_key(&selected) {
            for choice in page.choices.iter() {
                if choice.condition == selected {
                    signal_error!("Cannot remove Condition {} because it's used in one or more of the Page's Choices", selected);
                    return;
                }
            }
            if ask_to_confirm(&format!(
                "Are you sure you want to remove {} Condition?",
                &selected
            )) {
                page.conditions.remove(&selected);
                editor
                    .page_editor
                    .conditions
                    .populate_conditions(&page.conditions);
            }
        }
    }
}

/// Condition editor
///
/// Lists conditions by name
/// Customizes comparison and two expressions to evaluate
/// The story editor record inserters interactively insert tags here if the editor has focus
pub struct ConditionEditor {
    selector: SelectBrowser,
    name: Frame,
    expression_left: TextEditor,
    expression_right: TextEditor,
    comparison: fltk::menu::Choice,
    selected: Rc<RefCell<String>>,
}

impl ConditionEditor {
    /// Creates UI for editing Conditions
    pub fn new(area: Rect) -> Self {
        let group = Group::new(area.x, area.y, area.w, area.h, "Conditions");

        let font_size = app::font_size();

        let x_selector = area.x;
        let y_selector = area.y;
        let w_selector = area.w / 3;
        let h_selector = area.h - font_size;

        let y_butt = y_selector + h_selector;
        let w_butt = font_size;
        let h_butt = font_size;

        let x_add = x_selector;
        let x_mod = x_add + w_butt;
        let x_rem = x_selector + w_selector - w_butt;

        let marging_column = 20;
        let x_second_column = area.x + w_selector + marging_column;
        let w_second_column = area.w - w_selector - marging_column * 2;

        let h_line = font_size + font_size / 2;

        let y_name = y_selector + font_size;
        let y_exp = y_name + h_line * 2;
        let y_comp = y_exp + h_line * 2;
        let y_exp2 = y_comp + h_line * 2;

        let mut selector =
            SelectBrowser::new(x_selector, y_selector, w_selector, h_selector, "Conditions");
        let mut add = Button::new(x_add, y_butt, w_butt, h_butt, "@+");
        let mut ren = Button::new(x_mod, y_butt, w_butt, h_butt, None);
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
        let mut comparison = fltk::menu::Choice::new(
            x_second_column + w_second_column / 4,
            y_comp,
            w_second_column / 2,
            h_line,
            None,
        );
        group.end();

        let mut gear = SvgImage::from_data(crate::icons::GEAR_ICON).unwrap();
        let mut bin = SvgImage::from_data(crate::icons::BIN_ICON).unwrap();
        gear.scale(w_butt, h_butt, false, true);
        bin.scale(w_butt, h_butt, false, true);
        ren.set_image(Some(gear));
        rem.set_image(Some(bin));

        let (sender, _r) = app::channel();
        let selected = Rc::new(RefCell::new(String::new()));

        selector.set_callback({
            let sender = sender.clone();
            let selected = Rc::clone(&selected);
            move |x| {
                let mut s = selected.borrow_mut();
                if s.len() > 0 {
                    sender.send(emit!(Event::SaveCondition(Some(s.clone()))));
                }
                if let Some(new_s) = x.selected_text() {
                    *s = new_s;
                    sender.send(emit!(Event::LoadCondition(s.clone())));
                }
            }
        });
        add.emit(sender.clone(), emit!(Event::AddCondition));
        ren.emit(sender.clone(), emit!(Event::RenameCondition));
        rem.emit(sender, emit!(Event::RemoveCondition));

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
            selected,
        }
    }
    /// Returns name of the loaded Condition, or empty string if there's no Condition loaded
    fn selected(&self) -> String {
        self.selected.borrow().clone()
    }
    /// Loads a condition into active editor
    fn load(&mut self, con: &Condition) {
        self.name.set_label(&con.name);
        self.expression_left
            .buffer()
            .as_mut()
            .unwrap()
            .set_text(&con.expression_l);
        self.expression_right
            .buffer()
            .as_mut()
            .unwrap()
            .set_text(&con.expression_r);
        self.comparison.set_value(con.comparison.to_index());
        *self.selected.borrow_mut() = con.name.clone();
    }
    /// Clears up data from the editor
    fn clear_selection(&mut self) {
        self.name.set_label("Select a condition");
        self.expression_left.buffer().as_mut().unwrap().set_text("");
        self.expression_right
            .buffer()
            .as_mut()
            .unwrap()
            .set_text("");
        self.comparison.set_value(0);
        *self.selected.borrow_mut() = String::new();
    }
    /// Fills con with data from the editor
    ///
    /// It saves comparison and both expressions, name is not touched.
    fn save(&self, con: &mut Condition) {
        con.comparison = Comparison::from(self.comparison.choice().unwrap());
        con.expression_l = self.expression_left.buffer().as_ref().unwrap().text();
        con.expression_r = self.expression_right.buffer().as_ref().unwrap().text();
    }
    /// Fills the selector with a new set of Conditions
    ///
    /// The old entries will be removed from the selector.
    /// This function also clears the selected entry, and if conds isn't empty then it loads the first entry.
    pub fn populate_conditions(&mut self, conds: &HashMap<String, Condition>) {
        self.selector.clear();
        let mut set = true;
        for con in conds.iter() {
            if set {
                set = false;
                self.load(con.1);
            }
            self.selector.add(con.0);
        }
        if set {
            self.clear_selection();
        }
    }
    /// Renames entry in the selector to a new name
    ///
    /// # Errors
    ///
    /// When renaming an entry, make sure the new name matches the condition in the page, otherwise it will lead to errors.
    fn rename(&mut self, old: &str, new: &str) {
        let mut n = 1;
        while let Some(t) = self.selector.text(n) {
            if t == old {
                if *self.selected.borrow() == old {
                    self.name.set_label(new);
                }
                self.selector.set_text(n, new);
                return;
            }
            n += 1;
        }
    }
    /// Adds a new line entry to the selector
    ///
    /// # Errors
    ///
    /// When using this function, ensure the condition with the name actually exists in the page, otherwise it will lead to errors
    fn add(&mut self, line: &str) {
        self.selector.add(line);
    }
}
