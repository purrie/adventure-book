use fltk::{
    app,
    draw::Rect,
    group::Group,
    prelude::*,
    text::{TextBuffer, TextEditor},
};

use crate::{
    adventure::{is_keyword_valid, Adventure},
    dialog::{ask_for_name, ask_for_record},
    file::signal_error,
};

use super::{variables::VariableEditor, EditorWindow};

pub fn add_record(editor: &mut EditorWindow) {
    if let Some(rec) = ask_for_record() {
        if is_keyword_valid(&rec.name) {
            editor.adventure_editor.add_record(&rec.name, false);
            editor.adventure.records.insert(rec.name.clone(), rec);
            editor.group.redraw();
        } else {
            signal_error!(
                "The keyword {} is invalid, please use only letters and numbers",
                rec.name
            );
        }
    }
}
pub fn add_name(editor: &mut EditorWindow) {
    if let Some(nam) = ask_for_name() {
        if is_keyword_valid(&nam.keyword) {
            editor.adventure_editor.add_record(&nam.keyword, true);
            editor.adventure.names.insert(nam.keyword.clone(), nam);
            editor.group.redraw();
        } else {
            signal_error!(
                "The keyword {} is invalid, please use only letters and numbers",
                nam.keyword
            );
        }
    }
}

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
    pub fn active(&self) -> bool {
        self.group.visible()
    }
    pub fn hide(&mut self) {
        self.group.hide();
    }
    pub fn show(&mut self) {
        self.group.show();
    }
    fn set_title(&mut self, title: &str) {
        self.title.buffer().as_mut().unwrap().set_text(&title);
    }
    fn set_description(&mut self, description: &str) {
        self.description
            .buffer()
            .as_mut()
            .unwrap()
            .set_text(description);
    }
    fn add_record(&mut self, name: &String, is_name: bool) {
        if is_name {
            self.names.add_record(name, false);
        } else {
            self.records.add_record(name, false);
        }
    }
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
    }
    pub fn save(&self, adventure: &mut Adventure) {
        adventure.title = self.title.buffer().as_ref().unwrap().text();
        adventure.description = self.description.buffer().as_ref().unwrap().text();
        // saving only those because records and names are saved through their own controls
    }
}
