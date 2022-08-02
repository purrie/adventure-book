use fltk::{
    app::{self, App, Receiver},
    draw::Rect,
};

use crate::{
    adventure::{Adventure, Page},
    file::{capture_adventures, read_page},
    window::MainWindow,
};

pub struct Game {
    app: App,
    window: MainWindow,
    adventures: Vec<Adventure>,
    receiver: Receiver<Event>,
    selected_adventure: usize,
    active_storybook: Adventure,
    active_page: Page,
}

impl Game {
    pub fn create() -> Self {
        let app = App::default();
        let adventures = capture_adventures();
        let window_size = Rect::new(0, 0, 1000, 750);
        let window = MainWindow::create(window_size, window_size);
        let (_s, receiver) = app::channel();

        Game {
            app,
            adventures,
            window,
            receiver,
            selected_adventure: 0,
            active_storybook: Adventure::new(),
            active_page: Page::new(),
        }
    }
    pub fn start(mut self) {
        while self.app.wait() {
            if let Some(msg) = self.receiver.recv() {
                match msg {
                    Event::DisplayAdventureSelect => {
                        self.window.main_menu.fill_adventure_choices(&self.adventures);
                        self.window.switch_to_adventure_choice();
                    }
                    Event::DisplayMainMenu => self.window.switch_to_main_menu(),
                    Event::StartAdventure => {
                        self.active_storybook = self.adventures[self.selected_adventure].clone();
                        let page = self.active_storybook.start.clone();
                        self.switch_to_page(&page);
                        self.window.switch_to_game();
                    }
                    Event::Quit => {
                        app::quit();
                    }
                    Event::SelectAdventure(txt) => {
                        if let Some(find) = self.adventures.iter().position(|x| x.title == txt) {
                            self.selected_adventure = find;
                            let adventure = &self.adventures[find];
                            self.window.main_menu.set_adventure_preview_text(adventure);
                        }
                    }
                    Event::StoryChoice(index) => {
                        let choice = &self.active_page.choices[index];
                    }
                    Event::QuitToMainMenu => {
                        self.window.switch_to_adventure_choice();
                    }
                }
            }
        }
    }
    /// Changes currently displayed page.
    ///
    /// It refreshes windows contents to update changes in records and fills story and choices
    fn switch_to_page(&mut self, page_name: &String) {
        if let Ok(page) = read_page(&self.active_storybook.path, page_name) {
            self.active_page = page;
        } else {
            // TODO panic for now, replace with proper error handling later
            panic!(
                "Adventure {} or its starting page is corrupted!",
                self.active_storybook.title
            );
        }
        let story = self.parse_story_text();
        let choices = self.parse_choices();
        self.window.game_window.fill_choices(choices);
        self.window.game_window.display_story(story);
        self.window.game_window.update_records(&self.active_storybook.records);
    }
    fn parse_story_text(&self) -> String {
        unimplemented!();
    }
    fn parse_choices(&self) -> Vec<(bool, String)> {
        unimplemented!();
    }
}

#[derive(Clone)]
pub enum Event {
    DisplayMainMenu,
    DisplayAdventureSelect,
    StartAdventure,
    QuitToMainMenu,
    Quit,
    SelectAdventure(String),
    StoryChoice(usize),
}
