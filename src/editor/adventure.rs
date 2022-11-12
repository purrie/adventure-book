use fltk::{
    app,
    draw::Rect,
    group::Group,
    prelude::*,
    text::{TextBuffer, TextEditor}, button::Button,
};

use crate::adventure::{Adventure, Record, Name};

use super::{variables::VariableEditor, help, highlight_color};

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

        let x_help = x_title + w_title - font_size * 2;
        let y_help = y_desc - font_size;
        let w_help = font_size;
        let h_help = w_help;

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
        let mut help = Button::new(x_help, y_help, w_help, h_help, "?");

        let records = VariableEditor::new(rec_area, true);
        let names = VariableEditor::new(nam_area, false);
        group.end();

        title.set_buffer(TextBuffer::default());
        description.set_buffer(TextBuffer::default());
        description.wrap_mode(fltk::text::WrapMode::AtBounds, 0);

        let (sender, _) = app::channel();
        help.emit(sender, help!("adventure-meta"));
        help.set_frame(fltk::enums::FrameType::RoundUpBox);
        help.set_color(highlight_color!());

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
    /// Adds a new name to the UI
    pub fn add_name(&mut self, name: &Name) {
        self.names.add_name(name, false);
        self.group.redraw();
    }
    /// Adds a new record to the UI
    pub fn add_record(&mut self, record: &Record) {
        self.records.add_record(record, false);
        self.group.redraw();
    }
    /// Clears either names or records editor UI
    pub fn clear_variables(&mut self, names: bool) {
        if names {
            self.names.clear();
        } else {
            self.records.clear();
        }
    }
    /// Loads adventure information into UI
    pub fn load(&mut self, adventure: &Adventure) {
        self.set_title(&adventure.title);
        self.set_description(&adventure.description);
        self.records.clear();
        for rec in adventure.records.iter() {
            self.records.add_record(rec.1, false);
        }
        self.names.clear();
        for nam in adventure.names.iter() {
            self.names.add_name(nam.1, false);
        }
        self.group.redraw();
    }
    /// Saves values into the adventure
    pub fn save(&self, adventure: &mut Adventure) {
        adventure.title = self.title.buffer().as_ref().unwrap().text();
        adventure.description = self.description.buffer().as_ref().unwrap().text();
        // saving only those because records and names are saved through their own controls
    }
}
