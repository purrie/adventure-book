use std::{cell::RefCell, rc::Rc};

use fltk::{
    app,
    draw::*,
    enums::{Color, Event, FrameType},
    prelude::{WidgetBase, WidgetExt, BrowserExt},
    widget::Widget,
    widget_extends, browser::SelectBrowser,
};

/// Fancy non-interactive text renderer that allows background
pub struct TextRenderer {
    widget: Widget,
    text: Rc<RefCell<Vec<String>>>,
}

impl TextRenderer {
    /// Creates a new text renderer in specified area with text to render
    pub fn new(x: i32, y: i32, w: i32, h: i32, text: &str) -> Self {
        let mut widget = Widget::new(x, y, w, h, None);
        let text = text
            .split(&[' ', '\n'][..])
            .map(|x| x.to_string())
            .collect();
        let text = Rc::new(RefCell::new(text));

        widget.draw({
            let text: Rc<RefCell<Vec<String>>> = Rc::clone(&text);
            move |r| {
                let x = r.x();
                let y = r.y();
                let w = r.w();
                let h = r.h();
                let mut line = y + size();
                let mut cursor_x = 0;
                let whitespace_width = width(" ") as i32;
                let column_start = x + whitespace_width;

                push_clip(x, y, w, h);
                for word in text.borrow().iter() {
                    let width = width(&word) as i32;
                    if width + cursor_x + whitespace_width > w {
                        cursor_x = 0;
                        line += size() + size() / 2;
                    }
                    draw_text(&word, cursor_x + column_start, line);
                    if word.ends_with("\n") {
                        cursor_x = 0;
                        line += size() + size() / 2;
                    }
                    cursor_x += width;
                }
                pop_clip();
            }
        });
        Self { widget, text }
    }
    /// Sets new text to render
    pub fn set_text(&mut self, text: &str) {
        *self.text.borrow_mut() = text
            .split_inclusive(&[' ', '\n'][..])
            .map(|x| x.to_string())
            .collect();
        if let Some(mut p) = self.widget.parent() {
            p.redraw();
        }
    }
}
widget_extends!(TextRenderer, Widget, widget);

/// Fancy custom selector that doesn't obscure what's behind it in drawing order
pub struct Selector {
    widget: Widget,
    options: Rc<RefCell<Vec<String>>>,
    selected: Rc<RefCell<usize>>,
}

impl Selector {
    /// Creates a new selector widget in specified area
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        let mut widget = Widget::new(x, y, w, h, None);
        let options = Rc::new(RefCell::new(vec![]));
        let selected = Rc::new(RefCell::new(0));
        let highlight = Rc::new(RefCell::new(-1));

        widget.draw({
            let options: Rc<RefCell<Vec<String>>> = Rc::clone(&options);
            let selected: Rc<RefCell<usize>> = Rc::clone(&selected);
            let highlight: Rc<RefCell<i32>> = Rc::clone(&highlight);
            move |wid| {
                let x = wid.x();
                let y = wid.y();
                let w = wid.w();
                let h = wid.h();
                let opt = options.borrow();
                let label_size = wid.label_size();
                let line_size = label_size + label_size / 5;
                let box_size = label_size + label_size / 5;
                let box_loc = box_size - box_size / 5;
                let margin = width(" ") as i32;
                let mut row = y + label_size;
                let sel = selected.borrow();
                let high = highlight.borrow();

                push_clip(x, y, w, h);
                for (i, item) in opt.iter().enumerate() {
                    if *sel == i {
                        draw_box(
                            FrameType::BorderFrame,
                            x,
                            row - box_loc,
                            w,
                            box_size,
                            Color::Black.lighter(),
                        );
                    }
                    if *high == i as i32 {
                        set_draw_color(Color::Blue);
                    } else {
                        set_draw_color(Color::Black);
                    }
                    draw_text(&item, x + margin, row);
                    row += line_size;
                }
                pop_clip();
            }
        });
        widget.handle({
            let options: Rc<RefCell<Vec<String>>> = Rc::clone(&options);
            let selected: Rc<RefCell<usize>> = Rc::clone(&selected);
            let highlight: Rc<RefCell<i32>> = Rc::clone(&highlight);
            move |wid, ev| {
                let cursor_position = app::event_coords();
                let cursor_position = (cursor_position.0 - wid.x(), cursor_position.1 - wid.y());
                let elements = options.borrow().len();
                let label_size = wid.label_size();
                let line_size = label_size + label_size / 5;
                let sel = cursor_position.1 / line_size;

                match ev {
                    Event::Push => {
                        if sel < elements as i32 && *selected.borrow() != sel as usize {
                            *selected.borrow_mut() = sel as usize;
                            wid.parent().unwrap().redraw();
                            wid.do_callback();
                        }
                        true
                    }
                    Event::Enter => true,
                    Event::Leave => {
                        *highlight.borrow_mut() = -1;
                        wid.parent().unwrap().redraw();
                        true
                    }
                    Event::Move => {
                        let high = *highlight.borrow();
                        if sel < elements as i32 {
                            if high != sel {
                                *highlight.borrow_mut() = sel;
                                wid.parent().unwrap().redraw();
                            }
                        } else if high != -1 {
                            *highlight.borrow_mut() = -1;
                            wid.parent().unwrap().redraw();
                        }
                        true
                    }
                    _ => false,
                }
            }
        });

        Self {
            widget,
            options,
            selected,
        }
    }
    /// Clears all the elements of the selector
    pub fn clear(&mut self) {
        self.options.borrow_mut().clear();
    }
    /// Adds a new element to the selector
    pub fn add(&mut self, choice: String) {
        self.options.borrow_mut().push(choice);
    }
    /// Returns a text of selected item, or None if there's nothing selected
    pub fn selected_text(&self) -> Option<String> {
        let arr = self.options.borrow();
        if let Some(text) = arr.get(*self.selected.borrow()) {
            return Some(text.clone());
        }
        None
    }
}
widget_extends!(Selector, Widget, widget);

/// Returns index of item in selector SelectBrowser, or None if it isn't found
pub fn find_item(selector: &SelectBrowser, item: &str) -> Option<i32> {
    let mut n = 1;
    while let Some(t) = selector.text(n) {
        if t == item {
            return Some(n);
        }
        n += 1;
    }
    return None;
}
