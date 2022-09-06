use fltk::{app, button::Button, menu::Choice, prelude::*, window::Window, frame::Frame, input::Input};

use crate::adventure::Adventure;

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
    let mut win = Window::default()
        .with_size(300, 150)
        .with_label(label);

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
