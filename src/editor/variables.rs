use fltk::{
    app, button::Button, draw::Rect, frame::Frame, group::Scroll, image::SvgImage, prelude::*, enums::{Align, FrameType},
};
type HandleEvent = fltk::enums::Event;

use crate::{
    adventure::{create_keyword, Name, Record},
    icons::{BIN_ICON, GEAR_ICON},
};

use super::{emit, help, Event, highlight_color};

/// Sets up handle event for drag and drop receiver from variable editor
macro_rules! variable_receiver {
    ($widget:expr) => {
        $widget.handle(|w, ev| {
            if ev == fltk::enums::Event::DndRelease {
                w.paste();
                return true;
            }
            false
        });
    };
}
pub(crate) use variable_receiver;

/// Editor widget for editing records and names
pub struct VariableEditor {
    scroll: Scroll,
    button: Button,
    help: Button,
    children: usize,
    record: bool,
}

impl VariableEditor {
    /// Creates a scrollbar based group for displaying variables in provided area.
    ///
    /// is_record: Determines which callbacks will be triggered on button presses
    pub fn new(area: Rect, is_record: bool) -> Self {
        let x = area.x;
        let y = area.y;
        let w = area.w / 2;
        let h = app::font_size() + 4;

        let x_help = x + w + 10;
        let w_help = app::font_size();
        let h_help = w_help;
        let y_help = y + (h - h_help) / 2;

        let mut button = Button::new(x, y, w, h, None);
        let mut help = Button::new(x_help, y_help, w_help, h_help, "?");
        let scroll = Scroll::new(area.x, area.y + 20, area.w, area.h - 20, None);
        scroll.end();

        help.set_frame(FrameType::RoundUpBox);
        help.set_color(highlight_color!());

        let (s, _r) = app::channel();

        if is_record {
            button.set_label("Add Record");
            button.emit(s.clone(), emit!(Event::AddRecord));
            help.emit(s, help!("variable-record"));
        } else {
            button.set_label("Add Name");
            button.emit(s.clone(), emit!(Event::AddName));
            help.emit(s, help!("variable-name"));
        }

        Self {
            scroll,
            button,
            help,
            children: 0,
            record: is_record,
        }
    }
    /// Removes all children from the editor
    pub fn clear(&mut self) {
        self.scroll.clear();
        self.children = 0;
        self.scroll.redraw();
    }
    /// Adds a new variable to the editor, creating buttons and a label
    ///
    /// variable: Name to display in the editor
    /// extra: Extra part of the label shown in brackets
    /// inserter: Whatever to create a quick insert button for text editors or not
    fn add_line(&mut self, variable: &String, extra: &String, inserter: bool) {
        let child_count = self.children;

        let mut x = self.scroll.x();
        let y = self.scroll.y() + 20 * child_count as i32;
        let mut w = self.scroll.w();
        let h = 20;

        let mut frame = Frame::new(x, y, w, h, None);
        frame.set_frame(fltk::enums::FrameType::EngravedFrame);

        let (sender, _) = app::channel();

        let edit;
        let delete;
        if self.record {
            edit = emit!(Event::EditRecord(variable.clone()));
            delete = emit!(Event::RemoveRecord(variable.clone()));
        } else {
            edit = emit!(Event::EditName(variable.clone()));
            delete = emit!(Event::RemoveName(variable.clone()));
        }

        let bin_icon = SvgImage::from_data(BIN_ICON).unwrap();
        let mut gear_icon = SvgImage::from_data(GEAR_ICON).unwrap();
        gear_icon.scale(15, 15, true, false);

        let mut butt_edit = Button::new(x, y, 20, h, None);
        butt_edit.set_image(Some(gear_icon));
        butt_edit.emit(sender.clone(), edit);

        x += 20;
        w -= 20;

        let mut butt_delete = Button::new(x, y, 20, h, None);
        butt_delete.set_image(Some(bin_icon));
        butt_delete.emit(sender, delete);

        x += 20;
        w -= 20;

        let mut label = Frame::new(x, y, w, h, None);
        label.set_label(variable);

        let mut extra_label = Frame::new(x, y, w, h, None);
        extra_label.set_align(Align::Inside.union(Align::Left));
        extra_label.set_label(&format!("( {} )", extra));

        if inserter {
            label.handle({
                move |l, ev| -> bool {
                    match ev {
                        HandleEvent::Push => {
                            // seems to be needed since pasting seems to paste from both sources no matter what I tried
                            app::copy("");
                            app::copy2(&create_keyword(&l.label()));
                            app::dnd();
                            true
                        }
                        _ => false,
                    }
                }
            });
        }

        self.scroll.add(&frame);
        self.scroll.add(&butt_edit);
        self.scroll.add(&butt_delete);
        self.scroll.add(&label);
        self.scroll.add(&extra_label);

        self.children += 1;
    }
    /// Creates a new line with all necessary controls for the Record
    pub fn add_record(&mut self, record: &Record, inserter: bool) {
        let extra = match record.category.as_str() {
            "" => record.value_as_string(),
            x => format!("{}, {}", x, record.value_as_string()),
        };
        self.add_line(&record.name, &extra, inserter);
    }
    /// Creates a new line with all the necessary controls for the Name
    pub fn add_name(&mut self, name: &Name, inserter: bool) {
        self.add_line(&name.keyword, &name.value, inserter);
    }
    /// Displays the editor
    pub fn show(&mut self) {
        self.button.show();
        self.help.show();
        self.scroll.show();
    }
    /// Hides the editor
    pub fn hide(&mut self) {
        self.button.hide();
        self.help.hide();
        self.scroll.hide();
    }
}
