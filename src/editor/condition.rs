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
    adventure::{Comparison, Condition, Page},
    dialog::{ask_for_text, ask_to_confirm},
    editor::{variables::variable_receiver, highlight_color},
    file::signal_error,
    widgets::find_item,
};

use super::{emit, help, Event};

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
        let x_help = x_mod + w_butt * 2;

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
        let mut help = Button::new(x_help, y_butt, w_butt, h_butt, "?");

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

        selector.set_callback({
            let sender = sender.clone();
            let mut selected = 0;
            move |x| {
                let new = x.value();
                if new != selected {
                    if selected > 0 {
                        if let Some(last) = x.text(selected) {
                            sender.send(emit!(Event::SaveCondition(Some(last))));
                        }
                    }
                    if let Some(new_s) = x.selected_text() {
                        sender.send(emit!(Event::LoadCondition(new_s)));
                    }
                    selected = new;
                }
            }
        });
        add.emit(sender.clone(), emit!(Event::AddCondition));
        ren.emit(sender.clone(), emit!(Event::RenameCondition));
        rem.emit(sender.clone(), emit!(Event::RemoveCondition));
        help.emit(sender, help!("condition"));
        help.set_frame(fltk::enums::FrameType::RoundUpBox);
        help.set_color(highlight_color!());

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
        }
    }
    /// Returns name of the loaded Condition, or empty string if there's no Condition loaded
    fn selected(&self) -> String {
        if let Some(t) = self.selector.selected_text() {
            return t;
        }
        String::new()
    }
    /// Loads a condition into active editor
    fn load_ui(&mut self, con: &Condition) {
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
        if let Some(n) = find_item(&self.selector, &con.name) {
            self.selector.select(n);
            self.selector.do_callback();
        } else {
            println!(
                "Warning: Could not find {} in condition editor selector, creating a new entry",
                con.name
            );
            self.selector.add(&con.name);
            self.selector.select(self.selector.size());
            self.selector.do_callback();
        }
        self.show_controls();
    }
    /// Fills the selector with a new set of Conditions
    ///
    /// The old entries will be removed from the selector.
    /// This function also clears the selected entry, and if conds isn't empty then it loads the first entry.
    pub fn populate_conditions(&mut self, conds: &HashMap<String, Condition>) {
        self.selector.clear();
        self.selector.select(0);
        self.selector.do_callback();
        let mut set = true;
        for con in conds.iter() {
            self.selector.add(con.0);
            if set {
                set = false;
                self.load_ui(con.1);
            }
        }
        // clearing the condition UI if nothing was loaded
        if set {
            self.hide_controls();
        }
    }
    /// Shows the part of editor responsible for customizing condition
    fn show_controls(&mut self) {
        self.comparison.show();
        self.expression_left.show();
        self.expression_right.show();
        self.name.show();
    }
    /// Hides the part of editor responsible for customizing condition
    fn hide_controls(&mut self) {
        self.comparison.hide();
        self.expression_left.hide();
        self.expression_right.hide();
        self.name.hide();
        self.expression_left.buffer().as_mut().unwrap().set_text("");
        self.expression_right
            .buffer()
            .as_mut()
            .unwrap()
            .set_text("");
        self.comparison.set_value(0);
    }
    /// Event response that renames entry in the selector to a new name
    ///
    /// # Errors
    ///
    /// When renaming an entry, make sure the new name matches the condition in the page, otherwise it will lead to errors.
    pub fn remove(&mut self, page: &mut Page) {
        let selected = self.selected();
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
                self.populate_conditions(&page.conditions);
            }
        }
    }
    /// Event response that adds a new condition into UI and collection
    pub fn add(&mut self, conditions: &mut HashMap<String, Condition>) {
        let name = match ask_for_text("Insert name for the new Condition") {
            Some(n) if n.len() > 0 => n,
            _ => return,
        };
        if let Some(_cond) = conditions.get(&name) {
            signal_error!("Cannot add {} because it already exists!", name);
            return;
        }
        let cond = Condition {
            name: name.clone(),
            expression_l: "0".to_string(),
            expression_r: "0".to_string(),
            ..Default::default()
        };
        self.selector.add(&cond.name);
        self.selector.select(self.selector.size());
        self.selector.do_callback();
        conditions.insert(name, cond);
    }
    /// Event rezponse that renames selected condition
    ///
    /// Conditions are page specific and this function will update any links to reflect the new condition name
    pub fn rename(&mut self, page: &mut Page) {
        let selected = self.selected();
        let name = match ask_for_text(&format!("Insert new name for {} Condition", &selected)) {
            Some(n) if n.len() > 0 => n,
            _ => return,
        };

        if let Some(mut cond) = page.conditions.remove(&selected) {
            // renaming the condition in choices
            for choice in page.choices.iter_mut() {
                if choice.condition == cond.name {
                    choice.condition = name.clone();
                }
            }
            let n = self.selector.value();
            self.selector.set_text(n, &name);
            self.name.set_label(&name);
            cond.name = name.clone();
            page.conditions.insert(name, cond);
        }
    }
    /// Event response that loads specified condition into UI
    ///
    /// # Errors
    /// If the condition isn't found, it will silently fail and spew warning into console
    pub fn load(&mut self, conditions: &HashMap<String, Condition>, cond: String) {
        if let Some(con) = conditions.get(&cond) {
            self.load_ui(con);
        } else {
            println!(
                "Warning! Attempted to load a condition that doesn't exist: {}",
                cond
            );
        }
    }
    /// Event response that saves a condition into the collection
    ///
    /// If name for the condition isn't specified, it will save currently selected condition
    /// The option exists to allow saving any condition, mostly when switching which condition is selected
    pub fn save(&self, conditions: &mut HashMap<String, Condition>, cond: Option<String>) {
        let cond = match cond {
            Some(s) => s,
            None => self.selected(),
        };
        if let Some(con) = conditions.get_mut(&cond) {
            con.comparison = Comparison::from(self.comparison.choice().unwrap());
            con.expression_l = self.expression_left.buffer().as_ref().unwrap().text();
            con.expression_r = self.expression_right.buffer().as_ref().unwrap().text();
        }
    }
}
