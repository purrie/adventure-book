use std::collections::HashMap;

use fltk::{draw::Rect, group::Group, prelude::*};

use crate::{
    adventure::{is_keyword_valid, Adventure, Page},
    dialog::{ask_for_name, ask_for_record, ask_for_text, ask_to_confirm},
    file::{
        capture_pages, is_valid_file_name, read_page, remove_adventure, save_adventure, save_page,
        signal_error,
    },
};

mod adventure;
mod choice;
mod condition;
mod files;
mod result;
mod story;
mod test;
mod variables;

/// Creates a Game Event from Editor Event
/// Used for readibility mostly
macro_rules! emit {
    ($event:expr) => {
        crate::game::Event::Editor($event)
    };
}
pub(crate) use emit;
macro_rules! page {
    ($editor:expr) => {
        $editor.pages.get(&$editor.current_page).unwrap()
    };
}
macro_rules! page_mut {
    ($editor:expr) => {
        $editor.pages.get_mut(&$editor.current_page).unwrap()
    };
}

use self::{adventure::AdventureEditor, files::FileList, story::StoryEditor};

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Save,
    // TODO implement renaming pages
    AddPage,
    RemovePage,
    OpenMeta,
    OpenPage(String),
    AddRecord,
    AddName,
    EditRecord(String),
    EditName(String),
    RemoveRecord(String),
    RemoveName(String),
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
    AddChoice,
    RemoveChoice,
    SaveChoice(Option<usize>),
    LoadChoice(usize),
    RefreshResults,
    ToggleRecords(bool),
    ToggleNames(bool),
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
// TODO use cache folder and save pages and metadata in real time to allow crash recovery and editing continuation on accidental application closing
/// UI governing creation and edition of adventures and individual pages
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
                Err(e) => signal_error!("{}", e),
            };
        }
    }
    pub fn process(&mut self, ev: Event) {
        match ev {
            Event::Save                => self.save_project(),
            Event::AddPage             => self.add_page(),
            Event::RemovePage          => self.remove_page(),
            Event::OpenMeta            => self.open_adventure(),
            Event::OpenPage(name)      => self.open_page(name),
            Event::AddRecord           => self.add_keyword(false),
            Event::AddName             => self.add_keyword(true),
            Event::EditRecord(old)     => self.rename_keyword(old),
            Event::EditName(old)       => self.rename_keyword(old),
            Event::RemoveRecord(name)  => self.remove_keyword(name, false),
            Event::RemoveName(name)    => self.remove_keyword(name, true),
            Event::SaveCondition(cond) => self
                .page_editor
                .conditions
                .save(&mut page_mut!(self).conditions, cond),
            Event::LoadCondition(cond) => self
                .page_editor
                .conditions
                .load(&page!(self).conditions, cond),
            Event::RenameCondition     => self.page_editor.conditions.rename(page_mut!(self)),
            Event::AddCondition        => self
                .page_editor
                .conditions
                .add(&mut page_mut!(self).conditions),
            Event::RemoveCondition     => self.page_editor.conditions.remove(page_mut!(self)),
            Event::SaveTest(test)      => self
                .page_editor
                .tests
                .save(&mut page_mut!(self).tests, test),
            Event::LoadTest(test)      => self
                .page_editor
                .tests
                .load(&mut page_mut!(self).tests, test),
            Event::RenameTest          => self.page_editor.tests.rename(page_mut!(self)),
            Event::AddTest             => self.page_editor.tests.add(&mut page_mut!(self)),
            Event::RemoveTest          => self.page_editor.tests.remove(&mut page_mut!(self)),
            Event::AddResult           => self.page_editor.results.add(&mut page_mut!(self).results),
            Event::RenameResult        => self.page_editor.results.rename(page_mut!(self)),
            Event::RemoveResult        => self.page_editor.results.remove(page_mut!(self)),
            Event::SaveResult(res)     => {
                self.page_editor
                    .results
                    .save(&mut page_mut!(self).results, res, &self.adventure)
            }
            Event::LoadResult(res)     => self.page_editor.results.load(&page!(self).results, res),
            Event::SaveSideEffect(se)  => {
                self.page_editor
                    .results
                    .save_effect(page_mut!(self), &self.adventure, se)
            }
            Event::LoadSideEffect(se)  => self
                .page_editor
                .results
                .load_effect(&page!(self).results, se),
            Event::AddSideEffectRecord => self
                .page_editor
                .results
                .add_record(&mut page_mut!(self).results, &self.adventure.records),
            Event::AddSideEffectName   => self
                .page_editor
                .results
                .add_name(&mut page_mut!(self).results, &self.adventure.names),
            Event::RemoveSideEffect    => self
                .page_editor
                .results
                .remove_effect(&mut page_mut!(self).results),
            Event::AddChoice           => self
                .page_editor
                .choices
                .add_choice(&mut page_mut!(self).choices),
            Event::RemoveChoice        => self
                .page_editor
                .choices
                .remove_choice(&mut page_mut!(self).choices),
            Event::SaveChoice(c)       => self
                .page_editor
                .choices
                .save_choice(&mut page_mut!(self).choices, c),
            Event::LoadChoice(c)       => self
                .page_editor
                .choices
                .load_choice(&page!(self).choices, c),
            Event::RefreshResults      => {
                self.page_editor.choices.refresh_dropdowns(page!(self));
                self.page_editor.tests.populate(&page!(self).tests, &page!(self).results);
            }
            Event::ToggleRecords(f)    => self.page_editor.toggle_record_editor(f),
            Event::ToggleNames(f)      => self.page_editor.toggle_name_editor(f),
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
    fn save_project(&mut self) {
        // TODO strip unused page and adventure parts, warn user about it
        // alternative would be to not strip anything to allow resuming work, but instead implement a check-for-errors button

        // save any unsaved data
        if self.adventure_editor.active() {
            self.adventure_editor.save(&mut self.adventure);
        } else {
            self.page_editor.save_page(page_mut!(self), &self.adventure);
        }

        // serializing data
        let adv_ser = self.adventure.serialize_to_string();
        let pages_ser: HashMap<String, String> = self
            .pages
            .iter()
            .map(|x| (x.0.clone(), x.1.serialize_to_string()))
            .collect();

        // clearing the adventure's folder
        remove_adventure(&self.adventure.path);

        // Saving the serialized adventures into the folder
        save_adventure(&self.adventure.path, adv_ser);
        for page in pages_ser {
            save_page(&self.adventure.path, page.0, page.1);
        }
    }
    /// Opens page editor and loads page by filename into it
    fn open_page(&mut self, name: String) {
        if self.current_page == name {
            return;
        }
        if let Some(mut cur_page) = self.pages.get_mut(&self.current_page) {
            self.page_editor.save_page(&mut cur_page, &self.adventure);
        }
        self.adventure_editor.save(&mut self.adventure);
        self.adventure_editor.hide();

        self.current_page = name;
        self.load_page();
    }
    /// Loads current page into UI
    fn load_page(&mut self) {
        let page = page!(self);
        self.page_editor.load_page(page, &self.adventure);

        // loading page elements
        // TODO hide UI elements on subeditors when nothing is present/selected to avoid confusing users
        self.page_editor
            .conditions
            .populate_conditions(&page.conditions);
        self.page_editor.tests.populate(&page.tests, &page.results);
        self.page_editor
            .results
            .populate(&page.results, &self.pages);
        self.page_editor.choices.populate_dropdowns(&page);
        self.page_editor.choices.populate_choices(&page.choices);

        self.page_editor.show();
    }
    fn remove_page(&mut self) {
        if self.adventure_editor.active() {
            return;
        }
        if ask_to_confirm(&format!(
            "Are you sure you want to remove {} page?",
            self.current_page
        )) {
            self.pages.remove(&self.current_page);
            self.file_list.remove_line();
            self.open_adventure();
        }
    }
    fn add_page(&mut self) {
        if let Some(name) = ask_for_text("Enter name for the new page") {
            let file_name = name.to_lowercase().replace(" ", "-");
            if is_valid_file_name(&file_name) == false {
                signal_error!("The file name {} is invalid", file_name);
                return;
            }
            let page = Page {
                title: name,
                ..Default::default()
            };
            self.pages.insert(file_name.clone(), page);
            self.file_list.add_line(&file_name);
            self.open_page(file_name);
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
            self.page_editor.save_page(&mut cur_page, &self.adventure);
        }
        self.adventure_editor.load(&self.adventure);
        self.page_editor.hide();
        self.adventure_editor.show();
        self.current_page = String::new();
    }
    fn add_keyword(&mut self, is_name: bool) {
        if is_name {
            if let Some(nam) = ask_for_name() {
                if is_keyword_valid(&nam.keyword) {
                    if self.adventure.names.contains_key(&nam.keyword) {
                        signal_error!("The keyword {} is already present", nam.keyword);
                        return;
                    }
                    self.adventure_editor.add_variable(&nam.keyword, true);
                    self.page_editor.add_variable(&nam.keyword, true);
                    self.adventure.names.insert(nam.keyword.clone(), nam);
                } else {
                    signal_error!(
                        "The keyword {} is invalid, please use only letters and numbers",
                        nam.keyword
                    );
                }
            }
        } else {
            if let Some(rec) = ask_for_record() {
                if is_keyword_valid(&rec.name) {
                    if self.adventure.records.contains_key(&rec.name) {
                        signal_error!("The keyword {} is already present", rec.name);
                        return;
                    }
                    self.adventure_editor.add_variable(&rec.name, false);
                    self.page_editor.add_variable(&rec.name, false);
                    self.adventure.records.insert(rec.name.clone(), rec);
                    self.group.redraw();
                } else {
                    signal_error!(
                        "The keyword {} is invalid, please use only letters and numbers",
                        rec.name
                    );
                }
            }
        }
    }
    fn rename_keyword(&mut self, old: String) {
        if let Some(new_name) = ask_for_text(&format!("Renaming {} to?", &old)) {
            if is_keyword_valid(&new_name) == false {
                signal_error!("Keyword {} is invalid, use only regular letters", new_name);
                return;
            }
            if self.adventure_editor.active() == false {
                // saving unsaved page edits
                let mut page = page_mut!(self);
                self.page_editor.save_page(&mut page, &self.adventure);
                self.page_editor
                    .choices
                    .save_choice(&mut page.choices, None);
                self.page_editor
                    .tests
                    .save(&mut page_mut!(self).tests, None);
                self.page_editor
                    .conditions
                    .save(&mut page_mut!(self).conditions, None);
                self.page_editor
                    .results
                    .save(&mut page_mut!(self).results, None, &self.adventure);
            }
            self.pages
                .iter_mut()
                .for_each(|x| x.1.rename_keyword(&old, &new_name));
            self.adventure.rename_keyword(&old, &new_name);

            if self.adventure_editor.active() {
                self.adventure_editor.load(&self.adventure);
            } else {
                self.load_page();
            }
        }
    }
    fn remove_keyword(&mut self, name: String, is_name: bool) {
        if is_name {
            let keyword = match self.adventure.names.get(&name) {
                Some(k) => k,
                None => return,
            };
            for p in self.pages.iter() {
                if p.1.is_keyword_present(&keyword.keyword) {
                    signal_error!(
                        "Cannot remove the record {} as it is used in at least one of pages",
                        name
                    );
                    return;
                }
            }
            if ask_to_confirm(&format!("Are you sure you want to remove {}?", name)) {
                self.adventure.names.remove(&name);
                self.adventure_editor.clear_variables(true);
                self.page_editor.clear_variables(true);
                self.adventure.names.iter().for_each(|x| {
                    self.adventure_editor.add_variable(&x.1.keyword, true);
                    self.page_editor.add_variable(&x.1.keyword, true);
                });
            }
        } else {
            let keyword = match self.adventure.records.get(&name) {
                Some(k) => k,
                None => return,
            };
            for p in self.pages.iter() {
                if p.1.is_keyword_present(&keyword.name) {
                    signal_error!(
                        "Cannot remove the record {} as it is used in at least one of pages",
                        name
                    );
                    return;
                }
            }
            if ask_to_confirm(&format!("Are you sure you want to remove {}?", name)) {
                self.adventure.records.remove(&name);
                self.adventure_editor.clear_variables(false);
                self.page_editor.clear_variables(false);
                self.adventure.records.iter().for_each(|x| {
                    self.adventure_editor.add_variable(&x.1.name, false);
                    self.page_editor.add_variable(&x.1.name, false);
                });
            }
        }
    }
}
