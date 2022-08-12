use std::{cell::RefCell, rc::Rc};

use fltk::{
    app,
    draw::*,
    enums::{Color, Event, FrameType},
    prelude::{WidgetBase, WidgetExt},
    widget::Widget,
    widget_extends,
};

pub struct TextRenderer {
    widget: Widget,
    text: Rc<RefCell<Vec<String>>>,
}

impl TextRenderer {
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
                    if width + cursor_x > w {
                        cursor_x = 0;
                        line += size() + size() / 2;
                    }
                    draw_text(&word, cursor_x + column_start, line);
                    cursor_x += width + whitespace_width;
                }
                pop_clip();
            }
        });
        Self { widget, text }
    }
    pub fn set_text(&mut self, text: &str) {
        *self.text.borrow_mut() = text
            .split(&[' ', '\n'][..])
            .map(|x| x.to_string())
            .collect();
        self.widget.redraw();
    }
}
widget_extends!(TextRenderer, Widget, widget);

pub struct Selector {
    widget: Widget,
    options: Rc<RefCell<Vec<String>>>,
    selected: Rc<RefCell<usize>>,
}

impl Selector {
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
                let opt = options.borrow();
                let line_size = wid.label_size();
                let margin = width(" ") as i32;
                let mut row = y + line_size;
                let sel = selected.borrow();
                let high = highlight.borrow();

                push_clip(x, y, w, h);
                for (i, item) in opt.iter().enumerate() {
                    if *sel == i {
                        draw_box(
                            FrameType::BorderFrame,
                            x,
                            row - line_size,
                            w,
                            line_size + line_size / 4,
                            Color::Black.lighter(),
                        );
                    }
                    set_draw_color(Color::Black);
                    if *high == i as i32 {
                        set_draw_color(Color::Blue);
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
                let sel = cursor_position.1 / wid.label_size();

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
    pub fn clear(&self) {
        self.options.borrow_mut().clear();
    }
    pub fn add(&self, choice: String) {
        self.options.borrow_mut().push(choice);
    }
    pub fn selected_text(&self) -> Option<String> {
        let arr = self.options.borrow();
        if let Some(text) = arr.get(*self.selected.borrow()) {
            return Some(text.clone());
        }
        None
    }
}
widget_extends!(Selector, Widget, widget);
