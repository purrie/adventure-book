use fltk::{
    app,
    draw::Rect,
    group::{Group, Tabs},
    prelude::*,
    text::{TextBuffer, TextEditor}, frame::Frame, enums::Align,
};

use crate::{
    adventure::{Adventure, Page, Name, Record},
    editor::variables::variable_receiver,
};

use super::{
    choice::ChoiceEditor, condition::ConditionEditor, emit, result::ResultEditor, test::TestEditor,
    variables::VariableEditor, Event,
};

/// Edits page's title and story text
///
/// Aside from text editors, it has quick insert buttons for inserting records and names into the text
pub struct StoryEditor {
    group: Group,
    page_name: Frame,
    title: TextEditor,
    story: TextEditor,
    records: VariableEditor,
    names: VariableEditor,
    pub choices: ChoiceEditor,
    pub conditions: ConditionEditor,
    pub tests: TestEditor,
    pub results: ResultEditor,
}

impl StoryEditor {
    /// Creates a new story editor in specified area
    pub fn new(area: Rect) -> Self {
        let group = Group::new(area.x, area.y, area.w, area.h, None);

        let font_size = app::font_size();
        let h_line = font_size + font_size / 2;

        let x_tabs = area.x;
        let y_tabs = area.y;
        let w_tabs = area.w;
        let h_tabs = area.h / 3 * 2;

        let y_sidepanel = y_tabs + h_tabs;
        let w_sidepanel = area.w / 2;
        let h_sidepanel = area.h - h_tabs;
        let x_records = area.x;
        let x_names = x_records + w_sidepanel;

        let mut tabs = Tabs::new(x_tabs, y_tabs, w_tabs, h_tabs, None);
        let children = Rect::from(tabs.client_area());

        let y_title = children.y + font_size;
        let h_title = font_size + 4;
        let y_story = y_title + h_title + font_size;
        let h_story = children.h - h_title - font_size * 2;

        let text_page = Group::new(children.x, children.y, children.w, children.h, "Page");
        let mut title = TextEditor::new(children.x, y_title, children.w, h_title, "Title");
        let mut story = TextEditor::new(children.x, y_story, children.w, h_story, "Story Text");
        text_page.end();

        let choices = ChoiceEditor::new(children);
        let results = ResultEditor::new(children);
        let tests = TestEditor::new(children);
        let conditions = ConditionEditor::new(children);

        tabs.end();

        let mut page_name = Frame::new(x_tabs, y_tabs, w_tabs - 10, h_line, None);

        let records = VariableEditor::new(
            Rect::new(x_records, y_sidepanel, w_sidepanel, h_sidepanel),
            true,
        );
        let names = VariableEditor::new(
            Rect::new(x_names, y_sidepanel, w_sidepanel, h_sidepanel),
            false,
        );

        group.end();

        page_name.set_align(Align::Inside.union(Align::Right));
        title.set_buffer(TextBuffer::default());
        story.set_buffer(TextBuffer::default());
        story.wrap_mode(fltk::text::WrapMode::AtBounds, 0);

        tabs.set_callback({
            let mut old_select = "Choices".to_string();
            move |x| {
                let (s, _r) = app::channel();
                // saving data from editors on tab switch
                match old_select.as_str() {
                    "Choices" => s.send(emit!(Event::SaveChoice(None))),
                    "Conditions" => s.send(emit!(Event::SaveCondition(None))),
                    "Tests" => s.send(emit!(Event::SaveTest(None))),
                    "Results" => {
                        s.send(emit!(Event::SaveResult(None)));
                    }
                    "Page" => {}
                    _ => unreachable!(),
                }
                if let Some(new_select) = x.value() {
                    let new_select = new_select.label();
                    match new_select.as_str() {
                        "Choices" => {
                            s.send(emit!(Event::RefreshResults));
                            s.send(emit!(Event::ToggleNames(true)));
                            s.send(emit!(Event::ToggleRecords(true)));
                        }
                        "Conditions" => {
                            s.send(emit!(Event::ToggleNames(false)));
                            s.send(emit!(Event::ToggleRecords(true)));
                        }
                        "Tests" => {
                            s.send(emit!(Event::RefreshResults));
                            s.send(emit!(Event::ToggleNames(false)));
                            s.send(emit!(Event::ToggleRecords(true)));
                        }
                        "Results" => {
                            s.send(emit!(Event::ToggleNames(true)));
                            s.send(emit!(Event::ToggleRecords(true)));
                        }
                        "Page" => {
                            s.send(emit!(Event::ToggleNames(true)));
                            s.send(emit!(Event::ToggleRecords(true)));
                        }
                        _ => unreachable!(),
                    }
                    old_select = new_select;
                }
            }
        });

        variable_receiver!(title);
        variable_receiver!(story);

        Self {
            group,
            page_name,
            title,
            story,
            records,
            names,
            choices,
            conditions,
            tests,
            results,
        }
    }
    /// Hides the editor
    pub fn hide(&mut self) {
        self.group.hide();
    }
    /// Shows and redraws the editor
    pub fn show(&mut self) {
        self.group.redraw();
        self.group.show();
    }
    /// Loads a page and list of records and names into editor
    pub fn load_page(&mut self, page: &Page, page_name: &String, adventure: &Adventure) {
        self.page_name.set_label(page_name);
        self.title.buffer().as_mut().unwrap().set_text(&page.title);
        self.story.buffer().as_mut().unwrap().set_text(&page.story);

        self.records.clear();
        for rec in adventure.records.iter() {
            self.records.add_record(rec.1, true);
        }
        self.names.clear();
        for nam in adventure.names.iter() {
            self.names.add_name(nam.1, true);
        }
    }
    /// Saves the data from the editor into the provided page
    pub fn save_page(&self, page: &mut Page, adventure: &Adventure) {
        page.title = self.title.buffer().as_ref().unwrap().text();
        page.story = self.story.buffer().as_ref().unwrap().text();
        self.choices.save_choice(&mut page.choices, None);
        self.conditions.save(&mut page.conditions, None);
        self.tests.save(&mut page.tests, None);
        self.results.save(&mut page.results, None, adventure);
    }
    /// Toggles the display of records UI
    pub fn toggle_record_editor(&mut self, on: bool) {
        if on {
            self.records.show();
        } else {
            self.records.hide();
        }
    }
    /// Toggles the display of names UI
    pub fn toggle_name_editor(&mut self, on: bool) {
        if on {
            self.names.show();
        } else {
            self.names.hide();
        }
    }
    /// Adds a new name to the UI
    pub fn add_name(&mut self, name: &Name) {
        self.names.add_name(name, true);
        self.group.redraw();
    }
    /// Adds a new record to the UI
    pub fn add_record(&mut self, record: &Record) {
        self.records.add_record(record, true);
        self.group.redraw();
    }
    /// Clears either records or names UI depending on the provided flag
    pub fn clear_variables(&mut self, names: bool) {
        if names {
            self.names.clear();
        } else {
            self.records.clear();
        }
    }
}
