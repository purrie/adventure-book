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
        fltk::dialog::alert(0, 0, &format!($text, $($x)*))
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
    let mut win = Window::default().with_size(300, 150).with_label(label);

    Frame::new(50, 10, 200, 20, None).with_label(label);
    let input = Input::new(50, 30, 200, 30, None);

    let mut butt_accept = Button::new(210, 110, 80, 30, "Accept");
    let mut butt_cancel = Button::new(10, 110, 80, 30, "Cancel");

    win.end();
    win.make_modal(true);
    win.show();

    butt_accept.set_callback({
        |x| {
            x.window().unwrap().hide();
        }
    });
    butt_cancel.set_callback({
        let mut input = input.clone();
        move |x| {
            input.set_value("");
            x.window().unwrap().hide();
        }
    });

    while win.shown() {
        app::wait();
    }
    match input.value() {
        x if x == "" => None,
        x => Some(x),
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

    butt_accept.set_callback({
        |x| {
            x.window().unwrap().hide();
        }
    });
    butt_cancel.set_callback({
        let mut input = name.clone();
        move |x| {
            input.set_value("");
            x.window().unwrap().hide();
        }
    });

    while win.shown() {
        app::wait();
    }

    match name.value() {
        x if x=="" => None,
        x => {
            let category = category.value();
            if let Ok(value) = value.value().parse() {
                Some(Record {
                    name: x,
                    category,
                    value,
                })
            } else {
                Some(Record {
                    name: x,
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

    butt_accept.set_callback({
        |x| {
            x.window().unwrap().hide();
        }
    });
    butt_cancel.set_callback({
        let mut input = name.clone();
        move |x| {
            input.set_value("");
            x.window().unwrap().hide();
        }
    });

    while win.shown() {
        app::wait();
    }
    match name.value() {
        x if x == "" => None,
        x => {
            let value = value.value();
            return Some(Name { keyword: x, value });
        }
    }
}
