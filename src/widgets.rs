use std::{rc::Rc, cell::RefCell};

use fltk::{
    draw::*,
    prelude::{WidgetBase, WidgetExt},
    widget::Widget, widget_extends,
};

pub struct TextRenderer {
    widget: Widget,
    text: Rc<RefCell<Vec<String>>>,
}

impl TextRenderer {
    pub fn new(x: i32, y: i32, w: i32, h: i32, text: &str) -> Self {
        let mut widget = Widget::new(x, y, w, h, None);
        let text = text.split(&[' ', '\n'][..]).map(|x| x.to_string()).collect();
        let text = Rc::new(RefCell::new(text));

        widget.draw({
            let text : Rc<RefCell<Vec<String>>> = Rc::clone(&text);
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
                if width + cursor_x > w {
                    cursor_x = 0;
                    line += size() + size() / 2;
                }
                draw_text(&word, cursor_x + column_start, line);
                cursor_x += width + whitespace_width;
            }
            pop_clip();
            }});
        Self {
            widget,
            text,
        }
    }
    pub fn set_text(&mut self, text: &str) {
        *self.text.borrow_mut() = text.split(&[' ', '\n'][..]).map(|x| x.to_string()).collect();
        self.widget.redraw();
    }
}
widget_extends!(TextRenderer, Widget, widget);
