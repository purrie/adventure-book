use fltk::{prelude::*, group::Scroll, app, draw::Rect, button::Button, frame::Frame, image::SvgImage};

use crate::icons::{BIN_ICON, GEAR_ICON};

use super::{emit, Event};


/// Editor widget for editing records and names
pub struct VariableEditor {
    scroll: Scroll,
    children: usize,
    record: bool,
}

impl VariableEditor {
    /// Creates a scrollbar based group for displaying variables in provided area.
    ///
    /// is_record: Determines which callbacks will be triggered on button presses
    pub fn new(area: Rect, is_record: bool) -> Self {
        let mut butt_add = Button::new(area.x, area.y, area.w / 2, 20, None);
        let scroll = Scroll::new(area.x, area.y + 20, area.w, area.h - 20, None);
        scroll.end();

        let (s, _r) = app::channel();

        if is_record {
            butt_add.set_label("Add Record");
            butt_add.emit(s, emit!(Event::AddRecord));
        } else {
            butt_add.set_label("Add Name");
            butt_add.emit(s, emit!(Event::AddName));
        }

        Self {
            scroll,
            children: 0,
            record: is_record,
        }
    }
    /// Removes all children from the editor
    pub fn clear(&mut self) {
        self.scroll.clear();
        self.children = 0;
    }
    /// Adds a new variable to the editor, creating buttons and a label
    ///
    /// variable: Name to display in the editor
    /// inserter: Whatever to create a quick insert button for text editors or not
    pub fn add_record(&mut self, variable: &String, inserter: bool) {
        let child_count = self.children;

        let mut x = self.scroll.x();
        let y = self.scroll.y() + 20 * child_count as i32;
        let mut w = self.scroll.w();
        let h = 20;

        let mut frame = Frame::new(x, y, w, h, None);
        frame.set_frame(fltk::enums::FrameType::EngravedFrame);

        let (sender, _) = app::channel();

        if inserter {
            let mut butt_insert = Button::new(x, y, 20, h, "@<-");
            let ev;
            if self.record {
                ev = emit!(Event::InsertRecord(variable.clone()));
            } else {
                ev = emit!(Event::InsertName(variable.clone()));
            }

            butt_insert.emit(sender.clone(), ev);

            self.scroll.add(&butt_insert);

            x += 20;
            w -= 20;
        }
        let edit;
        let delete;
        if self.record {
            edit = emit!(Event::EditRecord(child_count));
            delete = emit!(Event::RemoveRecord(variable.clone()));
        } else {
            edit = emit!(Event::EditName(child_count));
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

        self.scroll.add(&frame);
        self.scroll.add(&butt_edit);
        self.scroll.add(&butt_delete);
        self.scroll.add(&label);

        self.children += 1;
    }
}
