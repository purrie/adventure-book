use std::{cell::RefCell, fs::create_dir, path::PathBuf, rc::Rc};

use dirs::data_dir;
use fltk::{
    app,
    browser::SelectBrowser,
    button::Button,
    dialog::NativeFileChooser,
    enums::{Key, Shortcut},
    frame::Frame,
    input::{Input, IntInput},
    menu::Choice,
    prelude::*,
    text::{TextBuffer, TextEditor},
    window::Window,
};

use crate::{
    adventure::{Adventure, Name, Record},
    file::{is_on_adventure_path, paths, save_adventure, PROJECT_PATH_NAME},
};

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

    adventures.iter().for_each(|x| chooser.add_choice(&x.title));
    chooser.add_choice("New");
    chooser.set_value(0);

    let conf = Rc::new(RefCell::new(false));

    butt_accept.set_callback({
        let conf = Rc::clone(&conf);
        move |x| {
            *conf.borrow_mut() = true;
            x.window().unwrap().hide();
        }
    });
    butt_cancel.set_callback({
        move |x| {
            x.window().unwrap().hide();
        }
    });
    butt_accept.set_shortcut(Shortcut::from_key(Key::Enter));
    butt_cancel.set_shortcut(Shortcut::from_key(Key::Escape));

    while win.shown() {
        app::wait();
    }
    if *conf.borrow() {
        Some(chooser.value() as usize)
    } else {
        None
    }
}
pub fn ask_for_new_adventure() -> Option<Adventure> {
    let mut win = Window::default()
        .with_size(500, 250)
        .with_label("Creating Adventure");

    Frame::new(50, 10, 400, 20, "Creating Adventure");
    let mut sel = SelectBrowser::new(10, 35, 230, 200, "Location");
    let mut name = TextEditor::new(260, 50, 230, 40, "name");
    let mut butt_accept = Button::new(410, 210, 80, 30, "Accept");
    let mut butt_cancel = Button::new(250, 210, 80, 30, "Cancel");

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
        move |x| {
            x.window().unwrap().hide();
        }
    });
    butt_accept.set_shortcut(Shortcut::from_key(Key::Enter));
    butt_cancel.set_shortcut(Shortcut::from_key(Key::Escape));

    name.set_buffer(TextBuffer::default());
    sel.add("New Root Location");
    paths!("books")
        .iter()
        .for_each(|x| sel.add(x.to_str().unwrap()));
    sel.set_callback(|x| {
        if x.value() == 1 {
            // opening a dialog to let user choose a new location
            let mut dialog = NativeFileChooser::new(fltk::dialog::FileDialogType::BrowseDir);
            dialog.set_directory(&paths!("books")[0]).unwrap();
            dialog.show();
            let mut dir = dialog.directory();
            // first we have to test if the chosen path is where program will be able to read it
            if is_on_adventure_path(&dir) == false {
                signal_error!(
                    "The selected path isn't inside of any folders expected to hold adventures."
                );
                return;
            }
            // next we have to test if user chosen an folder with no adventures
            dir.push("adventure");
            dir.set_extension("txt");
            if dir.exists() {
                signal_error!("You need to choose a folder that doesn't contain another adventure");
                return;
            }
            // everything seems to be in order, add and select the new path
            dir.set_extension("");
            dir.pop();
            x.add(dir.to_str().unwrap());
            x.select(x.size());
        }
    });

    while win.shown() {
        app::wait();
    }
    if *conf.borrow() {
        if let Some(path) = sel.selected_text() {
            let title = name.buffer().unwrap().text();
            if title.len() == 0 {
                signal_error!("Enter a valid name for the adventure");
                return None;
            }
            let folder = title.trim().to_lowercase().replace(" ", "-").to_string();
            let mut dir = PathBuf::from(path);
            dir.push(folder);
            if dir.exists() {
                dir.push("adventure");
                dir.set_extension("txt");
                if dir.exists() {
                    signal_error!("Selected path already contains adventure with specified name");
                    return None;
                }
                dir.pop();
                dir.set_extension("");
            } else {
                if let Err(e) = create_dir(&dir) {
                    signal_error!("Error creating a directory: {}", e);
                    return None;
                }
            }
            let adventure = Adventure {
                title,
                path: dir.to_str().unwrap().to_string(),
                ..Default::default()
            };
            let sa = adventure.serialize_to_string();
            save_adventure(dir.to_str().unwrap(), sa);
            return Some(adventure);
        } else {
            signal_error!("Choose a location for the adventure");
        }
    }
    return None;
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
    butt_accept.set_shortcut(Shortcut::from_key(Key::Enter));
    butt_cancel.set_shortcut(Shortcut::from_key(Key::Escape));

    while win.shown() {
        app::wait();
    }
    let test = *accept.borrow();
    match test {
        false => None,
        true => Some(input.value()),
    }
}
pub fn ask_for_record(record: Option<&Record>) -> Option<Record> {
    let label = "Insert record data";

    let mut win = Window::default().with_size(300, 170).with_label(label);

    Frame::new(50, 10, 200, 20, None).with_label(label);
    let mut name = Input::new(80, 30, 200, 30, "Keyword");
    let mut category = Input::new(80, 60, 200, 30, "Category");
    let mut value = IntInput::new(80, 90, 200, 30, "Default");
    let mut butt_accept = Button::new(210, 130, 80, 30, "Accept");
    let mut butt_cancel = Button::new(10, 130, 80, 30, "Cancel");

    win.end();
    win.make_modal(true);
    win.show();

    if let Some(rec) = record {
        name.set_value(&rec.name);
        category.set_value(&rec.category);
        value.set_value(&rec.value.to_string());
    }

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
    butt_accept.set_shortcut(Shortcut::from_key(Key::Enter));
    butt_cancel.set_shortcut(Shortcut::from_key(Key::Escape));

    while win.shown() {
        app::wait();
    }
    let test = *accept.borrow();
    let name = name.value();

    match test {
        true if name.len() > 0 => {
            let category = category.value();
            let record = match value.value().parse() {
                Ok(value) => Record {
                    name,
                    category,
                    value,
                },
                Err(_) => Record {
                    name,
                    category,
                    value: 0,
                },
            };
            Some(record)
        }
        _ => None,
    }
}
pub fn ask_for_name(default: Option<&Name>) -> Option<Name> {
    let label = "Input name data";
    let mut win = Window::default().with_size(300, 150).with_label(label);

    Frame::new(50, 10, 200, 20, None).with_label(label);
    let mut name = Input::new(80, 30, 200, 30, "Keyword");
    let mut value = Input::new(80, 60, 200, 30, "Default");

    let mut butt_accept = Button::new(210, 110, 80, 30, "Accept");
    let mut butt_cancel = Button::new(10, 110, 80, 30, "Cancel");

    win.end();
    win.make_modal(true);
    win.show();

    if let Some(val) = default {
        name.set_value(&val.keyword);
        value.set_value(&val.value);
    }

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
    butt_accept.set_shortcut(Shortcut::from_key(Key::Enter));
    butt_cancel.set_shortcut(Shortcut::from_key(Key::Escape));

    while win.shown() {
        app::wait();
    }
    let test = *accept.borrow();
    let keyword = name.value();
    match test {
        true if keyword.len() > 0 => {
            let value = value.value();
            return Some(Name { keyword, value });
        }
        _ => None,
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
    butt_accept.set_shortcut(Shortcut::from_key(Key::Enter));
    butt_cancel.set_shortcut(Shortcut::from_key(Key::Escape));

    while win.shown() {
        app::wait();
    }
    conf.take()
}
pub fn ask_for_choice<'a, T: Iterator>(label: &str, choices: T) -> Option<(i32, String)>
where
    T::Item: Into<&'a String>,
{
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
    butt_accept.set_shortcut(Shortcut::from_key(Key::Enter));
    butt_cancel.set_shortcut(Shortcut::from_key(Key::Escape));

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
        }
        false => None,
    }
}
