use std::{cell::RefCell, collections::HashMap, rc::Rc};

use fltk::{
    app,
    browser::SelectBrowser,
    button::Button,
    draw::Rect,
    frame::Frame,
    group::{Group, Scroll, Tabs},
    image::SvgImage,
    prelude::*,
    text::{TextBuffer, TextEditor},
};

use crate::{
    adventure::{is_keyword_valid, Adventure, Choice, Comparison, Condition, Name, Page, Record},
    dialog::{ask_for_name, ask_for_record, ask_for_text, ask_to_confirm},
    file::{capture_pages, read_page, signal_error},
};

/// Creates a Game Event from Editor Event
/// Used for readibility mostly
macro_rules! emit {
    ($event:expr) => {
        crate::game::Event::Editor($event)
    };
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
    /// Saves the currently displayed choice in editor into memory, event carries the index of the choice to be used for saving
    SaveChoice(usize),
    /// Requests a choice to be loaded into choice editor
    LoadChoice(usize),
    /// Saves the currently displayed condition into editor memory
    SaveCondition(Option<String>),
    /// Loads condition into editor
    LoadCondition(String),
    RenameCondition,
    AddCondition,
    RemoveCondition,

    /// This event is used to select data block for sub editors in pages
    SelectInSubEditor(i32),
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
            Event::Save => todo!(),
            Event::AddPage => todo!(),
            Event::RemovePage => todo!(),
            Event::OpenMeta => self.open_adventure(),
            Event::OpenPage(name) => self.open_page(name),
            Event::AddRecord => {
                if let Some(rec) = ask_for_record() {
                    if is_keyword_valid(&rec.name) {
                        self.adventure_editor.add_record(&rec.name, false);
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
            Event::AddName => {
                if let Some(nam) = ask_for_name() {
                    if is_keyword_valid(&nam.keyword) {
                        self.adventure_editor.add_record(&nam.keyword, true);
                        self.adventure.names.insert(nam.keyword.clone(), nam);
                        self.group.redraw();
                    } else {
                        signal_error!(
                            "The keyword {} is invalid, please use only letters and numbers",
                            nam.keyword
                        );
                    }
                }
            }
            Event::InsertRecord(_) => todo!(),
            Event::InsertName(_) => todo!(),
            Event::EditRecord(_) => todo!(),
            Event::EditName(_) => todo!(),
            Event::RemoveRecord(_) => todo!(),
            Event::RemoveName(_) => todo!(),
            Event::SaveChoice(_) => todo!(),
            Event::LoadChoice(_) => todo!(),
            Event::SaveCondition(cond) => {
                if let Some(page) = self.pages.get_mut(&self.current_page) {
                    let cond = match cond {
                        Some(s) => s,
                        None => self.page_editor.conditions.selected(),
                    };
                    if let Some(con) = page.conditions.get_mut(&cond) {
                        self.page_editor.conditions.save(con);
                    }
                }
            }
            Event::LoadCondition(cond) => {
                if let Some(page) = self.pages.get(&self.current_page) {
                    if let Some(con) = page.conditions.get(&cond) {
                        self.page_editor.conditions.load(&con);
                    }
                }
            }
            Event::RenameCondition => {
                if let Some(page) = self.pages.get_mut(&self.current_page) {
                    let selected = self.page_editor.conditions.selected();
                    let name;
                    if let Some(n) =
                        ask_for_text(&format!("Insert new name for {} Condition", &selected))
                    {
                        if n.len() == 0 {
                            return;
                        }
                        name = n;
                    } else {
                        return;
                    }

                    if let Some(cond) = page.conditions.get_mut(&selected) {
                        // renaming the condition in choices
                        for choice in page.choices.iter_mut() {
                            if choice.condition == cond.name {
                                choice.condition = name.clone();
                            }
                        }
                        self.page_editor.conditions.rename(&selected, &name);
                        cond.name = name;
                    }
                }
            }
            Event::AddCondition => {
                let name;
                if let Some(n) = ask_for_text("Insert name for the new Condition") {
                    name = n;
                } else {
                    return;
                }
                if name.len() == 0 {
                    return;
                }
                if let Some(page) = self.pages.get_mut(&self.current_page) {
                    if let Some(_cond) = page.conditions.get(&name) {
                        signal_error!("Cannot add {} because it already exists!", name);
                        return;
                    }
                    let cond = Condition {
                        name: name.clone(),
                        expression_l: "0".to_string(),
                        expression_r: "0".to_string(),
                        ..Default::default()
                    };
                    self.page_editor.conditions.add(&name);
                    self.page_editor.conditions.load(&cond);
                    page.conditions.insert(name, cond);
                }
            }
            Event::RemoveCondition => {
                if let Some(page) = self.pages.get_mut(&self.current_page) {
                    let selected = self.page_editor.conditions.selected();
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
                            self.page_editor
                                .conditions
                                .populate_conditions(&page.conditions);
                        }
                    }
                }
            }
            Event::SelectInSubEditor(_) => todo!(),
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
        }
    }
    /// Opens adventure metadata editor UI
    fn open_adventure(&mut self) {
        // no need to do anything if metadata already is shown
        if self.adventure_editor.group.visible() {
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
/// Displays the list of files in adventure
///
/// It displays both adventure metadata and each page
/// It also has buttons for adding and removing pages,
/// or deleting the whole adventure, adding a new one or loading existing
struct FileList {
    page_list: SelectBrowser,
}

impl FileList {
    fn new(area: Rect) -> Self {
        let group = Group::new(area.x, area.y, area.w, area.h, None);
        let mut butt_bac = Button::new(area.x + 5, area.y, area.w / 2 - 5, 20, "Return");
        let mut butt_sav =
            Button::new(area.x + area.w / 2 + 5, area.y, area.w / 2 - 10, 20, "Save");
        let mut butt_add = Button::new(area.x + 5, area.y + 25, area.w / 2 - 5, 20, "Add Page");
        let mut butt_rem = Button::new(
            area.x + area.w / 2 + 5,
            area.y + 25,
            area.w / 2 - 10,
            20,
            "Remove Page",
        );
        let mut adventure_meta = Button::new(
            area.x + 5,
            area.y + 50,
            area.w - 10,
            20,
            "Adventure Metadata",
        );
        Frame::new(area.x, area.y + 75, area.w, 20, "Pages");
        let mut page_list = SelectBrowser::new(area.x, area.y + 95, area.w, area.h - 95, None);
        group.end();

        let (s, _r) = app::channel();

        butt_bac.emit(s.clone(), crate::game::Event::DisplayMainMenu);
        butt_sav.emit(s.clone(), emit!(Event::Save));
        butt_add.emit(s.clone(), emit!(Event::AddPage));
        butt_rem.emit(s.clone(), emit!(Event::RemovePage));
        adventure_meta.emit(s.clone(), emit!(Event::OpenMeta));
        page_list.set_callback(move |x| {
            if let Some(text) = x.selected_text() {
                s.send(emit!(Event::OpenPage(text)));
            }
        });

        Self { page_list }
    }
    /// Fills the selection widget with page names
    fn populate_pages(&mut self, pages: &Vec<String>) {
        self.page_list.clear();
        for text in pages {
            self.page_list.add(&text);
        }
    }
}

/// Editor for customizing adventure metadata
///
/// Contains editors to set adventure's title and description,
/// as well as editors for adding records and names
struct AdventureEditor {
    group: Group,
    title: TextEditor,
    description: TextEditor,
    records: VariableEditor,
    names: VariableEditor,
}

impl AdventureEditor {
    fn new(area: Rect) -> Self {
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
    fn hide(&mut self) {
        self.group.hide();
    }
    fn show(&mut self) {
        self.group.show();
    }
    fn update_records(&mut self, records: Vec<String>) {
        unimplemented!()
    }
    fn update_names(&mut self, names: Vec<String>) {
        unimplemented!()
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
    fn load(&mut self, adventure: &Adventure) {
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
    fn save(&self, adventure: &mut Adventure) {
        adventure.title = self.title.buffer().as_ref().unwrap().text();
        adventure.description = self.description.buffer().as_ref().unwrap().text();
        // saving only those because records and names are saved through their own controls
    }
}

/// Editor widget for editing records and names
struct VariableEditor {
    scroll: Scroll,
    children: usize,
    record: bool,
}

impl VariableEditor {
    fn new(area: Rect, is_record: bool) -> Self {
        let mut butt_add = Button::new(area.x, area.y, area.w / 2, 20, None);
        let scroll = Scroll::new(area.x, area.y + 20, area.w, area.h - 20, None);
        scroll.end();

        let (s, _r) = app::channel();

        if is_record {
            butt_add.set_label("Add Record");
            butt_add.emit(s, emit!(Event::AddRecord));
        } else {
            butt_add.set_label("Add Name");
            butt_add.emit(s, emit!(Event::AddName));
        }

        Self {
            scroll,
            children: 0,
            record: is_record,
        }
    }
    fn clear(&mut self) {
        self.scroll.clear();
        self.children = 0;
    }
    fn add_record(&mut self, variable: &String, inserter: bool) {
        let child_count = self.children;

        let mut x = self.scroll.x();
        let y = self.scroll.y() + 20 * child_count as i32;
        let mut w = self.scroll.w();
        let h = 20;

        let mut frame = Frame::new(x, y, w, h, None);
        frame.set_frame(fltk::enums::FrameType::EngravedFrame);

        let (sender, _) = app::channel();

        if inserter {
            let mut butt_insert = Button::new(x, y, 20, h, "@<-");
            let ev;
            if self.record {
                ev = emit!(Event::InsertRecord(variable.clone()));
            } else {
                ev = emit!(Event::InsertName(variable.clone()));
            }

            butt_insert.emit(sender.clone(), ev);

            self.scroll.add(&butt_insert);

            x += 20;
            w -= 20;
        }
        let edit;
        let delete;
        if self.record {
            edit = emit!(Event::EditRecord(child_count));
            delete = emit!(Event::RemoveRecord(child_count));
        } else {
            edit = emit!(Event::EditName(child_count));
            delete = emit!(Event::RemoveName(child_count));
        }

        let bin_icon = SvgImage::from_data(crate::icons::BIN_ICON).unwrap();
        let mut gear_icon = SvgImage::from_data(crate::icons::GEAR_ICON).unwrap();
        gear_icon.scale(15, 15, true, false);

        let mut butt_edit = Button::new(x, y, 20, h, None);
        butt_edit.set_image(Some(gear_icon));
        butt_edit.emit(sender.clone(), edit);

        x += 20;
        w -= 20;

        let mut butt_delete = Button::new(x, y, 20, h, None);
        butt_delete.set_image(Some(bin_icon));
        butt_delete.emit(sender, delete);

        x += 20;
        w -= 20;

        let mut label = Frame::new(x, y, w, h, None);
        label.set_label(variable);

        self.scroll.add(&frame);
        self.scroll.add(&butt_edit);
        self.scroll.add(&butt_delete);
        self.scroll.add(&label);

        self.children += 1;
    }
}

/// Edits page's title and story text
///
/// Aside from text editors, it has quick insert buttons for inserting records and names into the text
struct StoryEditor {
    group: Group,
    title: TextEditor,
    story: TextEditor,
    records: VariableEditor,
    names: VariableEditor,
    choices: ChoiceEditor,
    conditions: ConditionEditor,
    tests: TestEditor,
    results: ResultEditor,
}

impl StoryEditor {
    fn new(area: Rect) -> Self {
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
                match old_select.as_str() {
                    "Choices" => {}
                    "Conditions" => s.send(emit!(Event::SaveCondition(None))),
                    "Tests" => {}
                    "Results" => {}
                    _ => unreachable!(),
                }
                if let Some(new_select) = x.value() {
                    let new_select = new_select.label();
                    match new_select.as_str() {
                        "Choices" => {}
                        "Conditions" => {}
                        "Tests" => {}
                        "Results" => {}
                        _ => unreachable!(),
                    }
                    old_select = new_select;
                }
            }
        });

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
    fn hide(&mut self) {
        self.group.hide();
    }
    fn show(&mut self) {
        self.group.show();
    }
    fn load_page(&mut self, page: &Page, adventure: &Adventure) {
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
    fn save_page(&self, page: &mut Page) {
        page.title = self.title.buffer().as_ref().unwrap().text();
        page.story = self.story.buffer().as_ref().unwrap().text();
    }
}

/// Editor for customizing choices for a page
///
/// Displays a list of choices for the page
/// It has a text editor for the choice text, and drop downs for choosing condition, test and result for each choice
struct ChoiceEditor {
    selector: SelectBrowser,
    text: TextEditor,
    condition: fltk::menu::Choice,
    test: Rc<RefCell<fltk::menu::Choice>>,
    result: Rc<RefCell<fltk::menu::Choice>>,
    last_selected: Rc<RefCell<i32>>,
}

impl ChoiceEditor {
    fn new(area: Rect) -> Self {
        use fltk::menu::Choice;
        let font_size = app::font_size();

        let group = Group::new(area.x, area.y, area.w, area.h, "Choices");

        let x_selector = area.x;
        let y_selector = area.y;
        let w_selector = area.w / 3;
        let h_selector = area.h - font_size;

        let margin_menu = 20;
        let x_menu = area.x + w_selector + margin_menu;
        let w_menu = area.w - w_selector - margin_menu * 2;
        let h_menu = font_size + 2;
        let y_menu_condition = area.y + font_size;
        let y_menu_test = y_menu_condition + h_menu * 2;
        let y_menu_result = y_menu_test + h_menu * 2;

        let x_text = x_menu;
        let y_text = y_menu_result + h_menu * 2;
        let w_text = w_menu;
        let h_text = h_menu;

        let mut selector = SelectBrowser::new(
            x_selector,
            y_selector,
            w_selector,
            h_selector,
            "Choices in this page",
        );
        let mut text = TextEditor::new(x_text, y_text, w_text, h_text, "Choice Text");
        Frame::new(
            x_menu,
            y_menu_condition - font_size,
            w_menu,
            h_menu,
            "Condition",
        );
        let condition = Choice::new(x_menu, y_menu_condition, w_menu, h_menu, None);
        Frame::new(x_menu, y_menu_test - font_size, w_menu, h_menu, "Test");
        let test = Choice::new(x_menu, y_menu_test, w_menu, h_menu, None);
        Frame::new(x_menu, y_menu_result - font_size, w_menu, h_menu, "Result");
        let result = Choice::new(x_menu, y_menu_result, w_menu, h_menu, None);
        group.end();

        text.set_buffer(TextBuffer::default());
        let last_selected = Rc::new(RefCell::new(0));

        let (sender, _r) = app::channel();

        selector.set_callback({
            let last_selected = Rc::clone(&last_selected);
            let sender = sender.clone();
            move |x| {
                let mut index = last_selected.borrow_mut();
                let new_index = x.value();
                if *index == new_index {
                    return;
                }
                sender.send(emit!(Event::SaveChoice(*index as usize)));
                sender.send(emit!(Event::LoadChoice(new_index as usize)));
                *index = new_index;
            }
        });

        let test = Rc::new(RefCell::new(test));
        let result = Rc::new(RefCell::new(result));

        test.borrow_mut().set_callback({
            let result = Rc::clone(&result);
            move |x| {
                if x.value() >= 0 {
                    let mut r = result.borrow_mut();
                    if r.value() >= 0 {
                        r.set_value(-1);
                    }
                }
            }
        });
        result.borrow_mut().set_callback({
            let test = Rc::clone(&test);
            move |x| {
                if x.value() >= 0 {
                    let mut r = test.borrow_mut();
                    if r.value() >= 0 {
                        r.set_value(-1);
                    }
                }
            }
        });
        Self {
            selector,
            text,
            test,
            condition,
            result,
            last_selected,
        }
    }
    fn save_choice(&self, choice: &mut Choice) {
        choice.text = self.text.buffer().as_ref().unwrap().text();
        choice.condition = match self.condition.choice() {
            Some(text) => text,
            None => String::new(),
        };
        choice.test = match self.test.borrow().choice() {
            Some(text) => text,
            None => String::new(),
        };
        choice.result = match self.result.borrow().choice() {
            Some(text) => text,
            None => String::new(),
        };
    }
    fn load_choice(&mut self, choice: &Choice) {
        self.text.buffer().as_mut().unwrap().set_text(&choice.text);
        if choice.condition.len() != 0 {
            let index = self.condition.find_index(&choice.condition);
            self.condition.set_value(index);
        } else {
            self.condition.set_value(-1);
        }
        if choice.test.len() != 0 {
            let index = self.test.borrow().find_index(&choice.test);
            self.test.borrow_mut().set_value(index);
        } else {
            self.test.borrow_mut().set_value(-1);
        }
        if choice.result.len() != 0 {
            let index = self.result.borrow().find_index(&choice.result);
            self.result.borrow_mut().set_value(index);
        } else {
            self.result.borrow_mut().set_value(-1);
        }
    }
    fn load_choices(&mut self, choices: Vec<Choice>) {}
}
/// Condition editor
///
/// Lists conditions by name
/// Customizes comparison and two expressions to evaluate
/// The story editor record inserters interactively insert tags here if the editor has focus
struct ConditionEditor {
    selector: SelectBrowser,
    name: Frame,
    expression_left: TextEditor,
    expression_right: TextEditor,
    comparison: fltk::menu::Choice,
    selected: Rc<RefCell<String>>,
}

impl ConditionEditor {
    /// Creates UI for editing Conditions
    fn new(area: Rect) -> Self {
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

        let marging_column = 20;
        let x_second_column = area.x + w_selector + marging_column;
        let w_second_column = area.w - w_selector - marging_column * 2;

        let h_line = font_size + 2;

        let y_name = y_selector + font_size;
        let y_exp = y_name + h_line * 2;
        let y_comp = y_exp + h_line * 2;
        let y_exp2 = y_comp + h_line * 2;

        let mut selector =
            SelectBrowser::new(x_selector, y_selector, w_selector, h_selector, "Conditions");
        let mut add = Button::new(x_add, y_butt, w_butt, h_butt, "@+");
        let mut ren = Button::new(x_mod, y_butt, w_butt, h_butt, None);
        let mut rem = Button::new(x_rem, y_butt, w_butt, h_butt, None);

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
        let selected = Rc::new(RefCell::new(String::new()));

        selector.set_callback({
            let sender = sender.clone();
            let selected = Rc::clone(&selected);
            move |x| {
                let mut s = selected.borrow_mut();
                if s.len() > 0 {
                    sender.send(emit!(Event::SaveCondition(Some(s.clone()))));
                }
                if let Some(new_s) = x.selected_text() {
                    *s = new_s;
                    sender.send(emit!(Event::LoadCondition(s.clone())));
                }
            }
        });
        add.emit(sender.clone(), emit!(Event::AddCondition));
        ren.emit(sender.clone(), emit!(Event::RenameCondition));
        rem.emit(sender, emit!(Event::RemoveCondition));

        expression_left.set_buffer(TextBuffer::default());
        expression_right.set_buffer(TextBuffer::default());
        comparison.add_choice(&Comparison::as_choice());
        comparison.set_value(0);

        Self {
            selector,
            name,
            expression_left,
            expression_right,
            comparison,
            selected,
        }
    }
    /// Returns name of the loaded Condition, or empty string if there's no Condition loaded
    fn selected(&self) -> String {
        self.selected.borrow().clone()
    }
    /// Loads a condition into active editor
    fn load(&mut self, con: &Condition) {
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
        *self.selected.borrow_mut() = con.name.clone();
    }
    /// Clears up data from the editor
    fn clear_selection(&mut self) {
        self.name.set_label("Select a condition");
        self.expression_left.buffer().as_mut().unwrap().set_text("");
        self.expression_right
            .buffer()
            .as_mut()
            .unwrap()
            .set_text("");
        self.comparison.set_value(0);
        *self.selected.borrow_mut() = String::new();
    }
    /// Fills con with data from the editor
    ///
    /// It saves comparison and both expressions, name is not touched.
    fn save(&self, con: &mut Condition) {
        con.comparison = Comparison::from(self.comparison.choice().unwrap());
        con.expression_l = self.expression_left.buffer().as_ref().unwrap().text();
        con.expression_r = self.expression_right.buffer().as_ref().unwrap().text();
    }
    /// Fills the selector with a new set of Conditions
    ///
    /// The old entries will be removed from the selector.
    /// This function also clears the selected entry, and if conds isn't empty then it loads the first entry.
    fn populate_conditions(&mut self, conds: &HashMap<String, Condition>) {
        self.selector.clear();
        let mut set = true;
        for con in conds.iter() {
            if set {
                set = false;
                self.load(con.1);
            }
            self.selector.add(con.0);
        }
        if set {
            self.clear_selection();
        }
    }
    /// Renames entry in the selector to a new name
    ///
    /// # Errors
    ///
    /// When renaming an entry, make sure the new name matches the condition in the page, otherwise it will lead to errors.
    fn rename(&mut self, old: &str, new: &str) {
        let mut n = 1;
        while let Some(t) = self.selector.text(n) {
            if t == old {
                if *self.selected.borrow() == old {
                    self.name.set_label(new);
                }
                self.selector.set_text(n, new);
                return;
            }
            n += 1;
        }
    }
    /// Adds a new line entry to the selector
    ///
    /// # Errors
    ///
    /// When using this function, ensure the condition with the name actually exists in the page, otherwise it will lead to errors
    fn add(&mut self, line: &str) {
        self.selector.add(line);
    }
}
/// Widgets for editing tests
///
/// Lists tests in page by name
/// Has widgets to customize two expressions and their comparison
/// It provides drop downs to fill success and failure results of the test
struct TestEditor {
    selector: SelectBrowser,
    name: TextEditor,
    expression_left: TextEditor,
    expression_right: TextEditor,
    comparison: fltk::menu::Choice,
    success: fltk::menu::Choice,
    failure: fltk::menu::Choice,
}

impl TestEditor {
    fn new(area: Rect) -> Self {
        let group = Group::new(area.x, area.y, area.w, area.h, "Tests");

        let font_size = app::font_size();

        let x_selector = area.x;
        let y_selector = area.y;
        let w_selector = area.w / 3;
        let h_selector = area.h - font_size;

        let column_margin = 20;
        let x_second_column = x_selector + w_selector + column_margin;
        let w_second_column = area.w - w_selector - column_margin * 2;
        let line_height = font_size + 2;

        let y_name = y_selector + font_size;
        let y_exp = y_name + line_height * 2;
        let y_comp = y_exp + line_height * 2;
        let y_exp2 = y_comp + line_height * 2;
        let y_result_success = y_exp2 + line_height * 3;
        let y_result_failure = y_result_success + line_height * 2;

        let x_comp = x_second_column + w_second_column / 4;
        let w_comp = w_second_column / 2;

        let mut selector =
            SelectBrowser::new(x_selector, y_selector, w_selector, h_selector, "Tests");
        let mut name = TextEditor::new(
            x_second_column,
            y_name,
            w_second_column,
            line_height,
            "Name",
        );
        let mut expression_left = TextEditor::new(
            x_second_column,
            y_exp,
            w_second_column,
            line_height,
            "Left side expression",
        );
        let mut expression_right = TextEditor::new(
            x_second_column,
            y_exp2,
            w_second_column,
            line_height,
            "Right side expression",
        );
        let mut comparison = fltk::menu::Choice::new(x_comp, y_comp, w_comp, line_height, None);
        Frame::new(
            x_second_column,
            y_result_success - font_size,
            w_second_column,
            line_height,
            "On Success",
        );
        let success = fltk::menu::Choice::new(
            x_second_column,
            y_result_success,
            w_second_column,
            line_height,
            None,
        );
        Frame::new(
            x_second_column,
            y_result_failure - font_size,
            w_second_column,
            line_height,
            "On Failure",
        );
        let failure = fltk::menu::Choice::new(
            x_second_column,
            y_result_failure,
            w_second_column,
            line_height,
            None,
        );
        group.end();

        let (sender, _r) = app::channel();

        selector.set_callback({
            let sender = sender.clone();
            move |x| {
                if x.value() > 0 {
                    sender.send(x.value() - 1);
                }
            }
        });

        name.set_buffer(TextBuffer::default());
        expression_left.set_buffer(TextBuffer::default());
        expression_right.set_buffer(TextBuffer::default());
        comparison.add_choice(">|>=|<|<=|=|!=");
        comparison.set_value(0);

        Self {
            selector,
            name,
            expression_left,
            expression_right,
            comparison,
            success,
            failure,
        }
    }
}
/// Widgets for customizing results of the page
///
/// Lists available results for the page
/// It will give a drop down for choosing the next page
/// It will give a growing field for adding changes to records or names
struct ResultEditor {
    selector: SelectBrowser,
    name: TextEditor,
    next_page: fltk::menu::Choice,
    effects: Vec<(fltk::menu::Choice, TextEditor)>,
}

impl ResultEditor {
    fn new(area: Rect) -> Self {
        let group = Group::new(area.x, area.y, area.w, area.h, "Results");

        let font_size = app::font_size();

        let x_column_1 = area.x;
        let w_column_1 = area.w / 3;

        let y_results = area.y;
        let h_result = area.h / 2 - font_size;

        let margin = 20;
        let x_column_2 = x_column_1 + w_column_1 + margin;
        let w_column_2 = area.w - w_column_1 - margin * 2;
        let h_line = font_size + 2;

        let y_mods = y_results + h_result + font_size;
        let h_mods = area.h - h_result - font_size * 2;

        let y_name = y_results + font_size;
        let y_page = y_name + h_line * 2;

        let margin2 = 5;
        let x_column_3 = x_column_2 + margin2;
        let w_column_3 = w_column_2 / 2 - margin2 * 2;
        let x_column_4 = x_column_3 + w_column_3 + margin2 * 2;

        let y_butt = y_mods + h_line;
        let y_exp = y_butt + h_line * 2;

        let mut select_result =
            SelectBrowser::new(x_column_1, y_results, w_column_1, h_result, "Results");
        let mut select_mod =
            SelectBrowser::new(x_column_1, y_mods, w_column_1, h_mods, "Modifications");

        let mut name = TextEditor::new(x_column_2, y_name, w_column_2, h_line, "Name");
        Frame::new(
            x_column_2,
            y_page - font_size,
            w_column_2,
            h_line,
            "Next Page",
        );
        let next_page = fltk::menu::Choice::new(x_column_2, y_page, w_column_2, h_line, None);

        let butt_rec = Button::new(x_column_3, y_butt, w_column_3, h_line, "Add Record");
        let butt_nam = Button::new(x_column_4, y_butt, w_column_3, h_line, "Add Name");
        let expression = TextEditor::new(x_column_2, y_exp, w_column_2, h_line, "Value expression");

        group.end();

        let (sender, _r) = app::channel();

        select_result.set_callback({
            let sender = sender.clone();
            move |x| {
                if x.value() > 0 {
                    sender.send(x.value() - 1);
                }
            }
        });

        name.set_buffer(TextBuffer::default());

        Self {
            selector: select_result,
            name,
            next_page,
            effects: Vec::new(),
        }
    }
}
