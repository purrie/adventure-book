use fltk::app::{self, App, Receiver};

use crate::{adventure::Adventure, file::capture_adventures, window::MainWindow};

pub struct Game {
    app: App,
    window: MainWindow,
    adventures: Vec<Adventure>,
    receiver: Receiver<Event>,
    active_adventure: usize,
}

impl Game {
    pub fn create() -> Self {
        let app = App::default();
        let adventures = capture_adventures();
        let window = MainWindow::create();
        let (_s, receiver) = app::channel();

        Game {
            app,
            adventures,
            window,
            receiver,
            active_adventure: 0,
        }
    }
    pub fn start(mut self) {
        while self.app.wait() {
            if let Some(msg) = self.receiver.recv() {
                match msg {
                    Event::DisplayAdventureSelect => {
                        self.window.fill_adventure_choices(&self.adventures);
                        self.window.switch_to_adventure_choice();
                    },
                    Event::DisplayMainMenu => self.window.switch_to_main_menu(),
                    Event::StartAdventure => {}
                    Event::Quit => {
                        app::quit();
                    }
                    Event::SelectAdventure(txt) => {
                        if let Some(find) = self.adventures.iter().position(|x| x.title == txt) {
                            self.active_adventure = find;
                            let adventure = &self.adventures[find];
                            self.window.set_adventure_choice(adventure);
                        }
                    }
                }
            }
        }
    }
    fn get_active_adventure(&self) -> &Adventure {
        &self.adventures[self.active_adventure]
    }
}

#[derive(Clone)]
pub enum Event {
    DisplayMainMenu,
    DisplayAdventureSelect,
    StartAdventure,
    Quit,
    SelectAdventure(String),
}
