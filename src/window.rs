use fltk::{
    app,
    browser::SelectBrowser,
    button::Button,
    frame::Frame,
    group::Group,
    prelude::*,
    window::Window, text::{TextDisplay, TextBuffer},
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
        self.adventure_description.buffer().as_mut().unwrap().set_text(&adventure.description);
    }
    fn fill_adventure_choices(&mut self, adventures: &Vec<Adventure>) {
        self.adventure_picker.clear();
        for adv in adventures {
            self.adventure_picker.add(&adv.title);
        }
    }
}
