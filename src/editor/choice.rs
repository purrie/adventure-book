use std::{rc::Rc, cell::RefCell};

use fltk::{prelude::*, app, browser::SelectBrowser, text::{TextEditor, TextBuffer}, draw::Rect, group::Group, frame::Frame};

use crate::adventure::Choice;


/// Editor for customizing choices for a page
///
/// Displays a list of choices for the page
/// It has a text editor for the choice text, and drop downs for choosing condition, test and result for each choice
pub struct ChoiceEditor {
    selector: SelectBrowser,
    text: TextEditor,
    condition: fltk::menu::Choice,
    test: Rc<RefCell<fltk::menu::Choice>>,
    result: Rc<RefCell<fltk::menu::Choice>>,
    last_selected: Rc<RefCell<i32>>,
}

impl ChoiceEditor {
    pub fn new(area: Rect) -> Self {
        use fltk::menu::Choice;
        let font_size = app::font_size();

        let group = Group::new(area.x, area.y, area.w, area.h, "Choices");

        let x_selector = area.x;
        let y_selector = area.y;
        let w_selector = area.w / 3;
        let h_selector = area.h - font_size;

        let margin_menu = 20;
        let x_menu = area.x + w_selector + margin_menu;
        let w_menu = area.w - w_selector - margin_menu * 2;
        let h_menu = font_size + 2;
        let y_menu_condition = area.y + font_size;
        let y_menu_test = y_menu_condition + h_menu * 2;
        let y_menu_result = y_menu_test + h_menu * 2;

        let x_text = x_menu;
        let y_text = y_menu_result + h_menu * 2;
        let w_text = w_menu;
        let h_text = h_menu;

        let selector = SelectBrowser::new(
            x_selector,
            y_selector,
            w_selector,
            h_selector,
            "Choices in this page",
        );
        let mut text = TextEditor::new(x_text, y_text, w_text, h_text, "Choice Text");
        Frame::new(
            x_menu,
            y_menu_condition - font_size,
            w_menu,
            h_menu,
            "Condition",
        );
        let condition = Choice::new(x_menu, y_menu_condition, w_menu, h_menu, None);
        Frame::new(x_menu, y_menu_test - font_size, w_menu, h_menu, "Test");
        let test = Choice::new(x_menu, y_menu_test, w_menu, h_menu, None);
        Frame::new(x_menu, y_menu_result - font_size, w_menu, h_menu, "Result");
        let result = Choice::new(x_menu, y_menu_result, w_menu, h_menu, None);
        group.end();

        text.set_buffer(TextBuffer::default());
        let last_selected = Rc::new(RefCell::new(0));

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
            last_selected,
        }
    }
    fn save_choice(&self, choice: &mut Choice) {
        choice.text = self.text.buffer().as_ref().unwrap().text();
        choice.condition = match self.condition.choice() {
            Some(text) => text,
            None => String::new(),
        };
        choice.test = match self.test.borrow().choice() {
            Some(text) => text,
            None => String::new(),
        };
        choice.result = match self.result.borrow().choice() {
            Some(text) => text,
            None => String::new(),
        };
    }
    fn load_choice(&mut self, choice: &Choice) {
        self.text.buffer().as_mut().unwrap().set_text(&choice.text);
        if choice.condition.len() != 0 {
            let index = self.condition.find_index(&choice.condition);
            self.condition.set_value(index);
        } else {
            self.condition.set_value(-1);
        }
        if choice.test.len() != 0 {
            let index = self.test.borrow().find_index(&choice.test);
            self.test.borrow_mut().set_value(index);
        } else {
            self.test.borrow_mut().set_value(-1);
        }
        if choice.result.len() != 0 {
            let index = self.result.borrow().find_index(&choice.result);
            self.result.borrow_mut().set_value(index);
        } else {
            self.result.borrow_mut().set_value(-1);
        }
    }
}
