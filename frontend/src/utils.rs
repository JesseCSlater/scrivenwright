use std::cell::RefCell;

use ratatui::{
    prelude::*,
    widgets::{block::Title, *},
};

/// A container for managing the logic for a well-formated scroll bar.
#[derive(Debug, PartialEq)]
pub struct ScrollRef {
    /// The number of lines that could be displayed.
    content_length: RefCell<usize>,
    /// The last-known length of the viewport. Used to calculate the position of the bottom-most
    /// element.
    view_length: RefCell<usize>,
    /// The line number of where the viewport starts.
    view_start: RefCell<usize>,
    /// The scrollbar state needed to render a scrollbar.
    state: RefCell<ScrollbarState>,
}

impl ScrollRef {
    pub fn new(content_length: usize, lines: usize) -> Self {
        Self {
            content_length: RefCell::new(content_length),
            view_length: RefCell::new(0),
            view_start: RefCell::new(0),
            state: RefCell::new(ScrollbarState::new(lines)),
        }
    }

    /// Renders the scrollbar into a frame
    pub fn render_scroll(&self, frame: &mut Frame<'_>, bar: Scrollbar<'_>, rect: Rect) {
        self.set_view_length(rect.height as usize);
        frame.render_stateful_widget(bar, rect, &mut self.state.borrow_mut());
    }

    /// Sets the number of lines of content to be displayed
    pub fn set_content_length(&self, lines: usize) {
        *self.content_length.borrow_mut() = lines;
        let state = self.state.borrow_mut().content_length(lines);
        *self.state.borrow_mut() = state;
    }

    /// Sets the length in the scrollbar state.
    fn set_view_length(&self, lines: usize) {
        *self.view_length.borrow_mut() = lines;
        let start = std::cmp::min(
            *self.view_start.borrow(),
            (*self.content_length.borrow()).saturating_sub(lines - 1),
        );
        *self.view_start.borrow_mut() = start;
        let inner_content_length =
            (*self.content_length.borrow()).saturating_sub(*self.view_length.borrow());
        let state = self.state.borrow().content_length(inner_content_length);
        *self.state.borrow_mut() = state;
    }

    /// Gets the scroll index.
    pub fn view_start(&self) -> usize {
        *self.view_start.borrow()
    }

    /// Gets the length of the view port.
    pub fn view_length(&self) -> usize {
        *self.view_length.borrow()
    }

    pub fn set_view_start(&self, view_start: usize) {
        *self.view_start.borrow_mut() = view_start;
        let state = self.state.borrow().position(view_start);
        *self.state.borrow_mut() = state;
    }

    /// Moves the scroll state down.
    pub fn next(&mut self) {
        *self.view_start.get_mut() = self
            .view_start
            .get_mut()
            .checked_add(1)
            .unwrap_or(*self.content_length.borrow());
        self.state.get_mut().next()
    }

    /// Moves the scroll state up.
    pub fn prev(&mut self) {
        *self.view_start.get_mut() = self.view_start.get_mut().saturating_sub(1);
        self.state.get_mut().prev()
    }
}

// Remember the order:
// Span -> Line -> Text (-> Paragraph)


pub fn padded_title(title: String, style: Style) -> Title<'static> {
    vec![
        Span::from(" "),
        Span::styled(" ", style),
        Span::styled(title, style),
        Span::styled(" ", style),
        Span::from(" "),
    ]
    .into()
}