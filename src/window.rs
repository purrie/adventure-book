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
    window::Window,
};

use crate::{adventure::Adventure, game::Event};

pub struct MainWindow {
    window: Window,
    main_menu: MainMenu,
}
struct MainMenu {
    main_manu: Group,
    start_menu: Group,
    adventure_choice: Group,
    adventure_title: Label,
    adventure_description: TextDisplay,
    adventure_picker: SelectBrowser,
}
struct GameWindow {
    game_window: Group,
    pub records: RecordWindow,
    pub story: StoryWindow,
    pub choices: ChoiceWindow,
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
    window: Group,
    text: TextDisplay,
}

type Label = Frame;

impl MainWindow {
    pub fn create() -> MainWindow {
        let mut window = Window::default()
            .with_size(1000, 750)
            .with_label("Adventure Book");
        let main_menu = MainMenu::create();

        window.end();
        window.show();

        MainWindow { window, main_menu }
    }

    pub fn switch_to_main_menu(&mut self) {
        self.main_menu.show_main();
    }
    pub fn switch_to_adventure_choice(&mut self) {
        self.main_menu.show_choice();
    }
    pub fn set_adventure_choice(&mut self, adventure: &Adventure) {
        self.main_menu.set_adventure_text(adventure);
    }
    pub fn fill_adventure_choices(&mut self, adventures: &Vec<Adventure>) {
        self.main_menu.fill_adventure_choices(adventures);
    }
}
impl MainMenu {
    fn create() -> MainMenu {
        let group = Group::default().size_of_parent();

        let main = Group::default().size_of_parent();
        let mut new_but = Button::new(450, 250, 100, 20, "New Game");
        let mut quit_but = Button::new(450, 280, 100, 20, "Quit");
        main.end();

        let mut starting = Group::default().size_of_parent();

        let title = Label::new(50, 100, 400, 20, "Select the Adventure");
        let mut desc_buffer = TextBuffer::default();
        desc_buffer.set_text("");
        let mut description = TextDisplay::new(50, 130, 400, 520, "");
        description.set_buffer(desc_buffer);
        description.wrap_mode(fltk::text::WrapMode::AtBounds, 0);

        let mut picker = SelectBrowser::new(550, 100, 400, 550, "");

        let mut back = Button::new(100, 700, 100, 20, "Back");
        let mut accept = Button::new(800, 700, 100, 20, "Start");

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
    fn show_main(&mut self) {
        self.main_manu.show();
        self.start_menu.show();
        self.adventure_choice.hide();
    }
    fn show_choice(&mut self) {
        self.main_manu.show();
        self.start_menu.hide();
        self.adventure_choice.show();
    }
    fn hide(&mut self) {
        self.main_manu.hide();
    }
    fn set_adventure_text(&mut self, adventure: &Adventure) {
        self.adventure_title.set_label(&adventure.title);
        self.adventure_description
            .buffer()
            .as_mut()
            .unwrap()
            .set_text(&adventure.description);
    }
    fn fill_adventure_choices(&mut self, adventures: &Vec<Adventure>) {
        self.adventure_picker.clear();
        for adv in adventures {
            self.adventure_picker.add(&adv.title);
        }
    }
}

impl GameWindow {
    /// creates UI for interacting with the story
    pub fn create(area: Rect) -> Self {
        let width_large = (area.w as f64 * 0.7) as i32;
        let width_small = area.w - width_large;
        let height_large = (area.h as f64 * 0.7) as i32;
        let height_small = area.h - height_large;

        let choice_area = Rect {
            x: area.x,
            y: area.y + height_large,
            w: area.w,
            h: height_small,
        };
        let record_area = Rect {
            x: area.x,
            y: area.y,
            w: width_small,
            h: height_large,
        };
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
    pub fn show(&mut self) {
        self.game_window.show();
    }
    /// hides the game play UI
    pub fn hide(&mut self) {
        self.game_window.hide();
    }
}
impl RecordWindow {
    /// Creates a new record window in provided area
    ///
    /// The window will be empty, use add_record and update_record to display things
    ///
    /// Record window also stores game specific buttons, like returning to main menu
    pub fn create(rect: Rect) -> Self {
        let root_window = Group::new(rect.x, rect.y, rect.w, rect.h, "");
        let window = Flex::new(rect.x, rect.y, rect.w, rect.h - 40, "").column();
        window.end();
        let mut butt = Button::new(rect.x, rect.h - 30, 20, 20, "@returnarrow");
        root_window.end();

        let (s, _r) = app::channel();

        butt.emit(s, Event::QuitToMainMenu);

        RecordWindow {
            window,
            categories: HashMap::new(),
        }
    }
    /// Shows the categories and records
    pub fn show(&mut self) {
        self.window.show();
    }
    /// Hides the categories and records
    pub fn hide(&mut self) {
        self.window.hide();
    }
    /// Removes all group and record displays
    pub fn clear(&mut self) {
        self.window.clear();
    }
    /// This will add a record into the window.
    ///
    /// Any groups for categories will be created if they haven't been already
    pub fn add_record(&mut self, record: &str, category: &str) {
        let &mut cat;

        // creating a category if it haven't been created yet, otherwise we just grab it
        if self.categories.contains_key(category) {
            cat = self.categories.get_mut(category).unwrap();
        } else {
            // here is group creation
            let mut group = Flex::default().column();
            group.set_margin(4);
            group.set_label(category);
            group.end();
            self.window.add(&group);

            let ccat = RecordCategory {
                group,
                entries: HashMap::new(),
            };
            self.categories.insert(category.to_string(), ccat);
            cat = self.categories.get_mut(category).unwrap();
        }

        if cat.entries.contains_key(record) == false {
            let f = Frame::default().with_label(format!("{}: 0", record).as_str());
            cat.group.add(&f);
            cat.entries.insert(record.to_string(), f);
        }
    }
    /// Updates displayed value for the record.
    ///
    /// It will silently fail if the record or group haven't been found
    pub fn update_record(&mut self, record: &str, category: &str, value: i32) {
        if let Some(cat) = self.categories.get_mut(category) {
            if let Some(rec) = cat.entries.get_mut(record) {
                rec.set_label(format!("{}: {}", record, value).as_str());
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
        let button_container = Flex::default().size_of_parent().column();
        window.end();

        Self {
            window,
            button_container,
        }
    }
    /// Adds a button with supplied text as available choice
    pub fn add_choice(&mut self, text: &str) {
        let count = self.button_container.children();
        let mut butt = Button::default().with_label(format!("{}: {}", count + 1, text).as_str());

        let (s, _r) = app::channel();
        butt.set_callback(move |_| {
            s.send(Event::StoryChoice(count as usize));
        });
        self.button_container.add(&butt);
    }
    /// Removes all choice buttons from the menu
    pub fn clear_choices(&mut self) {
        self.button_container.clear();
    }
}
impl StoryWindow {
    /// Creates empty story area
    ///
    /// The story window is where the main story events are displayed
    fn create(area: Rect) -> Self {
        let window = Group::new(area.x, area.y, area.w, area.h, "");
        let mut buff = TextBuffer::default();
        buff.set_text("");
        let mut text = TextDisplay::default().size_of_parent();
        text.set_buffer(buff);
        text.wrap_mode(fltk::text::WrapMode::AtBounds, 0);
        window.end();

        StoryWindow { window, text }
    }
    /// Sets text to the display
    pub fn set_text(&mut self, text: &str) {
        self.text.buffer().as_mut().unwrap().set_text(text);
    }
    /// Removes all text from the display
    pub fn clear_text(&mut self) {
        self.text.buffer().as_mut().unwrap().set_text("");
    }
}
