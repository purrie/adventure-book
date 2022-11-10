use std::{cell::RefCell, fs::create_dir, path::PathBuf, rc::Rc};

use dirs::data_dir;
use fltk::{
    app,
    browser::SelectBrowser,
    button::Button,
    enums::{Key, Shortcut},
    frame::Frame,
    input::{Input, IntInput},
    menu::Choice,
    prelude::*,
    text::{TextBuffer, TextEditor},
    window::Window, dialog::NativeFileChooser,
};

use crate::{
    adventure::{Adventure, Name, Record},
    file::{is_on_adventure_path, paths, save_adventure, PROJECT_PATH_NAME},
};

/// Displays a simple alert dialog with provided formatable message
macro_rules! signal_error {
    ($text:expr, $( $x:expr ), *) => {
        fltk::dialog::alert(0, 0, &format!($text, $($x),*))
    };
    ($text:expr) => {
         fltk::dialog::alert(0, 0, $text)
    };
}
pub(crate) use signal_error;

/// Creates and shows a modal dialog that lets user choose an adventure.
///
/// The dialog has additional "New" entry appended that will be returned as equal to provided list's length should it be chosen
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
/// Creates and shows a modal dialog to user that allows for creating a new adventure and choosing its path
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
    // new root location not supported yet
    //sel.add("New Root Location");
    paths!("books")
        .iter()
        .for_each(|x| sel.add(x.to_str().unwrap()));
    sel.set_callback(|x| {
        match x.selected_text() {
            Some(n) if n == "New Root Location" => {
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
                    signal_error!(
                        "You need to choose a folder that doesn't contain another adventure"
                    );
                    return;
                }
                // everything seems to be in order, add and select the new path
                dir.set_extension("");
                dir.pop();
                x.add(dir.to_str().unwrap());
                x.select(x.size());
            }
            _ => {}
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
/// Creates and shows a modal dialog to the user asking for a text input
///
/// The label will be presented above the input.
///
/// # Warning
/// Be warned that an empty string is a valid input for the user
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
/// Creates and shows a modal dialog asking user to put data for record creation into it
///
/// If optional record value is provided then the fields are prefilled with data from the record
///
/// # Warning
/// While the function return will always be a valid record, it still needs to be tested for duplicate keyword
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
/// Asks user to input data to create a name
///
/// # Warning
/// While the function ensures the returned name is valid, it still needs to be tested for duplicate keyword
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
/// Presents a simple modal dialog asking to confirm a choice
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
/// Presents a dialog with a dropdown populated with the data from the provided iterator
///
/// Returns an index of chosen element and its name
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
