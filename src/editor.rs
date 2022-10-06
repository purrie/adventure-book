use std::collections::HashMap;

use fltk::{
    draw::Rect,
    group::Group,
    prelude::*,
};

use crate::{
    adventure::{Adventure, Page},
    file::{capture_pages, read_page, signal_error},
};

mod adventure;
mod choice;
mod condition;
mod result;
mod test;
mod files;
mod variables;
mod story;

/// Creates a Game Event from Editor Event
/// Used for readibility mostly
macro_rules! emit {
    ($event:expr) => {
        crate::game::Event::Editor($event)
    };
}
pub(crate) use emit;

use self::{
    adventure::AdventureEditor, files::FileList, story::StoryEditor,
};

#[derive(Clone)]
pub enum Event {
    Save,

    AddPage,
    RemovePage,
    OpenMeta,
    OpenPage(String),
    AddRecord,
    AddName,
    InsertRecord(String),
    InsertName(String),
    EditRecord(usize),
    EditName(usize),
    RemoveRecord(usize),
    RemoveName(usize),
    SaveCondition(Option<String>),
    LoadCondition(String),
    RenameCondition,
    AddCondition,
    RemoveCondition,
    SaveTest(Option<String>),
    LoadTest(String),
    AddTest,
    RenameTest,
    RemoveTest,
    AddResult,
    RenameResult,
    RemoveResult,
    SaveResult(Option<String>),
    LoadResult(String),
    SaveSideEffect(Option<String>),
    LoadSideEffect(String),
    AddSideEffectRecord,
    AddSideEffectName,
    RemoveSideEffect,
}

/// Responsible for managing all the editor widgets, saving adventures and opening existing ones for editing
pub struct EditorWindow {
    /// Root UI group
    group: Group,
    /// Collection of global editor controls
    file_list: FileList,
    /// Collection of UI controls for editing adventure metadata
    adventure_editor: AdventureEditor,
    /// Collection of UI controls for editing individual page contents
    page_editor: StoryEditor,

    /// Index of the edited adventure within the main adventure list, None for a new unsaved adventure
    adventure_index: Option<usize>,
    /// Adventure that is loaded for editing
    adventure: Adventure,

    current_page: String,
    /// Map of file name keys and pages on those file names
    pages: HashMap<String, Page>,
}

impl EditorWindow {
    pub fn new(area: Rect) -> Self {
        let x_file = area.x;
        let y_file = area.y;
        let w_file = area.w / 4;
        let h_file = area.h;

        let x_editor = x_file + w_file + 5;
        let y_editor = area.y;
        let w_editor = area.w - w_file - 5;
        let h_editor = area.h;

        let group = Group::new(area.x, area.y, area.w, area.h, None);
        let file_list = FileList::new(Rect::from((x_file, y_file, w_file, h_file)));
        let adventure_editor =
            AdventureEditor::new(Rect::from((x_editor, y_editor, w_editor, h_editor)));
        let mut page_editor =
            StoryEditor::new(Rect::from((x_editor, y_editor, w_editor, h_editor)));
        group.end();

        page_editor.hide();

        Self {
            group,
            file_list,
            adventure_editor,
            page_editor,
            adventure: Adventure::default(),
            pages: HashMap::new(),
            adventure_index: None,
            current_page: String::new(),
        }
    }
    pub fn load_adventure(&mut self, adventure: &Adventure, index: usize) {
        self.adventure = adventure.clone();
        self.adventure_index = Some(index);
        let pages = capture_pages(&self.adventure.path);
        self.file_list.populate_pages(&pages);
        self.adventure_editor.load(&self.adventure);
        for page in pages {
            match read_page(&adventure.path, &page) {
                Ok(p) => drop(self.pages.insert(page, p)),
                Err(e) => signal_error!(&e),
            };
        }
    }
    pub fn process(&mut self, ev: Event) {
        match ev {
            Event::Save                => {
                // TODO strip unused page and adventure parts, warn user about it
            }
            Event::AddPage             => todo!(),
            Event::RemovePage          => todo!(),
            Event::OpenMeta            => self.open_adventure(),
            Event::OpenPage(name)      => self.open_page(name),
            Event::AddRecord           => adventure::add_record(self),
            Event::AddName             => adventure::add_name(self),
            Event::InsertRecord(_)     => todo!(),
            Event::InsertName(_)       => todo!(),
            Event::EditRecord(_)       => todo!(),
            Event::EditName(_)         => todo!(),
            Event::RemoveRecord(_)     => todo!(),
            Event::RemoveName(_)       => todo!(),
            Event::SaveCondition(cond) => condition::save(self, cond),
            Event::LoadCondition(cond) => condition::load(self, cond),
            Event::RenameCondition     => condition::rename(self),
            Event::AddCondition        => condition::add(self),
            Event::RemoveCondition     => condition::remove(self),
            Event::SaveTest(test)      => test::save(self, test),
            Event::LoadTest(test)      => test::load(self, test),
            Event::RenameTest          => test::rename(self),
            Event::AddTest             => test::add(self),
            Event::RemoveTest          => test::remove(self),
            Event::AddResult           => result::add(self),
            Event::RenameResult        => result::rename(self),
            Event::RemoveResult        => result::remove(self),
            Event::SaveResult(res)     => result::save(self, res),
            Event::LoadResult(res)     => result::load(self, res),
            Event::SaveSideEffect(se)  => result::save_effect(self, se),
            Event::LoadSideEffect(se)  => result::load_effect(self, se),
            Event::AddSideEffectRecord => result::add_record(self),
            Event::AddSideEffectName   => result::add_name(self),
            Event::RemoveSideEffect    => result::remove_effect(self),
        }
    }
    pub fn hide(&mut self) {
        self.group.hide();
    }
    pub fn show(&mut self) {
        self.group.show();
        self.page_editor.hide();
        self.adventure_editor.show();
    }
    /// Opens page editor and loads page by filename into it
    fn open_page(&mut self, name: String) {
        // skipping loading the same page twice
        if name == self.current_page {
            return;
        }
        if let Some(mut cur_page) = self.pages.get_mut(&self.current_page) {
            self.page_editor.save_page(&mut cur_page);
        }
        if let Some(page) = self.pages.get(&name) {
            self.adventure_editor.save(&mut self.adventure);
            self.adventure_editor.hide();
            self.page_editor.load_page(page, &self.adventure);
            self.page_editor.show();
            self.current_page = name;

            // loading page elements
            self.page_editor
                .conditions
                .populate_conditions(&page.conditions);
            self.page_editor.tests.populate(&page.tests, &page.results);
            self.page_editor
                .results
                .populate(&page.results, &self.pages);
        }
    }
    /// Opens adventure metadata editor UI
    fn open_adventure(&mut self) {
        // no need to do anything if metadata already is shown
        if self.adventure_editor.active() {
            return;
        }
        // saving open page
        if let Some(mut cur_page) = self.pages.get_mut(&self.current_page) {
            self.page_editor.save_page(&mut cur_page);
        }
        self.adventure_editor.load(&self.adventure);
        self.page_editor.hide();
        self.adventure_editor.show();
        self.current_page = String::new();
    }
}


