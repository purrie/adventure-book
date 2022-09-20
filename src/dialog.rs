use std::{rc::Rc, cell::RefCell};

use fltk::{
    app,
    button::Button,
    frame::Frame,
    input::{Input, IntInput},
    menu::Choice,
    prelude::*,
    window::Window,
};

use crate::adventure::{Adventure, Name, Record};

macro_rules! signal_error {
    ($text:expr, $( $x:expr ), *) => {
        fltk::dialog::alert(0, 0, &format!($text, $($x),*))
    };
    ($text:expr) => {
         fltk::dialog::alert(0, 0, $text)
    };
}
pub(crate) use signal_error;

pub fn ask_to_choose_adventure(adventures: &Vec<Adventure>) -> Option<usize> {
    let mut win = Window::default()
        .with_size(300, 150)
        .with_label("Choose the Adventure");

    Frame::new(50, 10, 200, 20, "Choose the Adventure");
    let mut chooser = Choice::new(50, 30, 200, 30, None);
    let mut butt_accept = Button::new(210, 110, 80, 30, "Accept");
    let mut butt_cancel = Button::new(10, 110, 80, 30, "Cancel");

    win.end();
    win.make_modal(true);
    win.show();

    for adv in adventures.iter() {
        chooser.add_choice(&adv.title);
    }
    chooser.set_value(0);

    butt_accept.set_callback({
        |x| {
            x.window().unwrap().hide();
        }
    });
    butt_cancel.set_callback({
        let mut chooser = chooser.clone();
        move |x| {
            x.window().unwrap().hide();
            chooser.set_value(-1);
        }
    });
    while win.shown() {
        app::wait();
    }
    if chooser.value() < 0 {
        return None;
    }
    Some(chooser.value() as usize)
}

pub fn ask_for_text(label: &str) -> Option<String> {
    let len = i32::max(fltk::draw::width(label) as i32 + 20, 300);

    let mut win = Window::default().with_size(len, 110).with_label(label);

    Frame::new(20, 10, len - 40, 20, None).with_label(label);
    let input = Input::new(20, 30, len - 40, 30, None);

    let mut butt_accept = Button::new(len - 90, 70, 80, 30, "Accept");
    let mut butt_cancel = Button::new(10, 70, 80, 30, "Cancel");

    win.end();
    win.make_modal(true);
    win.show();

    let accept = Rc::new(RefCell::new(false));

    butt_accept.set_callback({
        let accept = Rc::clone(&accept);
        move |x| {
            *accept.borrow_mut() = true;
            x.window().unwrap().hide();
        }
    });
    butt_cancel.set_callback({
        |x| {
            x.window().unwrap().hide();
        }
    });

    while win.shown() {
        app::wait();
    }
    let test = *accept.borrow();
    match test {
        false => None,
        true => Some(input.value()),
    }
}
pub fn ask_for_record() -> Option<Record> {
    let label = "Insert record data";

    let mut win = Window::default().with_size(300, 170).with_label(label);

    Frame::new(50, 10, 200, 20, None).with_label(label);
    let name = Input::new(80, 30, 200, 30, "Keyword");
    let category = Input::new(80, 60, 200, 30, "Category");
    let value = IntInput::new(80, 90, 200, 30, "Default");
    // TODO see if accept button can be auto pressed on return key
    let mut butt_accept = Button::new(210, 130, 80, 30, "Accept");
    let mut butt_cancel = Button::new(10, 130, 80, 30, "Cancel");

    win.end();
    win.make_modal(true);
    win.show();

    let accept = Rc::new(RefCell::new(false));

    butt_accept.set_callback({
        let accept = Rc::clone(&accept);
        move |x| {
            *accept.borrow_mut() = true;
            x.window().unwrap().hide();
        }
    });
    butt_cancel.set_callback({
        |x| {
            x.window().unwrap().hide();
        }
    });

    while win.shown() {
        app::wait();
    }
    let test = *accept.borrow();
    match test {
        false   => None,
        true    => {
            let name = name.value();
            let category = category.value();
            if let Ok(value) = value.value().parse() {
                Some(Record {
                    name,
                    category,
                    value,
                })
            } else {
                Some(Record {
                    name,
                    category,
                    value: 0,
                })
            }
        }
    }
}
pub fn ask_for_name() -> Option<Name> {
    let label = "Input name data";
    let mut win = Window::default().with_size(300, 150).with_label(label);

    Frame::new(50, 10, 200, 20, None).with_label(label);
    let name = Input::new(80, 30, 200, 30, "Keyword");
    let value = Input::new(80, 60, 200, 30, "Default");

    // TODO see if accept button can be auto pressed on return key
    let mut butt_accept = Button::new(210, 110, 80, 30, "Accept");
    let mut butt_cancel = Button::new(10, 110, 80, 30, "Cancel");

    win.end();
    win.make_modal(true);
    win.show();

    let accept = Rc::new(RefCell::new(false));

    butt_accept.set_callback({
        let accept = Rc::clone(&accept);
        move |x| {
            *accept.borrow_mut() = true;
            x.window().unwrap().hide();
        }
    });
    butt_cancel.set_callback({
        |x| {
            x.window().unwrap().hide();
        }
    });

    while win.shown() {
        app::wait();
    }
    let test = *accept.borrow();
    match test {
        false => None,
        true  => {
            let keyword = name.value();
            let value = value.value();
            return Some(Name { keyword, value });
        }
    }
}
pub fn ask_to_confirm(label: &str) -> bool {
    let len = i32::max(fltk::draw::width(label) as i32 + 20, 300);

    let mut win = Window::default().with_size(len, 100).with_label(label);

    Frame::new(20, 10, len - 40, 20, None).with_label(label);

    let mut butt_accept = Button::new(len - 100, 60, 80, 30, "Yes");
    let mut butt_cancel = Button::new(20, 60, 80, 30, "No");

    win.end();
    win.make_modal(true);
    win.show();

    let conf = Rc::new(RefCell::new(false));

    butt_accept.set_callback({
        let conf = Rc::clone(&conf);
        move |x| {
            *conf.borrow_mut() = true;
            x.window().unwrap().hide();
        }
    });
    butt_cancel.set_callback({
        |x| {
            x.window().unwrap().hide();
        }
    });

    while win.shown() {
        app::wait();
    }
    conf.take()
}
pub fn ask_for_choice<'a, T: Iterator>(label: &str, choices: T) -> Option<(i32, String)> where T::Item: Into<&'a String>  {

    let choices: Vec<&String> = choices.map(|x| x.into()).collect();
    if choices.len() == 0 {
        signal_error!("Nothing to choose from");
        return None;
    }

    let len = i32::max(fltk::draw::width(label) as i32 + 20, 300);

    let mut win = Window::default().with_size(len, 120).with_label(label);

    Frame::new(20, 10, len - 40, 20, None).with_label(label);

    let mut choice = Choice::new(20, 40, len - 40, 30, None);

    let mut butt_accept = Button::new(len - 100, 80, 80, 30, "Accept");
    let mut butt_cancel = Button::new(20, 80, 80, 30, "Cancel");

    win.end();
    win.make_modal(true);
    win.show();

    let conf = Rc::new(RefCell::new(false));

    butt_accept.set_callback({
        let conf = Rc::clone(&conf);
        move |x| {
            *conf.borrow_mut() = true;
            x.window().unwrap().hide();
        }
    });
    butt_cancel.set_callback({
        |x| {
            x.window().unwrap().hide();
        }
    });

    for c in choices.iter() {
        choice.add_choice(c);
    }

    while win.shown() {
        app::wait();
    }
    match conf.take() {
        true => {
            let index = choice.value();
            if index < 0 {
                return None;
            }
            let value = choice.choice().unwrap();
            Some((index, value))
        },
        false => None,
    }
}
