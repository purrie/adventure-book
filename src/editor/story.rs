use fltk::{prelude::*, group::{Group, Tabs}, text::{TextEditor, TextBuffer}, draw::Rect, app};

use crate::{adventure::{Adventure, Page}, editor::variables::variable_receiver};

use super::{variables::VariableEditor, choice::ChoiceEditor, condition::ConditionEditor, test::TestEditor, result::ResultEditor, emit, Event};


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

        let x_editor = area.x;
        let w_editor = area.w;

        let y_title = area.y + font_size;
        let h_title = font_size + 4;
        let y_story = y_title + h_title + font_size;
        let h_story = area.h / 2;

        let x_sidepanel = x_editor + w_editor;
        let y_records = area.y;
        let w_sidepanel = area.w / 3;
        let h_sidepanel = area.h / 2;
        let y_names = y_records + h_sidepanel;

        let x_valuators = area.x;
        let y_valuators = y_story + h_story;
        let w_valuators = area.w;
        let h_valuators = area.h - h_story - h_title - font_size * 2;

        let mut title = TextEditor::new(x_editor, y_title, w_editor, h_title, "Title");
        let mut story = TextEditor::new(x_editor, y_story, w_editor, h_story, "Story Text");

        let records = VariableEditor::new(
            Rect::new(x_sidepanel, y_records, w_sidepanel, h_sidepanel),
            true,
        );
        let names = VariableEditor::new(
            Rect::new(x_sidepanel, y_names, w_sidepanel, h_sidepanel),
            false,
        );

        let mut tabs = Tabs::new(x_valuators, y_valuators, w_valuators, h_valuators, None);
        let children = Rect::from(tabs.client_area());

        let choices = ChoiceEditor::new(children);
        let conditions = ConditionEditor::new(children);
        let tests = TestEditor::new(children);
        let results = ResultEditor::new(children);

        tabs.end();

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
                    _ => unreachable!(),
                }
                if let Some(new_select) = x.value() {
                    let new_select = new_select.label();
                    match new_select.as_str() {
                        "Choices" => s.send(emit!(Event::RefreshChoices)),
                        "Conditions" => {}
                        "Tests" => {}
                        "Results" => {}
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
    pub fn save_page(&self, page: &mut Page) {
        page.title = self.title.buffer().as_ref().unwrap().text();
        page.story = self.story.buffer().as_ref().unwrap().text();
        // TODO save data from editors
    }
}
