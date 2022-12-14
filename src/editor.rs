use std::collections::HashMap;

use fltk::{draw::Rect, group::Group, prelude::*};

use crate::{
    adventure::{is_keyword_valid, Adventure, Page},
    dialog::{ask_for_name, ask_for_record, ask_for_text, ask_to_confirm},
    file::{
        capture_pages, is_valid_file_name, read_page, remove_adventure, save_adventure, save_page,
        signal_error, open_help,
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
macro_rules! help {
    ($page:expr) => {
        crate::editor::emit!(crate::editor::Event::OpenHelp($page))
    };
}
pub(crate) use help;
macro_rules! highlight_color {
    () => {
        fltk::enums::Color::Yellow.inactive()
    };
}
pub(crate) use highlight_color;
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
    RenamePage,
    AddPage,
    RemovePage,
    SelectStartingPage(String),
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
    OpenHelp(&'static str),
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
    // creates a new editor in specified area
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
    /// Loads an adventure into editor
    ///
    /// The function may take some time as it loads in all the pages for editing
    pub fn load_adventure(&mut self, adventure: &Adventure, index: usize) {
        self.adventure = adventure.clone();
        self.adventure_index = Some(index);
        self.pages.clear();
        let pages = capture_pages(&self.adventure.path);
        self.file_list.populate_pages(&pages);
        self.adventure_editor.load(&self.adventure);
        for page in pages {
            match read_page(&adventure.path, &page) {
                Ok(p) => drop(self.pages.insert(page, p)),
                Err(e) => match e {
                    crate::file::FileError::ParsingFailure(_, p) => match p {
                        crate::adventure::ParsingError::IncomplatePage(p) => {
                            drop(self.pages.insert(page, p))
                        }
                        _ => signal_error!("Fatal Error while parsing page {:?}: {}", &page, p),
                    },
                    _ => signal_error!("Fatal Error while loading a page {}: {}", &page, e),
                },
            };
        }
        self.current_page = String::new();
        self.set_starting_page(self.adventure.start.clone());
    }
    /// Returns adventure and its index if it's existing adventure or None if the adventure has not been loaded yet
    pub fn get_adventure(&self) -> (Adventure, Option<usize>) {
        (self.adventure.clone(), self.adventure_index)
    }
    /// Processes editor events
    pub fn process(&mut self, ev: Event) {
        match ev {
            Event::Save                  => self.save_project(),
            Event::RenamePage            => self.rename_page(),
            Event::AddPage               => self.add_page(),
            Event::RemovePage            => self.remove_page(),
            Event::SelectStartingPage(p) => self.set_starting_page(p),
            Event::OpenMeta              => self.open_adventure(),
            Event::OpenPage(name)        => self.open_page(name),
            Event::AddRecord             => self.add_keyword(false),
            Event::AddName               => self.add_keyword(true),
            Event::EditRecord(old)       => self.rename_keyword(true, old),
            Event::EditName(old)         => self.rename_keyword(false, old),
            Event::RemoveRecord(name)    => self.remove_keyword(name, false),
            Event::RemoveName(name)      => self.remove_keyword(name, true),
            Event::SaveCondition(cond)   => self
                .page_editor
                .conditions
                .save(&mut page_mut!(self).conditions, cond),
            Event::LoadCondition(cond)   => self
                .page_editor
                .conditions
                .load(&page!(self).conditions, cond),
            Event::RenameCondition       => self.page_editor.conditions.rename(page_mut!(self)),
            Event::AddCondition          => self
                .page_editor
                .conditions
                .add(&mut page_mut!(self).conditions),
            Event::RemoveCondition       => self.page_editor.conditions.remove(page_mut!(self)),
            Event::SaveTest(test)        => self
                .page_editor
                .tests
                .save(&mut page_mut!(self).tests, test),
            Event::LoadTest(test)        => self
                .page_editor
                .tests
                .load(&mut page_mut!(self).tests, test),
            Event::RenameTest            => self.page_editor.tests.rename(page_mut!(self)),
            Event::AddTest               => self.page_editor.tests.add(&mut page_mut!(self)),
            Event::RemoveTest            => self.page_editor.tests.remove(&mut page_mut!(self)),
            Event::AddResult             => self.page_editor.results.add(&mut page_mut!(self).results, &self.current_page),
            Event::RenameResult          => self.page_editor.results.rename(page_mut!(self)),
            Event::RemoveResult          => self.page_editor.results.remove(page_mut!(self)),
            Event::SaveResult(res)       => {
                self.page_editor
                    .results
                    .save(&mut page_mut!(self).results, res, &self.adventure)
            }
            Event::LoadResult(res)       => self.page_editor.results.load(&page!(self).results, res),
            Event::SaveSideEffect(se)    => {
                self.page_editor
                    .results
                    .save_effect(&mut page_mut!(self).results, &self.adventure, se)
            }
            Event::LoadSideEffect(se)    => self
                .page_editor
                .results
                .load_effect(&page!(self).results, se),
            Event::AddSideEffectRecord   => self
                .page_editor
                .results
                .add_record(&mut page_mut!(self).results, &self.adventure.records),
            Event::AddSideEffectName     => self
                .page_editor
                .results
                .add_name(&mut page_mut!(self).results, &self.adventure.names),
            Event::RemoveSideEffect      => self
                .page_editor
                .results
                .remove_effect(&mut page_mut!(self).results),
            Event::AddChoice             => self
                .page_editor
                .choices
                .add_choice(&mut page_mut!(self).choices),
            Event::RemoveChoice          => self
                .page_editor
                .choices
                .remove_choice(&mut page_mut!(self).choices),
            Event::SaveChoice(c)         => self
                .page_editor
                .choices
                .save_choice(&mut page_mut!(self).choices, c),
            Event::LoadChoice(c)         => self
                .page_editor
                .choices
                .load_choice(&page!(self).choices, c),
            Event::RefreshResults        => {
                self.page_editor.choices.refresh_dropdowns(page!(self));
                self.page_editor
                    .tests
                    .populate(&page!(self).tests, &page!(self).results);
            }
            Event::ToggleRecords(f)      => self.page_editor.toggle_record_editor(f),
            Event::ToggleNames(f)        => self.page_editor.toggle_name_editor(f),
            Event::OpenHelp(help)        => open_help(help),
        }
    }
    /// Hides editor UI
    pub fn hide(&mut self) {
        self.group.hide();
    }
    /// Shows editor UI
    pub fn show(&mut self) {
        self.group.show();
        self.page_editor.hide();
        self.adventure_editor.show();
    }
    /// Saves the project into drive
    fn save_project(&mut self) {
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
        self.page_editor.load_page(page, &self.current_page, &self.adventure);

        // loading page elements
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
    /// Removes currently selected page
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
    /// Renames currently selected page
    ///
    /// It also updates all references to the page name
    fn rename_page(&mut self) {
        if let Some(name) =
            ask_for_text(&format!("Enter a new name for page {}", self.current_page))
        {
            let name = name.to_lowercase().replace(" ", "-");
            if is_valid_file_name(&name) == false {
                signal_error!("The file name {} is invalid", name);
                return;
            }
            if let Some(page) = self.pages.remove(&self.current_page) {
                self.pages
                    .iter_mut()
                    .map(|x| x.1.results.iter_mut().filter(|x| x.1.next_page == self.current_page))
                    .for_each(|x| x.for_each(|x| x.1.next_page = name.clone()));
                self.file_list.rename_selected(&name);
                self.pages.insert(name.clone(), page);
                self.current_page = name;
            }
        }
    }
    /// Adds a new empty page
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
    /// Marks provided page as starting page
    fn set_starting_page(&mut self, p: String) {
        if self.pages.contains_key(&p) {
            self.file_list.mark_line(&self.adventure.start, &p);
            self.adventure.start = p;
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
    /// Adds a keyword to the adventure through appropriate user dialog
    ///
    /// Provided flag determines if it is a name or record
    fn add_keyword(&mut self, is_name: bool) {
        if is_name {
            if let Some(nam) = ask_for_name(None) {
                if is_keyword_valid(&nam.keyword) {
                    if self.adventure.names.contains_key(&nam.keyword) {
                        signal_error!("The keyword {} is already present", nam.keyword);
                        return;
                    }
                    self.adventure_editor.add_name(&nam);
                    self.page_editor.add_name(&nam);
                    self.adventure.names.insert(nam.keyword.clone(), nam);
                } else {
                    signal_error!(
                        "The keyword {} is invalid, please use only letters and numbers",
                        nam.keyword
                    );
                }
            }
        } else {
            if let Some(rec) = ask_for_record(None) {
                if is_keyword_valid(&rec.name) {
                    if self.adventure.records.contains_key(&rec.name) {
                        signal_error!("The keyword {} is already present", rec.name);
                        return;
                    }
                    self.adventure_editor.add_record(&rec);
                    self.page_editor.add_record(&rec);
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
    /// Renames a keyword in the adventure
    fn rename_keyword(&mut self, is_record: bool, old: String) {
        // saving unsaved page edits
        if self.adventure_editor.active() == false {
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
        // renaming and updating the records
        if is_record {
            // grabbing the old record for default values
            let rec = match self.adventure.records.get(&old) {
                Some(r) => r,
                None => {
                    println!("Error: Tried to rename a record that doesn't exist: {}", old);
                    return;
                }
            };
            if let Some(new_rec) = ask_for_record(Some(rec)) {
                if is_keyword_valid(&new_rec.name) == false {
                    signal_error!("Keyword {} is invalid, use only regular letters", new_rec.name);
                    return;
                }
                // test for a different name only happens when the name was changed
                if old != new_rec.name {
                    if self.adventure.records.contains_key(&new_rec.name) || self.adventure.names.contains_key(&new_rec.name) {
                        signal_error!("Keyword {} is already used, choose a different name", new_rec.name);
                        return;
                    }
                }
                self.pages
                    .iter_mut()
                    .for_each(|x| x.1.rename_keyword(&old, &new_rec.name));
                self.adventure.update_record(&old, new_rec);
            }
        } else {
            // as above, grabbing the name for default values in dialog
            let nam = match self.adventure.names.get(&old) {
                Some(n) => n,
                None => {
                    println!("Error: Tried to rename a name that doesn't exist: {}", old);
                    return;
                }
            };
            if let Some(new_nam) = ask_for_name(Some(nam)) {
                if is_keyword_valid(&new_nam.keyword) == false {
                   signal_error!("Keyword {} is invalid, use only regular letters", new_nam.keyword);
                    return;
                }
                // test for a different name only happens when the name was changed
                if old != new_nam.keyword {
                    if self.adventure.records.contains_key(&new_nam.keyword) || self.adventure.names.contains_key(&new_nam.keyword) {
                        signal_error!("Keyword {} is already used, choose a different name", new_nam.keyword);
                        return;
                    }
                }
                self.pages
                    .iter_mut()
                    .for_each(|x| x.1.rename_keyword(&old, &new_nam.keyword));
                self.adventure.update_name(&old, new_nam);
            }
        }

        // updating the UI
        if self.adventure_editor.active() {
            self.adventure_editor.load(&self.adventure);
        } else {
            self.load_page();
        }
    }
    /// Removes a keyword from adventure
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
                    self.adventure_editor.add_name(&x.1);
                    self.page_editor.add_name(&x.1);
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
                    self.adventure_editor.add_record(&x.1);
                    self.page_editor.add_record(&x.1);
                });
            }
        }
    }
}
