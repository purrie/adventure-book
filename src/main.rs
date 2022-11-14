use adventure::{Adventure, Page};
use dialog::{ask_for_new_adventure, ask_to_choose_adventure, ask_to_confirm};
use evaluation::{evaluate_expression, Random};
use file::{capture_adventures, signal_error};
use fltk::{
    app::{self, App},
    draw::Rect,
    prelude::*,
    window::Window,
};
use game::{render_page, Event};
use window::MainWindow;

extern crate dirs;
extern crate fltk;
extern crate rand;
extern crate regex;

mod adventure;
mod dialog;
mod editor;
mod evaluation;
mod file;
mod game;
mod icons;
mod widgets;
mod window;

fn main() {
    let app = App::default();
    let (s, game_events) = app::channel();
    let mut adventures = capture_adventures();

    let window_size = Rect::new(0, 0, 1000, 750);
    let mut window = Window::new(
        window_size.x,
        window_size.y,
        window_size.w,
        window_size.h,
        "Adventure Book",
    );
    window.make_resizable(true);
    window.set_xclass("Choose your own adventure");

    let mut main_window = MainWindow::create(window_size);
    window.end();
    window.show();

    let mut selected_adventure = 0;
    let mut active_storybook = Adventure::default();
    let mut active_page = Page::default();
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
                        main_window.switch_to_adventure_choice();
                    } else {
                        signal_error!("Could not find any adventures!");
                        s.send(Event::DisplayMainMenu);
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
                    main_window.game_window.clear_records();
                    match render_page(
                        &mut main_window,
                        &active_storybook,
                        &active_storybook.start,
                        &mut rng,
                    ) {
                        Ok(v) => active_page = v,
                        Err(_) => {
                            signal_error!("The adventure has invalid start page");
                            s.send(Event::DisplayAdventureSelect);
                            continue;
                        }
                    }
                    main_window.switch_to_game();
                }
                // Result of a choice button in gameplay screen, parses the choice and enters another storybook page into the screen
                Event::StoryChoice(index) => {
                    let choice = &active_page.choices[index];
                    let result;
                    if choice.is_game_over() {
                        s.send(Event::QuitToMainMenu);
                        continue;
                    }
                    if choice.is_constant() {
                        // the choice leads to a result straight away, just switching pages
                        if let Some(res) = active_page.results.get(&choice.result) {
                            result = res;
                        } else {
                            signal_error!(
                                "Selected result ({}) doesn't exist in the page ({})!",
                                choice.result,
                                active_page.title
                            );
                            s.send(Event::DisplayAdventureSelect);
                            continue;
                        }
                    } else {
                        if let Some(test) = &active_page.tests.get(&choice.test) {
                            let tres = match test.evaluate(&active_storybook.records, &mut rng) {
                                Ok(v) => v,
                                Err(e) => {
                                    signal_error!("Error evaluating a test: {}", e);
                                    s.send(Event::DisplayAdventureSelect);
                                    continue;
                                }
                            };

                            if let Some(res) = active_page.results.get(tres) {
                                result = res;
                            } else {
                                signal_error!(
                                    "Page {}: The result {} isn't declared",
                                    active_page.title,
                                    choice.result
                                );
                                s.send(Event::DisplayAdventureSelect);
                                continue;
                            }
                        } else {
                            signal_error!(
                                "Page {}: The test {} isn't declared",
                                active_page.title,
                                choice.test
                            );
                            s.send(Event::DisplayAdventureSelect);
                            continue;
                        }
                    }

                    for mods in result.side_effects.iter() {
                        if active_storybook.records.contains_key(mods.0) {
                            if let Ok(v) =
                                evaluate_expression(mods.1, &active_storybook.records, &mut rng)
                            {
                                if let Some(r) = active_storybook.records.get_mut(mods.0) {
                                    r.value += v;
                                }
                            } else {
                                if ask_to_confirm(&format!("Misconfigured Result {} in page {}! The adventure will likely not proceed correctly, do you wish to return to main menu?", result.name, active_page.title)) {
                                    s.send(Event::QuitToMainMenu);
                                }
                            }
                        }
                    }
                    // now we move on to the next scene
                    match render_page(
                        &mut main_window,
                        &active_storybook,
                        &result.next_page,
                        &mut rng,
                    ) {
                        Ok(v) => active_page = v,
                        Err(e) => {
                            signal_error!("{}", e);
                            s.send(Event::DisplayAdventureSelect);
                            continue;
                        }
                    }

                    window.redraw();
                }
                Event::EditAdventure => {
                    if let Some(index) = ask_to_choose_adventure(&adventures) {
                        if let Some(ad) = adventures.get(index) {
                            main_window.editor_window.load_adventure(&ad, index);
                            main_window.switch_to_editor();
                        } else {
                            if let Some(ad) = ask_for_new_adventure() {
                                main_window
                                    .editor_window
                                    .load_adventure(&ad, adventures.len());
                                adventures.push(ad);
                                main_window.switch_to_editor();
                            }
                        }
                    }
                }
                Event::Editor(e) => {
                    if e == crate::editor::Event::Save {
                        main_window.editor_window.process(e);
                        let ret = main_window.editor_window.get_adventure();
                        if let Some(index) = ret.1 {
                            adventures[index] = ret.0;
                        } else {
                            adventures.push(ret.0);
                        }
                    } else {
                        main_window.editor_window.process(e);
                    }
                }
            }
        }
    }
}
