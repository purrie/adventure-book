use fltk::{
    app, browser::SelectBrowser, button::Button, draw::Rect, group::Group, prelude::*, image::SvgImage,
};

use crate::icons::{GEAR_ICON, BIN_ICON};

use super::{emit, Event};

/// Displays the list of files in adventure
///
/// It displays both adventure metadata and each page
/// It also has buttons for adding and removing pages,
/// or deleting the whole adventure, adding a new one or loading existing
pub struct FileList {
    page_list: SelectBrowser,
}

impl FileList {
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
        let x_remove = x_column_1 + w_whole - w_controls;

        let mut butt_bac = Button::new(x_column_1, y_first_line, w_column, h_line, "Return");
        let mut butt_sav = Button::new(x_column_2, y_first_line, w_column, h_line, "Save");
        let mut butt_add = Button::new(x_add, y_controls, w_controls, h_controls, "@+");
        let mut butt_rem = Button::new(x_remove, y_controls, w_controls, h_controls, None);
        let mut butt_ren = Button::new(x_rename, y_controls, w_controls, h_controls, None);
        let mut adventure_meta = Button::new(
            x_column_1,
            y_second_line,
            w_whole,
            h_line,
            "Adventure Metadata",
        );
        let mut page_list = SelectBrowser::new(x_column_1, y_third_line, w_whole, h_selector, "Pages");
        group.end();

        let (s, _r) = app::channel();

        let mut gear = SvgImage::from_data(GEAR_ICON).unwrap();
        let mut bin = SvgImage::from_data(BIN_ICON).unwrap();
        gear.scale(w_controls, h_controls, false, true);
        bin.scale(w_controls, h_controls, false, true);

        butt_rem.set_image(Some(bin));
        butt_ren.set_image(Some(gear));

        butt_bac.emit(s.clone(), crate::game::Event::DisplayMainMenu);
        butt_sav.emit(s.clone(), emit!(Event::Save));
        butt_add.emit(s.clone(), emit!(Event::AddPage));
        butt_rem.emit(s.clone(), emit!(Event::RemovePage));
        butt_ren.emit(s.clone(), emit!(Event::RenamePage));
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
    pub fn remove_line(&mut self) {
        let selection = self.page_list.value();
        if selection > 0 {
            self.page_list.remove(selection);
        }
    }
    pub fn rename_selected(&mut self, new_name: &str) {
        let x = self.page_list.value();
        if x > 0 {
            self.page_list.set_text(x, new_name);
        }
    }
    pub fn add_line(&mut self, text: &str) {
        self.page_list.add(text);
        self.page_list.select(self.page_list.size());
    }
}
