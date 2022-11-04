use fltk::{
    app, button::Button, draw::Rect, frame::Frame, group::Scroll, image::SvgImage, prelude::*, enums::Align,
};
type HandleEvent = fltk::enums::Event;

use crate::{
    adventure::{create_keyword, Name, Record},
    icons::{BIN_ICON, GEAR_ICON},
};

use super::{emit, Event};

/// Sets up handle event for drag and drop receiver from variable editor
macro_rules! variable_receiver {
    ($widget:expr) => {
        $widget.handle(|w, ev| {
            if ev == fltk::enums::Event::DndRelease {
                app::paste_text(w);
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
    children: usize,
    record: bool,
}

impl VariableEditor {
    /// Creates a scrollbar based group for displaying variables in provided area.
    ///
    /// is_record: Determines which callbacks will be triggered on button presses
    pub fn new(area: Rect, is_record: bool) -> Self {
        let mut button = Button::new(area.x, area.y, area.w / 2, 20, None);
        let scroll = Scroll::new(area.x, area.y + 20, area.w, area.h - 20, None);
        scroll.end();

        let (s, _r) = app::channel();

        if is_record {
            button.set_label("Add Record");
            button.emit(s, emit!(Event::AddRecord));
        } else {
            button.set_label("Add Name");
            button.emit(s, emit!(Event::AddName));
        }

        Self {
            scroll,
            button,
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
    pub fn add_line(&mut self, variable: &String, extra: &String, inserter: bool) {
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
                            // copy2 is called because default fltk behavior is retarded
                            // and keeps pasting random things from all buffers when you call paste
                            // even when you specify where it should get the text from
                            // so other buffer needs to be cleared
                            app::copy2("");
                            app::copy(&create_keyword(&l.label()));
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

    pub fn add_record(&mut self, record: &Record, inserter: bool) {
        let extra = match record.category.as_str() {
            "" => record.value_as_string(),
            x => format!("{}, {}", x, record.value_as_string()),
        };
        self.add_line(&record.name, &extra, inserter);
    }
    pub fn add_name(&mut self, name: &Name, inserter: bool) {
        self.add_line(&name.keyword, &name.value, inserter);
    }

    pub fn show(&mut self) {
        self.button.show();
        self.scroll.show();
    }
    pub fn hide(&mut self) {
        self.button.hide();
        self.scroll.hide();
    }
}
