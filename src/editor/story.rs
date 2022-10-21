use fltk::{
    app,
    draw::Rect,
    group::{Group, Tabs},
    prelude::*,
    text::{TextBuffer, TextEditor},
};

use crate::{
    adventure::{Adventure, Page},
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
    pub fn new(area: Rect) -> Self {
        let group = Group::new(area.x, area.y, area.w, area.h, None);

        let font_size = app::font_size();

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
        let conditions = ConditionEditor::new(children);
        let tests = TestEditor::new(children);
        let results = ResultEditor::new(children);

        tabs.end();

        let records = VariableEditor::new(
            Rect::new(x_records, y_sidepanel, w_sidepanel, h_sidepanel),
            true,
        );
        let names = VariableEditor::new(
            Rect::new(x_names, y_sidepanel, w_sidepanel, h_sidepanel),
            false,
        );

        group.end();

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
    pub fn hide(&mut self) {
        self.group.hide();
    }
    pub fn show(&mut self) {
        self.group.redraw();
        self.group.show();
    }
    pub fn load_page(&mut self, page: &Page, adventure: &Adventure) {
        self.title.buffer().as_mut().unwrap().set_text(&page.title);
        self.story.buffer().as_mut().unwrap().set_text(&page.story);

        self.records.clear();
        for rec in adventure.records.iter() {
            self.records.add_record(rec.0, true);
        }
        self.names.clear();
        for nam in adventure.names.iter() {
            self.names.add_record(nam.0, true);
        }
    }
    pub fn save_page(&self, page: &mut Page, adventure: &Adventure) {
        page.title = self.title.buffer().as_ref().unwrap().text();
        page.story = self.story.buffer().as_ref().unwrap().text();
        self.choices.save_choice(&mut page.choices, None);
        self.conditions.save(&mut page.conditions, None);
        self.tests.save(&mut page.tests, None);
        self.results.save(&mut page.results, None, adventure);
    }
    pub fn toggle_record_editor(&mut self, on: bool) {
        if on {
            self.records.show();
        } else {
            self.records.hide();
        }
    }
    pub fn toggle_name_editor(&mut self, on: bool) {
        if on {
            self.names.show();
        } else {
            self.names.hide();
        }
    }
    pub fn add_variable(&mut self, name: &String, is_name: bool) {
        if is_name {
            self.names.add_record(name, true);
        } else {
            self.records.add_record(name, true);
        }
        self.group.redraw();
    }
    pub fn clear_variables(&mut self, names: bool) {
        if names {
            self.names.clear();
        } else {
            self.records.clear();
        }
    }
}
