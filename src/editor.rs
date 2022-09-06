use std::{cell::RefCell, rc::Rc};

use fltk::{
    app,
    browser::SelectBrowser,
    button::Button,
    draw::Rect,
    frame::Frame,
    group::{Group, Scroll, Tabs},
    prelude::*,
    text::{TextBuffer, TextEditor}, image::SvgImage,
};

use crate::{
    adventure::{Adventure, Page},
    file::capture_pages,
    game::Event,
};

/// Responsible for managing all the editor widgets, saving adventures and opening existing ones for editing
pub struct EditorWindow {
    group: Group,
    file_list: FileList,
    adventure_editor: AdventureEditor,
    page_editor: StoryEditor,

    adventure: Adventure,
    page: Page,
    adventure_index: Option<usize>,
}

impl EditorWindow {
    pub fn new(area: Rect) -> Self {
        let x_file = area.x;
        let y_file = area.y;
        let w_file = area.w / 3;
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
            page: Page::default(),
            adventure_index: None,
        }
    }
    pub fn load_adventure(&mut self, adventure: &Adventure, index: usize) {
        self.adventure = adventure.clone();
        self.adventure_index = Some(index);
        let pages = capture_pages(&self.adventure.path);
        self.file_list.populate_pages(pages);
        self.adventure_editor.load(&self.adventure);
    }
    pub fn hide(&mut self) {
        self.group.hide();
    }
    pub fn show(&mut self) {
        self.group.show();
        self.page_editor.hide();
        self.adventure_editor.show();
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

        butt_bac.emit(s.clone(), Event::DisplayMainMenu);
        butt_sav.emit(s.clone(), Event::EditorSave);
        butt_add.emit(s.clone(), Event::EditorAddPage);
        butt_rem.emit(s.clone(), Event::EditorRemovePage);
        adventure_meta.emit(s.clone(), Event::EditorOpenMeta);
        page_list.set_callback(move |x| {
            if let Some(text) = x.selected_text() {
                s.send(Event::EditorOpenPage(text));
            }
        });

        Self { page_list }
    }
    /// Fills the selection widget with page names
    fn populate_pages(&mut self, pages: Vec<String>) {
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

        Self {
            group,
            title,
            description,
            records,
            names,
        }
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
    fn hide(&mut self) {
        self.group.hide();
    }
    fn show(&mut self) {
        self.group.show();
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
        unimplemented!()
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
            butt_add.emit(s, Event::EditorAddRecord);
        } else {
            butt_add.set_label("Add Name");
            butt_add.emit(s, Event::EditorAddName);
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

        let (sender, _) = app::channel();

        if inserter {
            let mut butt_insert = Button::new(x, y, 20, h, "@<-");
            let ev;
            if self.record {
                ev = Event::EditorInsertRecord(variable.clone());
            } else {
                ev = Event::EditorInsertName(variable.clone());
            }

            butt_insert.emit(sender.clone(), ev);

            self.scroll.add(&butt_insert);

            x += 20;
            w -= 20;
        }
        let edit;
        let delete;
        if self.record {
            edit = Event::EditorEditRecord(child_count);
            delete = Event::EditorRemoveRecord(child_count);
        } else {
            edit = Event::EditorEditName(child_count);
            delete = Event::EditorRemoveName(child_count);
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

        let x_editor = area.x;
        let y_title = area.y;
        let w_editor = area.w / 2 * 2;
        let h_title = 20;

        let y_story = y_title + h_title;
        let h_story = area.h / 2 - h_title;

        let x_sidepanel = x_editor + w_editor;
        let y_records = area.y;
        let w_sidepanel = area.w / 3;
        let h_sidepanel = area.h / 2;
        let y_names = y_records + h_sidepanel;

        let x_valuators = area.x;
        let y_valuators = y_story + h_story;
        let w_valuators = area.w;
        let h_valuators = area.h - h_story + h_title;

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

        let tabs = Tabs::new(x_valuators, y_valuators, w_valuators, h_valuators, None);
        let children = Rect::from(tabs.client_area());

        let choices = ChoiceEditor::new(children);
        let conditions = ConditionEditor::new(children);
        let tests = TestEditor::new(children);
        let results = ResultEditor::new(children);

        tabs.end();

        group.end();

        title.set_buffer(TextBuffer::default());
        story.set_buffer(TextBuffer::default());

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
}

impl ChoiceEditor {
    fn new(area: Rect) -> Self {
        use fltk::menu::Choice;

        let group = Group::new(area.x, area.y, area.w, area.h, None);

        let x_selector = area.x;
        let y_selector = area.y;
        let w_selector = area.w / 3;
        let h_selector = area.h - 50;

        let x_text = area.x;
        let y_text = area.y + area.h - 30;
        let w_text = area.w;
        let h_text = 25;

        let x_menu = area.x + w_selector + 10;
        let w_menu = area.w - w_selector - 20;
        let h_menu = 20;
        let y_menu_condition = area.y;
        let y_menu_test = y_menu_condition + h_menu * 2;
        let y_menu_result = y_menu_test + h_menu * 2;

        let mut selector = SelectBrowser::new(
            x_selector,
            y_selector,
            w_selector,
            h_selector,
            "Choices in this page",
        );
        let mut text = TextEditor::new(x_text, y_text, w_text, h_text, "Choice Text");
        let condition = Choice::new(x_menu, y_menu_condition, w_menu, h_menu, "Condition");
        let test = Choice::new(x_menu, y_menu_test, w_menu, h_menu, "Test");
        let result = Choice::new(x_menu, y_menu_result, w_menu, h_menu, "Result");
        group.end();

        text.set_buffer(TextBuffer::default());

        let (sender, _r) = app::channel();

        selector.set_callback({
            let sender = sender.clone();
            move |x| {
                if x.value() > 0 {
                    sender.send(x.value() - 1);
                }
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
        }
    }
}
/// Condition editor
///
/// Lists conditions by name
/// Customizes comparison and two expressions to evaluate
/// The story editor record inserters interactively insert tags here if the editor has focus
struct ConditionEditor {
    selector: SelectBrowser,
    name: TextEditor,
    expression_left: TextEditor,
    expression_right: TextEditor,
    comparison: fltk::menu::Choice,
}

impl ConditionEditor {
    fn new(area: Rect) -> Self {
        let group = Group::new(area.x, area.y, area.w, area.h, None);

        let x_selector = area.x;
        let y_selector = area.y;
        let w_selector = area.w / 10 * 4;
        let h_selector = area.h - 50;

        let x_name = area.x + area.w / 10 * 6;
        let y_name = y_selector + 20;
        let w_name = w_selector;
        let h_name = 20;

        let y_exp = y_selector + h_selector + 20;
        let w_exp = area.w / 10 * 4;
        let h_exp = 20;
        let x_exp_first = x_selector;
        let x_exp_second = area.w - w_exp;
        let x_comp = x_exp_first + w_exp;
        let w_comp = area.w - w_exp * 2;

        let mut selector =
            SelectBrowser::new(x_selector, y_selector, w_selector, h_selector, "Conditions");
        let mut name = TextEditor::new(x_name, y_name, w_name, h_name, "Name");
        let mut expression_left = TextEditor::new(x_exp_first, y_exp, w_exp, h_exp, None);
        let mut expression_right = TextEditor::new(x_exp_second, y_exp, w_exp, h_exp, None);
        let mut comparison = fltk::menu::Choice::new(x_comp, y_exp, w_comp, h_exp, None);
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

        comparison.add_choice(">|>=|<|<=|=|!=");
        comparison.set_value(0);

        Self {
            selector,
            name,
            expression_left,
            expression_right,
            comparison,
        }
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
        let group = Group::new(area.x, area.y, area.w, area.h, None);

        let x_selector = area.x;
        let y_selector = area.y;
        let w_selector = area.w / 10 * 4;
        let h_selector = area.h - 50;

        let x_name = area.x + area.w / 10 * 6;
        let y_name = y_selector + 20;
        let w_name = w_selector;
        let h_name = 20;

        let x_results = x_name;
        let w_result = w_name;
        let h_result = 20;
        let y_result_success = y_name + h_name + 10;
        let y_result_failure = y_result_success + h_result + 5;

        let y_exp = y_selector + h_selector + 20;
        let w_exp = area.w / 10 * 4;
        let h_exp = 20;
        let x_exp_first = x_selector;
        let x_exp_second = area.w - w_exp;
        let x_comp = x_exp_first + w_exp;
        let w_comp = area.w - w_exp * 2;

        let mut selector =
            SelectBrowser::new(x_selector, y_selector, w_selector, h_selector, "Conditions");
        let mut name = TextEditor::new(x_name, y_name, w_name, h_name, "Name");
        let mut expression_left = TextEditor::new(x_exp_first, y_exp, w_exp, h_exp, None);
        let mut expression_right = TextEditor::new(x_exp_second, y_exp, w_exp, h_exp, None);
        let mut comparison = fltk::menu::Choice::new(x_comp, y_exp, w_comp, h_exp, None);
        let success = fltk::menu::Choice::new(
            x_results,
            y_result_success,
            w_result,
            h_result,
            "On Success",
        );
        let failure = fltk::menu::Choice::new(
            x_results,
            y_result_failure,
            w_result,
            h_result,
            "On Failure",
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
    scroll: Scroll,
    effects: Vec<(fltk::menu::Choice, TextEditor)>,
}

impl ResultEditor {
    fn new(area: Rect) -> Self {
        let group = Group::new(area.x, area.y, area.w, area.h, None);

        let x_selector = area.x;
        let y_selector = area.y;
        let w_selector = area.w / 2 - 10;
        let h_selector = area.h / 2 - 5;

        let x_name = x_selector + w_selector + 20;
        let y_name = y_selector;
        let w_name = w_selector;
        let h_name = 20;

        let x_page = x_name;
        let y_page = y_name + h_name + 5;
        let w_page = w_name;
        let h_page = 20;

        let x_scroll = area.x;
        let y_scroll = y_selector + h_selector + 10;
        let w_scroll = area.w;
        let h_scroll = area.h - h_selector;

        let mut selector =
            SelectBrowser::new(x_selector, y_selector, w_selector, h_selector, "Results");
        let mut name = TextEditor::new(x_name, y_name, w_name, h_name, "Name");
        let next_page = fltk::menu::Choice::new(x_page, y_page, w_page, h_page, "Next Page");
        let scroll = Scroll::new(x_scroll, y_scroll, w_scroll, h_scroll, None);
        scroll.end();
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

        Self {
            selector,
            name,
            next_page,
            scroll,
            effects: Vec::new(),
        }
    }
}
