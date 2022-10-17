use fltk::{
    app, browser::SelectBrowser, button::Button, draw::Rect, frame::Frame, group::Group, prelude::*,
};

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
        let mut butt_bac = Button::new(area.x + 5, area.y, area.w / 2 - 5, 20, "Return");
        let mut butt_sav =
            Button::new(area.x + area.w / 2 + 5, area.y, area.w / 2 - 10, 20, "Save");
        let mut butt_add = Button::new(area.x + 5, area.y + 25, area.w / 2 - 5, 20, "Add Page");
        let mut butt_rem = Button::new(
            area.x + area.w / 2 + 5,
            area.y + 25,
            area.w / 2 - 10,
            20,
            "Remove Page",
        );
        let mut adventure_meta = Button::new(
            area.x + 5,
            area.y + 50,
            area.w - 10,
            20,
            "Adventure Metadata",
        );
        Frame::new(area.x, area.y + 75, area.w, 20, "Pages");
        let mut page_list = SelectBrowser::new(area.x, area.y + 95, area.w, area.h - 95, None);
        group.end();

        let (s, _r) = app::channel();

        butt_bac.emit(s.clone(), crate::game::Event::DisplayMainMenu);
        butt_sav.emit(s.clone(), emit!(Event::Save));
        butt_add.emit(s.clone(), emit!(Event::AddPage));
        butt_rem.emit(s.clone(), emit!(Event::RemovePage));
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
    pub fn add_line(&mut self, text: &str) {
        self.page_list.add(text);
        self.page_list.select(self.page_list.size());
    }
}
