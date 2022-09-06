use std::{cell::RefCell, collections::HashMap, rc::Rc};

use fltk::{
    app,
    button::Button,
    draw::{draw_text, draw_text2, pop_clip, push_clip, Rect},
    enums::Align,
    frame::Frame,
    group::{Group, Scroll},
    prelude::*,
    widget::Widget,
    widget_extends,
};

use crate::{
    adventure::{Adventure, Record},
    editor::EditorWindow,
    file::get_image_png,
    game::Event,
    widgets::{Selector, TextRenderer},
};

pub struct MainWindow {
    pub main_menu: MainMenu,
    pub game_window: GameWindow,
    pub editor_window: EditorWindow,
}
pub struct MainMenu {
    main_manu: Group,
    start_menu: Group,
    adventure_choice: Group,
    adventure_title: Label,
    adventure_description: TextRenderer,
    adventure_picker: Rc<RefCell<Selector>>,
}
pub struct GameWindow {
    game_window: Group,
    records: RecordWindow,
    story: StoryWindow,
    choices: ChoiceWindow,
}
struct RecordWindow {
    widget: Widget,
    categories: Rc<RefCell<HashMap<String, HashMap<String, i32>>>>,
}
struct ChoiceWindow {
    window: Scroll,
}
struct StoryWindow {
    text: TextRenderer,
}

type Label = Frame;

impl MainWindow {
    /// Creates a window and all the associated UI
    ///
    /// window_area: size and position of the window
    /// ui_area: area within the window that will be used for placing the controls
    pub fn create(ui_area: Rect) -> MainWindow {
        let main_menu = MainMenu::create(ui_area);
        let mut game_window = GameWindow::create(ui_area);
        let mut editor_window = EditorWindow::new(ui_area);
        game_window.hide();
        editor_window.hide();

        MainWindow {
            main_menu,
            game_window,
            editor_window,
        }
    }

    /// Switches the view to main menu
    ///
    /// It will hide game UI if it is shown
    pub fn switch_to_main_menu(&mut self) {
        self.game_window.hide();
        self.main_menu.show_main();
        self.editor_window.hide();
    }
    /// Switches to adventure choice menu
    ///
    /// It will hide game UI if it is shown
    pub fn switch_to_adventure_choice(&mut self) {
        self.game_window.hide();
        self.main_menu.show_choice();
    }
    /// Switches UI to display game interface
    ///
    /// It replaces main menu UI
    pub fn switch_to_game(&mut self) {
        self.main_menu.hide();
        self.game_window.show();
    }
    pub fn switch_to_editor(&mut self) {
        self.main_menu.hide();
        self.editor_window.show();
    }
}
impl MainMenu {
    fn create(area: Rect) -> MainMenu {
        let group = Group::new(area.x, area.y, area.w, area.h, "");

        let main = Group::default().size_of_parent();
        if let Ok(mut image) = get_image_png("title.png") {
            let mut img = Widget::default().size_of_parent();
            img.draw(move |b| {
                image.scale(b.width(), b.height(), false, true);
                image.draw(b.x(), b.y(), b.width(), b.height());
            });
        }
        let mut title = Frame::new(area.w / 2 - 100 + area.x, 150, 200, 40, "Adventure Book");
        title.set_label_size(20);
        let but_x = area.w / 2 - 50 + area.x;
        let but_y = area.h / 2 - 50 + area.y;
        let mut new_but = Button::new(but_x, but_y, 100, 20, "New Game");
        let mut edit_but = Button::new(but_x, but_y + 30, 100, 20, "Editor");
        let mut quit_but = Button::new(but_x, but_y + 60, 100, 20, "Quit");
        main.end();

        let mut starting = Group::default().size_of_parent();

        if let Ok(mut image) = get_image_png("choice.png") {
            let mut img = Widget::default().size_of_parent();
            img.draw(move |b| {
                image.scale(b.width(), b.height(), false, true);
                image.draw(b.x(), b.y(), b.width(), b.height());
            });
        }
        let horizontal_margin = 80;
        let vertical_margin = 100;

        let left_border = area.x + horizontal_margin;
        let half_width = area.w / 2 - horizontal_margin - horizontal_margin / 2;
        let middle_border = area.w / 2 + horizontal_margin / 2;

        let top_border = area.y + vertical_margin;
        let chooser_height = area.h - vertical_margin * 2;
        let bottom_border = area.h - vertical_margin / 2;

        let title = Label::new(
            left_border,
            top_border,
            half_width,
            20,
            "Select the Adventure",
        );

        let description = TextRenderer::new(
            left_border,
            top_border + 30,
            half_width,
            chooser_height - 30,
            "",
        );

        let picker = Selector::new(middle_border, top_border, half_width, chooser_height);

        let mut back = Button::new(
            left_border + horizontal_margin,
            bottom_border,
            100,
            20,
            "Back",
        );
        let mut accept = Button::new(area.w - 200, bottom_border, 100, 20, "Start");

        starting.end();
        starting.hide();
        group.end();

        let (send, _r) = app::channel();

        new_but.emit(send.clone(), Event::DisplayAdventureSelect);
        edit_but.emit(send.clone(), Event::EditAdventure);
        back.emit(send.clone(), Event::DisplayMainMenu);
        quit_but.emit(send.clone(), Event::Quit);
        accept.emit(send.clone(), Event::StartAdventure);

        let picker = Rc::new(RefCell::new(picker));
        picker.borrow_mut().set_callback({
            let picker: Rc<RefCell<Selector>> = Rc::clone(&picker);
            move |_| {
                if let Some(txt) = picker.borrow().selected_text() {
                    send.send(Event::SelectAdventure(txt));
                }
            }
        });

        MainMenu {
            main_manu: group,
            start_menu: main,
            adventure_choice: starting,
            adventure_title: title,
            adventure_description: description,
            adventure_picker: picker,
        }
    }
    /// Switches the screen to display main menu
    fn show_main(&mut self) {
        self.main_manu.show();
        self.start_menu.show();
        self.adventure_choice.hide();
    }
    /// Switches the screen to display adventure choice / new game menu
    fn show_choice(&mut self) {
        self.main_manu.show();
        self.start_menu.hide();
        self.adventure_choice.show();
    }
    /// Hides active screen
    fn hide(&mut self) {
        self.main_manu.hide();
    }
    /// Fills adventure information preview area with supplied adventure data
    pub fn set_adventure_preview_text(&mut self, adventure: &Adventure) {
        self.adventure_title.set_label(&adventure.title);
        self.adventure_description.set_text(&adventure.description);
    }
    /// Fills chooser control with adventures to choose from
    pub fn fill_adventure_choices(&mut self, adventures: &Vec<Adventure>) {
        self.adventure_picker.borrow_mut().clear();
        for adv in adventures {
            self.adventure_picker.borrow_mut().add(adv.title.clone());
        }
    }
}

impl GameWindow {
    /// creates UI for interacting with the story
    fn create(area: Rect) -> Self {
        let width_large = (area.w as f64 * 0.7) as i32;
        let width_small = area.w - width_large;
        let height_large = (area.h as f64 * 0.7) as i32;
        let height_small = area.h - height_large;

        // area where choices will be presented
        // placed along the bottom of the window
        let choice_area = Rect {
            x: area.x,
            y: area.y + height_large,
            w: area.w,
            h: height_small,
        };

        // area where the list of records is show to the player
        // placed along left side
        let record_area = Rect {
            x: area.x,
            y: area.y,
            w: width_small,
            h: height_large,
        };
        // Area where story text is displayed
        // placed in top right part of the window
        let story_area = Rect {
            x: area.x + width_small,
            y: area.y,
            w: width_large,
            h: height_large,
        };

        let game_window = Group::new(area.x, area.y, area.w, area.h, "");

        if let Ok(mut image) = get_image_png("story.png") {
            let mut img = Widget::default().size_of_parent();
            img.draw(move |b| {
                image.scale(b.width(), b.height(), false, true);
                image.draw(b.x(), b.y(), b.width(), b.height());
            });
        }

        let choices = ChoiceWindow::create(choice_area);
        let records = RecordWindow::create(record_area);
        let story = StoryWindow::create(story_area);

        let mut butt = Button::new(record_area.x + 10, record_area.h - 30, 20, 20, "@<-");
        let (s, _r) = app::channel();

        butt.emit(s, Event::QuitToMainMenu);

        game_window.end();

        Self {
            game_window,
            choices,
            records,
            story,
        }
    }
    /// shows the game play UI
    fn show(&mut self) {
        self.game_window.show();
    }
    /// hides the game play UI
    fn hide(&mut self) {
        self.game_window.hide();
    }
    /// fills the story window with provided text
    pub fn display_story(&mut self, story: String) {
        self.story.set_text(&story);
    }
    /// Clears record window
    pub fn clear_records(&mut self) {
        self.records.clear();
    }
    /// Adds records to the record window
    ///
    /// don't call more than once per game
    /// use update_records to update the screen
    pub fn fill_records(&mut self, records: &HashMap<String, Record>) {
        for rec in records.iter() {
            self.records.set_record(rec.1);
        }
    }
    /// Updates choices window
    ///
    /// All choices are removed first, then the window is filled with supplied choices
    /// Expected list of choices consists of tuples that have choice text
    /// and a flag that determines if the choice is active or not
    pub fn fill_choices(&mut self, choices: Vec<(bool, String)>) {
        self.choices.clear_choices();
        for choice in choices {
            self.choices.add_choice(&choice.1, choice.0);
        }
    }
}
widget_extends!(RecordWindow, Widget, widget);
impl RecordWindow {
    /// Creates a new record window in provided area
    ///
    /// The window will be empty, use add_record and update_record to display things
    ///
    /// Record window also stores game specific buttons, like returning to main menu
    fn create(rect: Rect) -> Self {
        let mut widget = Widget::new(rect.x, rect.y, rect.w, rect.h - 40, None);
        let categories = Rc::new(RefCell::new(HashMap::new()));

        widget.draw({
            let categories: Rc<RefCell<HashMap<String, HashMap<String, i32>>>> =
                Rc::clone(&categories);
            move |wid| {
                let x = wid.x();
                let y = wid.y();
                let w = wid.w();
                let h = wid.h();
                let font_size = wid.label_size() + wid.label_size() / 4;
                let el = categories.borrow();
                let mut offset = font_size;

                push_clip(x, y, w, h);
                draw_text2(
                    "Story Records",
                    x,
                    y + offset,
                    w - w / 4,
                    font_size,
                    Align::Center,
                );
                offset += font_size * 3;
                for e in el.iter() {
                    draw_text(&e.0, x + 10, y + offset);
                    offset += font_size;
                    for c in e.1.iter() {
                        let txt = format!("{}: {}", c.0, c.1);
                        draw_text(&txt, x + 20, y + offset);
                        offset += font_size;
                    }
                }
                pop_clip();
            }
        });

        RecordWindow { widget, categories }
    }
    /// Removes all group and record displays
    fn clear(&mut self) {
        self.categories.borrow_mut().clear();
    }
    /// This will add a record into the window.
    ///
    /// Any records for categories will be created if they haven't been already
    /// Existing records will be updated
    fn set_record(&mut self, record: &Record) {
        let mut categories = self.categories.borrow_mut();
        let &mut cat;

        // creating a category if it haven't been created yet, otherwise we just grab it
        if let Some(v) = categories.get_mut(&record.category) {
            cat = v
        } else {
            // here is group creation
            let new_group = HashMap::new();
            categories.insert(record.category.clone(), new_group);
            cat = categories.get_mut(&record.category).unwrap();
        }
        cat.insert(record.name.clone(), record.value);
    }
}
impl ChoiceWindow {
    /// Creates empty choice menu
    ///
    /// Use add_choice and clear_choices to populate and clear the menu
    fn create(area: Rect) -> Self {
        let window = Scroll::new(area.x, area.y, area.w, area.h, "");
        window.end();

        Self { window }
    }
    /// Adds a button with supplied text as available choice
    fn add_choice(&mut self, text: &str, active: bool) {
        let count = self.window.children() - 2;
        let label = format!("{}: {}", count + 1, text);
        let mut butt = Button::new(
            self.window.x(),
            self.window.y() + count * 30,
            self.window.width(),
            25,
            "",
        );
        butt.set_label(&label);

        let (s, _r) = app::channel();
        butt.set_callback(move |_| {
            s.send(Event::StoryChoice(count as usize));
        });
        butt.handle(|wid, ev| {
            use fltk::enums::Event;
            if let Event::Resize = ev {
                let parent = wid.parent().unwrap();
                let w = parent.w();
                let h = wid.h();
                wid.set_size(w, h);
                wid.redraw();
            }
            // returning false because otherwise only the first button gets redrawn properly on resize
            false
        });

        self.window.add(&butt);
        if active {
            butt.activate();
        } else {
            butt.deactivate();
        }
    }
    /// Removes all choice buttons from the menu
    fn clear_choices(&mut self) {
        self.window.clear();
    }
}
impl StoryWindow {
    /// Creates empty story area
    ///
    /// The story window is where the main story events are displayed
    fn create(area: Rect) -> Self {
        let text = TextRenderer::new(area.x + 30, area.y + 100, area.w - 80, area.h - 100, "");

        StoryWindow { text }
    }
    /// Sets text to the display
    fn set_text(&mut self, text: &str) {
        self.text.set_text(text);
    }
}
