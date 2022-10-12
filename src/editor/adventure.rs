use std::collections::HashMap;

use fltk::{
    app,
    draw::Rect,
    group::Group,
    prelude::*,
    text::{TextBuffer, TextEditor},
};

use crate::{
    adventure::{is_keyword_valid, Adventure, Page},
    dialog::{ask_for_name, ask_for_record, ask_to_confirm},
    file::signal_error,
};

use super::variables::VariableEditor;

/// Editor for customizing adventure metadata
///
/// Contains editors to set adventure's title and description,
/// as well as editors for adding records and names
pub struct AdventureEditor {
    group: Group,
    title: TextEditor,
    description: TextEditor,
    records: VariableEditor,
    names: VariableEditor,
}

impl AdventureEditor {
    /// Creates adventure editor UI, setting up callbacks and widgets in provided area
    pub fn new(area: Rect) -> Self {
        let font_size = app::font_size();

        let x_title = area.x;
        let y_title = area.y + font_size;
        let w_title = area.w;
        let h_title = font_size + 4;

        let x_desc = area.x;
        let y_desc = y_title + h_title + font_size;
        let w_desc = area.w;
        let h_desc = area.h / 2;

        let rec_area = Rect::new(
            area.x,
            area.y + y_desc + h_desc,
            area.w / 2,
            area.h - h_desc - h_title - font_size,
        );
        let nam_area = Rect::new(area.x + rec_area.w, rec_area.y, rec_area.w, rec_area.h);

        let group = Group::new(area.x, area.y, area.w, area.h, None);
        let mut title = TextEditor::new(x_title, y_title, w_title, h_title, "Title");
        let mut description = TextEditor::new(x_desc, y_desc, w_desc, h_desc, "Description");

        let records = VariableEditor::new(rec_area, true);
        let names = VariableEditor::new(nam_area, false);
        group.end();

        title.set_buffer(TextBuffer::default());
        description.set_buffer(TextBuffer::default());
        description.wrap_mode(fltk::text::WrapMode::AtBounds, 0);

        Self {
            group,
            title,
            description,
            records,
            names,
        }
    }
    /// Tests if the adventure UI is shown and visible
    pub fn active(&self) -> bool {
        self.group.visible()
    }
    /// Hides adventure editor UI
    pub fn hide(&mut self) {
        self.group.hide();
    }
    /// Shows the adventure editor UI
    pub fn show(&mut self) {
        self.group.show();
    }
    /// Sets title into title editor
    fn set_title(&mut self, title: &str) {
        self.title.buffer().as_mut().unwrap().set_text(&title);
    }
    /// Sets text into adventure description editor
    fn set_description(&mut self, description: &str) {
        self.description
            .buffer()
            .as_mut()
            .unwrap()
            .set_text(description);
    }
    /// Creates a variable UI for a new variable
    fn add_variable(&mut self, name: &String, is_name: bool) {
        if is_name {
            self.names.add_record(name, false);
        } else {
            self.records.add_record(name, false);
        }
    }
    /// Loads adventure information into UI
    pub fn load(&mut self, adventure: &Adventure) {
        self.set_title(&adventure.title);
        self.set_description(&adventure.description);
        self.records.clear();
        for rec in adventure.records.iter() {
            self.records.add_record(rec.0, false);
        }
        self.names.clear();
        for nam in adventure.names.iter() {
            self.names.add_record(nam.0, false);
        }
        self.group.redraw();
    }
    /// Saves values into the adventure
    pub fn save(&self, adventure: &mut Adventure) {
        adventure.title = self.title.buffer().as_ref().unwrap().text();
        adventure.description = self.description.buffer().as_ref().unwrap().text();
        // saving only those because records and names are saved through their own controls
    }
    /// Adds a name to the adventure
    ///
    /// It will open a window allowing user to enter values for it
    pub fn add_name(&mut self, adventure: &mut Adventure) {
        if let Some(nam) = ask_for_name() {
            if is_keyword_valid(&nam.keyword) {
                if adventure.names.contains_key(&nam.keyword) {
                    signal_error!("The keyword {} is already present", nam.keyword);
                    return;
                }
                self.add_variable(&nam.keyword, true);
                adventure.names.insert(nam.keyword.clone(), nam);
                self.group.redraw();
            } else {
                signal_error!(
                    "The keyword {} is invalid, please use only letters and numbers",
                    nam.keyword
                );
            }
        }
    }
    /// Adds a record to the adventure
    ///
    /// It opens a window allowing user to input values
    pub fn add_record(&mut self, adventure: &mut Adventure) {
        if let Some(rec) = ask_for_record() {
            if is_keyword_valid(&rec.name) {
                if adventure.records.contains_key(&rec.name) {
                    signal_error!("The keyword {} is already present", rec.name);
                    return;
                }
                self.add_variable(&rec.name, false);
                adventure.records.insert(rec.name.clone(), rec);
                self.group.redraw();
            } else {
                signal_error!(
                    "The keyword {} is invalid, please use only letters and numbers",
                    rec.name
                );
            }
        }
    }
    /// Removes record from adventure
    ///
    /// The function asks user for confirmation. It also fails with a warning when the record is in use.
    pub fn remove_record(
        &mut self,
        adventure: &mut Adventure,
        pages: &HashMap<String, Page>,
        name: String,
    ) {
        let keyword = match adventure.records.get(&name) {
            Some(k) => k,
            None => return,
        };
        for p in pages.iter() {
            if p.1.is_keyword_present(&keyword.name) {
                signal_error!(
                    "Cannot remove the record {} as it is used in at least one of pages",
                    name
                );
                return;
            }
        }
        if ask_to_confirm(&format!("Are you sure you want to remove {}?", name)) {
            adventure.records.remove(&name);
            self.records.clear();
            adventure
                .records
                .iter()
                .for_each(|x| self.records.add_record(&x.0, false));
            self.group.redraw();
        }
    }
    /// Removes name from adventure
    ///
    /// The function asks the user for confirmation. It also fails with a warning when the name is in use.
    pub fn remove_name(
        &mut self,
        adventure: &mut Adventure,
        pages: &HashMap<String, Page>,
        name: String,
    ) {
        let keyword = match adventure.names.get(&name) {
            Some(k) => k,
            None => return,
        };
        for p in pages.iter() {
            if p.1.is_keyword_present(&keyword.keyword) {
                signal_error!(
                    "Cannot remove the record {} as it is used in at least one of pages",
                    name
                );
                return;
            }
        }
        if ask_to_confirm(&format!("Are you sure you want to remove {}?", name)) {
            adventure.names.remove(&name);
            self.names.clear();
            adventure
                .names
                .iter()
                .for_each(|x| self.names.add_record(&x.0, false));
            self.group.redraw();
        }
    }
}
