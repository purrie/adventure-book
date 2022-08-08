use adventure::{Adventure, Page};
use evaluation::{evaluate_result, Random};
use file::capture_adventures;
use fltk::{
    app::{self, App},
    draw::Rect,
    prelude::*,
    window::Window,
};
use game::{render_page, Event};
use window::MainWindow;

extern crate fltk;
extern crate rand;
extern crate regex;

mod adventure;
mod evaluation;
mod file;
mod game;
mod window;

fn main() {
    let app = App::default();
    let (_s, game_events) = app::channel();
    let adventures = capture_adventures();

    let window_size = Rect::new(0, 0, 1000, 750);
    let mut window = Window::new(
        window_size.x,
        window_size.y,
        window_size.w,
        window_size.h,
        "Adventure Book",
    );

    let mut main_window = MainWindow::create(window_size);

    window.end();
    window.show();

    let mut selected_adventure = usize::MAX;
    let mut active_storybook = Adventure::new();
    let mut active_page = Page::new();
    let mut rng = Random::new(69420);

    while app.wait() {
        if let Some(msg) = game_events.recv() {
            match msg {
                Event::Quit => {
                    app::quit();
                }
                // Enters adventure select screen
                Event::DisplayAdventureSelect => {
                    if adventures.len() > 0 {
                        main_window.main_menu.fill_adventure_choices(&adventures);
                        if selected_adventure == usize::MAX {
                            selected_adventure = 0;
                            let adventure = &adventures[0];
                            main_window.main_menu.set_adventure_preview_text(adventure);
                        }
                        main_window.switch_to_adventure_choice();
                    } else {
                        // TODO display alert saying that no adventures were found
                        panic!("Could not find any adventures!")
                    }
                }
                // Enters main menu screen
                Event::DisplayMainMenu => main_window.switch_to_main_menu(),
                Event::QuitToMainMenu => main_window.switch_to_adventure_choice(),
                // Changes which adventure is selected in adventure select screen
                Event::SelectAdventure(txt) => {
                    if let Some(find) = adventures.iter().position(|x| x.title == txt) {
                        selected_adventure = find;
                        let adventure = &adventures[find];
                        main_window.main_menu.set_adventure_preview_text(adventure);
                    }
                }

                // Enters gameplay screen and starts a new game
                Event::StartAdventure => {
                    active_storybook = adventures[selected_adventure].clone();
                    active_page = render_page(
                        &mut main_window,
                        &active_storybook,
                        &active_storybook.start,
                        &mut rng,
                    );
                    main_window.switch_to_game();
                }
                // Result of a choice button in gameplay screen, parses the choice and enters another storybook page into the screen
                Event::StoryChoice(index) => {
                    let choice = &active_page.choices[index];
                    let result;
                    if choice.is_constant() {
                        // the choice leads to a result straight away, just switching pages
                        if let Some(res) = active_page.results.get(&choice.result) {
                            result = res;
                        } else {
                            // the result doesn't exist TODO handle this in a better way
                            panic!(
                                "Page {}: The result {} isn't declared",
                                active_page.title, choice.result
                            );
                        }
                    } else {
                        if let Some(test) = &active_page.tests.get(&choice.test) {
                            let tres = test.evaluate(&active_storybook.records, &mut rng);
                            if let Some(res) = active_page.results.get(tres) {
                                result = res;
                            } else {
                                // TODO proper error handling
                                panic!(
                                    "Page {}: The result {} isn't declared",
                                    active_page.title, choice.result
                                );
                            }
                        } else {
                            // TODO proper error handing
                            panic!(
                                "Page {}: The test {} isn't declared",
                                active_page.title, choice.test
                            );
                        }
                    }

                    if let Ok(evaluated_result) =
                        evaluate_result(&result.expression, &active_storybook.records, &mut rng)
                    {
                        // first we process all the record changes
                        if let Some(delta) = evaluated_result.1 {
                            for (kee, val) in delta {
                                if let Some(rec) = active_storybook.records.get_mut(&kee) {
                                    rec.value += val;
                                }
                            }
                        }
                        // now we move on to the next scene
                        active_page = render_page(
                            &mut main_window,
                            &active_storybook,
                            &evaluated_result.0,
                            &mut rng,
                        );
                    } else {
                        // TODO handle this better
                        panic!(
                            "Page {}: result {} didn't evaluate properly",
                            active_page.title, result.name
                        )
                    }
                }
            }
        }
    }
}
