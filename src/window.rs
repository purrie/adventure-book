use std::collections::HashMap;

use fltk::{
    app,
    browser::SelectBrowser,
    button::Button,
    draw::Rect,
    frame::Frame,
    group::{Flex, Group, Scroll},
    prelude::*,
    text::{TextBuffer, TextDisplay},
};

use crate::adventure::Record;
use crate::{adventure::Adventure, widgets::TextRenderer};
use crate::{file::get_image_png, game::Event};

pub struct MainWindow {
    pub main_menu: MainMenu,
    pub game_window: GameWindow,
}
pub struct MainMenu {
    main_manu: Group,
    start_menu: Group,
    adventure_choice: Group,
    adventure_title: Label,
    adventure_description: TextRenderer,
    adventure_picker: SelectBrowser,
}
pub struct GameWindow {
    game_window: Group,
    records: RecordWindow,
    story: StoryWindow,
    choices: ChoiceWindow,
}
struct RecordWindow {
    window: Flex,
    categories: HashMap<String, RecordCategory>,
}
struct RecordCategory {
    group: Flex,
    entries: HashMap<String, Frame>,
}
struct ChoiceWindow {
    window: Scroll,
    button_container: Flex,
}
struct StoryWindow {
    text: TextDisplay,
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
        game_window.hide();

        MainWindow {
            main_menu,
            game_window,
        }
    }

    /// Switches the view to main menu
    ///
    /// It will hide game UI if it is shown
    pub fn switch_to_main_menu(&mut self) {
        self.game_window.hide();
        self.main_menu.show_main();
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
}
impl MainMenu {
    fn create(area: Rect) -> MainMenu {
        let group = Group::new(area.x, area.y, area.w, area.h, "");

        let main = Group::default().size_of_parent();
        if let Ok(image) = get_image_png("title.png") {
            let mut background = Frame::default().size_of_parent();
            background.set_image_scaled(Some(image));
        }
        let mut title = Frame::new(area.w / 2 - 100 + area.x, 150, 200, 40, "Adventure Book");
        title.set_label_size(20);
        let but_x = area.w / 2 - 50 + area.x;
        let but_y = area.h / 2 - 50 + area.y;
        let mut new_but = Button::new(but_x, but_y, 100, 20, "New Game");
        let mut quit_but = Button::new(but_x, but_y + 30, 100, 20, "Quit");
        main.end();

        let mut starting = Group::default().size_of_parent();

        if let Ok(image) = get_image_png("choice.png") {
            let mut background = Frame::default().size_of_parent();
            background.set_image_scaled(Some(image));
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

        let mut picker =
            SelectBrowser::new(middle_border, top_border, half_width, chooser_height, "");

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
        back.emit(send.clone(), Event::DisplayMainMenu);
        quit_but.emit(send.clone(), Event::Quit);
        accept.emit(send.clone(), Event::StartAdventure);

        picker.set_callback(move |x| {
            if let Some(txt) = x.selected_text() {
                send.send(Event::SelectAdventure(txt));
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
        self.adventure_picker.clear();
        for adv in adventures {
            self.adventure_picker.add(&adv.title);
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

        let choices = ChoiceWindow::create(choice_area);
        let records = RecordWindow::create(record_area);
        let story = StoryWindow::create(story_area);

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
    pub fn display_story(&mut self, story: String) {
        self.story.set_text(&story);
    }
    pub fn update_records(&mut self, records: &HashMap<String, Record>) {
        for rec in records.iter() {
            self.records.update_record(rec.1);
        }
    }
    /// Adds records to the record window
    ///
    /// don't call more than once per game
    /// use update_records to update the screen
    pub fn fill_records(&mut self, records: &HashMap<String, Record>) {
        self.records.clear();
        for rec in records.iter() {
            self.records.add_record(rec.1);
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
        self.choices.window.redraw();
    }
}
impl RecordWindow {
    /// Creates a new record window in provided area
    ///
    /// The window will be empty, use add_record and update_record to display things
    ///
    /// Record window also stores game specific buttons, like returning to main menu
    fn create(rect: Rect) -> Self {
        let root_window = Group::new(rect.x, rect.y, rect.w, rect.h, "");
        let window = Flex::new(rect.x, rect.y, rect.w, rect.h - 40, "").column();
        window.end();
        let mut butt = Button::new(rect.x + 10, rect.h - 30, 20, 20, "@returnarrow");
        root_window.end();

        let (s, _r) = app::channel();

        butt.emit(s, Event::QuitToMainMenu);

        RecordWindow {
            window,
            categories: HashMap::new(),
        }
    }
    /// Removes all group and record displays
    fn clear(&mut self) {
        self.window.clear();
        self.categories.clear();
    }
    /// This will add a record into the window.
    ///
    /// Any groups for categories will be created if they haven't been already
    fn add_record(&mut self, record: &Record) {
        let &mut cat;

        // creating a category if it haven't been created yet, otherwise we just grab it
        if self.categories.contains_key(&record.category) {
            cat = self.categories.get_mut(&record.category).unwrap();
        } else {
            // here is group creation
            let mut group = Flex::default().column();
            group.set_margin(4);
            group.set_label(&record.category);
            group.end();
            self.window.add(&group);

            let ccat = RecordCategory {
                group,
                entries: HashMap::new(),
            };
            self.categories.insert(record.category.to_string(), ccat);
            cat = self.categories.get_mut(&record.category).unwrap();
        }

        if cat.entries.contains_key(&record.name) == false {
            let f =
                Frame::default().with_label(format!("{}: {}", record.name, record.value).as_str());
            cat.group.add(&f);
            cat.entries.insert(record.name.to_string(), f);
        }
    }
    /// Updates displayed value for the record.
    ///
    /// It will silently fail if the record or group haven't been found
    fn update_record(&mut self, record: &Record) {
        if let Some(cat) = self.categories.get_mut(&record.category) {
            if let Some(rec) = cat.entries.get_mut(&record.name) {
                rec.set_label(format!("{}: {}", record.name, record.value).as_str());
            }
        }
    }
}
impl ChoiceWindow {
    /// Creates empty choice menu
    ///
    /// Use add_choice and clear_choices to populate and clear the menu
    fn create(area: Rect) -> Self {
        let window = Scroll::new(area.x, area.y, area.w, area.h, "");
        let button_container = Flex::new(area.x, area.y, area.w, area.h, "").column();
        window.end();

        Self {
            window,
            button_container,
        }
    }
    /// Adds a button with supplied text as available choice
    fn add_choice(&mut self, text: &str, active: bool) {
        let count = self.button_container.children();
        let mut butt = Button::default().with_label(format!("{}: {}", count + 1, text).as_str());

        let (s, _r) = app::channel();
        butt.set_callback(move |_| {
            s.send(Event::StoryChoice(count as usize));
        });
        self.button_container.add(&butt);
        if active {
            butt.activate();
        } else {
            butt.deactivate();
        }
    }
    /// Removes all choice buttons from the menu
    fn clear_choices(&mut self) {
        self.button_container.clear();
    }
}
impl StoryWindow {
    /// Creates empty story area
    ///
    /// The story window is where the main story events are displayed
    fn create(area: Rect) -> Self {
        let mut buff = TextBuffer::default();
        buff.set_text("");
        let mut text = TextDisplay::new(area.x, area.y, area.w, area.h, "");
        text.set_buffer(buff);
        text.wrap_mode(fltk::text::WrapMode::AtBounds, 0);

        StoryWindow { text }
    }
    /// Sets text to the display
    fn set_text(&mut self, text: &str) {
        self.text.buffer().as_mut().unwrap().set_text(text);
    }
}
