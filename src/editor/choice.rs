use fltk::{
    app,
    browser::SelectBrowser,
    button::Button,
    draw::Rect,
    frame::Frame,
    group::Group,
    image::SvgImage,
    prelude::*,
    text::{TextBuffer, TextEditor},
};
type Dropdown = fltk::menu::Choice;

use crate::{
    adventure::{Choice, Page, GAME_OVER_KEYWORD},
    dialog::ask_to_confirm,
    editor::{emit, variables::variable_receiver, Event},
    icons::BIN_ICON,
};

/// Editor for customizing choices for a page
///
/// Displays a list of choices for the page
/// It has a text editor for the choice text, and drop downs for choosing condition, test and result for each choice
pub struct ChoiceEditor {
    selector: SelectBrowser,
    text: TextEditor,
    condition: Dropdown,
    test: Dropdown,
    result: Dropdown,
    condition_label: Frame,
    test_label: Frame,
    result_label: Frame,
}

impl ChoiceEditor {
    /// Creates UI for choice editor
    pub fn new(area: Rect) -> Self {
        let font_size = app::font_size();

        let group = Group::new(area.x, area.y, area.w, area.h, "Choices");

        let x_selector = area.x;
        let y_selector = area.y;
        let w_selector = area.w / 3;
        let h_selector = area.h - font_size;

        let y_butt = y_selector + h_selector;
        let w_butt = font_size;
        let h_butt = w_butt;
        let x_butt_add = x_selector;
        let x_butt_rem = x_selector + w_selector - w_butt;

        let margin_menu = 20;
        let x_menu = area.x + w_selector + margin_menu;
        let w_menu = area.w - w_selector - margin_menu * 2;
        let h_menu = font_size + font_size / 2;
        let y_menu_condition = area.y + h_menu;
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
        let mut butt_add = Button::new(x_butt_add, y_butt, w_butt, h_butt, "@+");
        let mut butt_rem = Button::new(x_butt_rem, y_butt, w_butt, h_butt, None);

        let mut text = TextEditor::new(x_text, y_text, w_text, h_text, "Choice Text");
        let condition_label = Frame::new(
            x_menu,
            y_menu_condition - font_size,
            w_menu,
            h_menu,
            "Condition",
        );
        let condition = Dropdown::new(x_menu, y_menu_condition, w_menu, h_menu, None);
        let test_label = Frame::new(x_menu, y_menu_test - font_size, w_menu, h_menu, "Test");
        let mut test = Dropdown::new(x_menu, y_menu_test, w_menu, h_menu, None);
        let result_label = Frame::new(x_menu, y_menu_result - font_size, w_menu, h_menu, "Result");
        let mut result = Dropdown::new(x_menu, y_menu_result, w_menu, h_menu, None);
        group.end();

        let mut bin = SvgImage::from_data(BIN_ICON).unwrap();
        bin.scale(font_size, font_size, false, true);
        butt_rem.set_image(Some(bin));

        text.set_buffer(TextBuffer::default());

        let (s, _r) = app::channel();
        butt_add.emit(s.clone(), emit!(Event::AddChoice));
        butt_rem.emit(s.clone(), emit!(Event::RemoveChoice));

        selector.set_callback({
            let mut old_selection = 0;
            move |x| {
                // when there are no elements, it means we're clearing out the selector and resetting the selection
                if x.size() == 0 {
                    old_selection = 0;
                    x.select(0);
                    return;
                }
                let new_selection = x.value();
                if old_selection != new_selection {
                    let (s, _r) = app::channel();
                    if old_selection != 0 {
                        s.send(emit!(Event::SaveChoice(Some((old_selection - 1) as usize))));
                    }
                    if new_selection > 0 {
                        s.send(emit!(Event::LoadChoice((new_selection - 1) as usize)));
                    }
                    old_selection = new_selection;
                }
            }
        });
        test.set_callback({
            let mut result = result.clone();
            move |x| {
                if x.value() >= 0 {
                    if result.value() >= 0 {
                        result.set_value(-1);
                    }
                }
            }
        });
        result.set_callback({
            let mut test = test.clone();
            move |x| {
                if x.value() >= 0 {
                    if test.value() >= 0 {
                        test.set_value(-1);
                    }
                }
            }
        });

        variable_receiver!(text);

        Self {
            selector,
            text,
            test,
            condition,
            result,
            condition_label,
            test_label,
            result_label,
        }
    }
    /// Hides controls
    ///
    /// Used for hiding the UI when user isn't meant to manipulate it
    fn hide_controls(&mut self) {
        self.condition_label.hide();
        self.condition.hide();
        self.test_label.hide();
        self.test.hide();
        self.result_label.hide();
        self.result.hide();
        self.text.hide();
    }
    /// Displays controls
    ///
    /// Used to display the UI when user adds a new choice to the UI
    fn show_controls(&mut self) {
        self.condition_label.show();
        self.condition.show();
        self.test_label.show();
        self.test.show();
        self.result_label.show();
        self.result.show();
        self.text.show();
    }
    /// Clears and readds elements to dropdown menus, refreshing available choices
    ///
    /// The function will attempt to reload previously selected choice
    pub fn populate_dropdowns(&mut self, page: &Page) {
        self.condition.clear();
        page.conditions
            .iter()
            .for_each(|x| self.condition.add_choice(x.0));
        self.test.clear();
        page.tests.iter().for_each(|x| self.test.add_choice(x.0));
        self.result.clear();
        page.results
            .iter()
            .for_each(|x| self.result.add_choice(x.0));
        self.result.add_choice(GAME_OVER_KEYWORD);
    }
    /// Refreshes dropdowns and selected choice
    ///
    /// This is used to load changes from other editors when going back to choice tab
    pub fn refresh_dropdowns(&mut self, page: &Page) {
        self.populate_dropdowns(page);
        // reloading the previously selected choice
        let selected = self.selector.value();
        if selected > 0 {
            self.load_choice(&page.choices, (selected - 1) as usize);
        }
    }
    /// Clears and repopulates the selector UI with available choices
    ///
    /// The function will also attempt to select first choice if it is available
    pub fn populate_choices(&mut self, choices: &Vec<Choice>) {
        self.selector.clear();
        // Clearing the selector selection index
        // This is needed to be able to refresh the UI when switching between two pages that have only one choice
        // Since both will have the same index number and the selector will think it's the same item
        self.selector.do_callback();
        if choices.len() == 0 {
            self.hide_controls();
            return;
        }
        choices
            .iter()
            .enumerate()
            .for_each(|x| self.selector.add(&(x.0 + 1).to_string()));
        self.show_controls();
        self.selector.select(1);
        self.selector.do_callback();
    }
    /// Event response that adds an element to choice list
    pub fn add_choice(&mut self, choices: &mut Vec<Choice>) {
        // save the old selection if it exists
        let selected = self.selector.value();
        if selected > 0 {
            self.save_choice(choices, Some(selected as usize));
        }

        // create a new entry
        let new_choice = Choice::default();
        choices.push(new_choice);
        self.selector.add(&self.selector.size().to_string());
        // select and load the new entry
        self.show_controls();
        self.selector.select(self.selector.size());
        self.selector.do_callback();
    }
    /// Event response that removes currently selected choice
    ///
    /// It also loads next in line choice into UI if there is any
    pub fn remove_choice(&mut self, choices: &mut Vec<Choice>) {
        let selected = self.selector.value() - 1;
        if selected < 0 {
            return;
        }
        if ask_to_confirm("Are you sure you want to remove the selected choice?") {
            choices.remove(selected as usize);
            self.selector.remove(self.selector.size());

            // loading a new element
            let new_size = self.selector.size();
            if new_size <= selected {
                // removed last element on the list
                if new_size == 0 {
                    // no elements to load, hide the UI
                    self.hide_controls();
                    self.selector.select(0);
                    self.selector.do_callback();
                } else {
                    self.selector.select(new_size);
                    self.selector.do_callback();
                }
            } else {
                self.selector.select(selected);
                self.selector.do_callback();
            }
        }
    }
    /// Event response that saves currently selected element to the list
    pub fn save_choice(&self, choices: &mut Vec<Choice>, index: Option<usize>) {
        // determining the selected element
        let choice = match index {
            Some(v) => match choices.get_mut(v) {
                Some(c) => c,
                None => return,
            },
            None => match self.selector.value() {
                0 => return,
                x => match choices.get_mut(x as usize) {
                    Some(c) => c,
                    None => return,
                },
            },
        };
        // saving the data
        choice.text = self.text.buffer().as_ref().unwrap().text();
        choice.condition = match self.condition.choice() {
            Some(text) => text,
            None => String::new(),
        };
        choice.test = match self.test.choice() {
            Some(text) => text,
            None => String::new(),
        };
        choice.result = match self.result.choice() {
            Some(text) => text,
            None => String::new(),
        };
    }
    /// Event response that loads a choice on index into UI
    pub fn load_choice(&mut self, choices: &Vec<Choice>, index: usize) {
        let choice = match choices.get(index) {
            Some(x) => x,
            None => {
                println!("Choice at index {} is unreachable", index);
                return;
            }
        };
        self.text.buffer().as_mut().unwrap().set_text(&choice.text);
        if choice.condition.len() != 0 {
            let index = self.condition.find_index(&choice.condition);
            self.condition.set_value(index);
        } else {
            self.condition.set_value(-1);
            self.condition.redraw();
        }
        if choice.test.len() != 0 {
            let index = self.test.find_index(&choice.test);
            self.test.set_value(index);
        } else {
            self.test.set_value(-1);
            self.test.redraw();
        }
        if choice.result.len() != 0 {
            let index = self.result.find_index(&choice.result);
            self.result.set_value(index);
        } else {
            self.result.set_value(-1);
            self.result.redraw();
        }
        self.show_controls();
    }
}
