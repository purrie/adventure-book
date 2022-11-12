use fltk::{
    app, browser::SelectBrowser, button::Button, draw::Rect, group::Group, image::SvgImage,
    prelude::*
};

use crate::{
    icons::{BIN_ICON, GEAR_ICON, STAR_ICON},
    widgets::find_item,
};

use super::{emit, help, Event, highlight_color};

/// Displays the list of files in adventure
///
/// It displays both adventure metadata and each page
/// It also has buttons for adding and removing pages,
/// or deleting the whole adventure, adding a new one or loading existing
pub struct FileList {
    page_list: SelectBrowser,
}

impl FileList {
    /// Creates a new file list within specified area
    pub fn new(area: Rect) -> Self {
        let group = Group::new(area.x, area.y, area.w, area.h, None);

        let font_size = app::font_size();
        let x_column_1 = area.x + 5;
        let x_column_2 = area.x + area.w / 2 + 5;
        let w_whole = area.w - 10;
        let w_column = area.w / 2 - 10;
        let h_line = font_size + font_size / 2;
        let y_first_line = area.y;
        let y_second_line = y_first_line + h_line + 2;
        let y_third_line = y_second_line + h_line + 2;
        let h_selector = area.h - h_line * 3 - 4;
        let y_controls = y_third_line + h_selector;
        let w_controls = font_size;
        let h_controls = font_size;
        let x_add = x_column_1;
        let x_rename = x_add + w_controls;
        let x_help = x_rename + w_controls * 2;
        let x_remove = x_column_1 + w_whole - w_controls;
        let x_start = x_remove - w_controls;

        let mut butt_bac = Button::new(x_column_1, y_first_line, w_column, h_line, "Return");
        let mut butt_sav = Button::new(x_column_2, y_first_line, w_column, h_line, "Save");
        let mut butt_add = Button::new(x_add, y_controls, w_controls, h_controls, "@+");
        let mut butt_rem = Button::new(x_remove, y_controls, w_controls, h_controls, None);
        let mut butt_ren = Button::new(x_rename, y_controls, w_controls, h_controls, None);
        let mut butt_str = Button::new(x_start, y_controls, w_controls, h_controls, None);
        let mut help = Button::new(x_help, y_controls, w_controls, h_controls, "?");
        let mut adventure_meta = Button::new(
            x_column_1,
            y_second_line,
            w_whole,
            h_line,
            "Adventure Metadata",
        );
        let mut page_list =
            SelectBrowser::new(x_column_1, y_third_line, w_whole, h_selector, "Pages");
        group.end();

        let (s, _r) = app::channel();

        let mut gear = SvgImage::from_data(GEAR_ICON).unwrap();
        let mut bin = SvgImage::from_data(BIN_ICON).unwrap();
        let mut star = SvgImage::from_data(STAR_ICON).unwrap();
        gear.scale(w_controls, h_controls, false, true);
        bin.scale(w_controls, h_controls, false, true);
        star.scale(w_controls, h_controls, false, true);

        butt_rem.set_image(Some(bin));
        butt_ren.set_image(Some(gear));
        butt_str.set_image(Some(star));

        butt_bac.emit(s.clone(), crate::game::Event::DisplayMainMenu);
        butt_sav.emit(s.clone(), emit!(Event::Save));
        butt_add.emit(s.clone(), emit!(Event::AddPage));
        butt_rem.emit(s.clone(), emit!(Event::RemovePage));
        butt_ren.emit(s.clone(), emit!(Event::RenamePage));
        help.emit(s.clone(), help!("pages-explorer"));
        help.set_color(highlight_color!());
        help.set_frame(fltk::enums::FrameType::RoundUpBox);
        butt_str.set_callback({
            let fl = page_list.clone();
            let s = s.clone();
            move |_| {
                if let Some(page) = fl.selected_text() {
                    s.send(emit!(Event::SelectStartingPage(page)));
                }
            }
        });
        adventure_meta.emit(s.clone(), emit!(Event::OpenMeta));
        page_list.set_callback(move |x| {
            if let Some(text) = x.selected_text() {
                s.send(emit!(Event::OpenPage(text)));
            }
        });

        Self { page_list }
    }
    /// Fills the selection widget with page names
    pub fn populate_pages(&mut self, pages: &Vec<String>) {
        self.page_list.clear();
        for text in pages {
            self.page_list.add(&text);
        }
    }
    /// Removes selected line from the file list
    pub fn remove_line(&mut self) {
        let selection = self.page_list.value();
        if selection > 0 {
            self.page_list.remove(selection);
        }
    }
    /// Marks a selected line with a star, taking the star away from the previous line
    pub fn mark_line(&mut self, previous: &str, new: &str) {
        if let Some(x) = find_item(&self.page_list, previous) {
            self.page_list.set_icon::<SvgImage>(x, None);
        }
        if let Some(x) = find_item(&self.page_list, new) {
            let mut star = SvgImage::from_data(STAR_ICON).unwrap();
            let font_size = app::font_size();
            star.scale(font_size, font_size, false, true);
            self.page_list.set_icon(x, Some(star));
        }
    }
    /// Renames the selected line to a new name
    pub fn rename_selected(&mut self, new_name: &str) {
        let x = self.page_list.value();
        if x > 0 {
            self.page_list.set_text(x, new_name);
        }
    }
    ///Adds a new line and selects it
    pub fn add_line(&mut self, text: &str) {
        self.page_list.add(text);
        self.page_list.select(self.page_list.size());
    }
}
