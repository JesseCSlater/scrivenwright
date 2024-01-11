use ratatui::{
    buffer::Cell,
    prelude::{Backend, Rect},
    style::{Color, Modifier},
};
use std::{borrow::Cow, io::Result};
use wasm_bindgen::JsValue;
use yew::{html, Html};

#[derive(Debug, Default)]
pub struct WebTerm {
    buffer: Vec<Vec<Cell>>,
    rendered: Html,
}

impl WebTerm {
    pub fn new() -> Self {
        Self {
            buffer: Self::get_sized_buffer(),
            rendered: Html::default(),
        }
    }

    fn get_sized_buffer() -> Vec<Vec<Cell>> {
        let (width, height) = get_window_size();
        vec![vec![Cell::default(); width as usize]; height as usize]
    }

    pub fn view(&mut self) -> Html {
        self.rendered.clone()
    }

    pub fn render(&mut self) -> Html {
        let mut rows: Vec<Html> = Vec::with_capacity(self.buffer.len());
        for line in self.buffer.clone() {
            let mut row: Vec<Html> = Vec::with_capacity(line.len());
            for cell in line {
                let Cell {
                    fg, bg, modifier, ..
                } = cell;
                let fg = to_css_color(fg).unwrap_or("white".into());
                let bg = to_css_color(bg).unwrap_or("darkslategrey".into());
                let mut style =
                    format!("color: {fg}; background-color: {bg}; white-space: pre-line;");
                extend_css(modifier, &mut style);
                row.push(html! { <td style={ style }> { cell.symbol().to_owned() } </td> });
            }
            rows.push(html! { <tr> { for row.into_iter() } </tr> });
        }
        rows.push(html! { <br/>});
        self.rendered = html! { <table id="the_terminal"> { for rows.into_iter() } </table> };
        self.rendered.clone()
    }

    pub fn resize_buffer(&mut self) {
        let (width, height) = get_window_size();
        if self.buffer.len() != height as usize || self.buffer[0].len() != width as usize {
            self.buffer = Self::get_sized_buffer();
        }
    }
}

impl Backend for WebTerm {
    fn draw<'a, I>(&mut self, content: I) -> Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        for (x, y, cell) in content {
            let y = y as usize;
            let x = x as usize;
            let line = &mut self.buffer[y];
            line.extend(std::iter::repeat_with(Cell::default).take(x.saturating_sub(line.len())));
            line[x] = cell.clone();
        }
        Ok(())
    }

    fn hide_cursor(&mut self) -> Result<()> {
        Ok(())
    }

    fn show_cursor(&mut self) -> Result<()> {
        unimplemented!()
    }

    fn get_cursor(&mut self) -> Result<(u16, u16)> {
        unimplemented!()
    }

    fn set_cursor(&mut self, _x: u16, _y: u16) -> Result<()> {
        unimplemented!()
    }

    fn clear(&mut self) -> Result<()> {
        self.buffer = Self::get_sized_buffer();
        Ok(())
    }

    fn size(&self) -> Result<Rect> {
        let (width, height) = get_window_size();
        Ok(Rect::new(0, 0, width, height))
    }

    fn window_size(&mut self) -> Result<ratatui::backend::WindowSize> {
        unimplemented!()
    }

    fn flush(&mut self) -> Result<()> {
        let _ = self.render();
        Ok(())
    }
}

pub fn get_window_size() -> (u16, u16) {
    let (w, h) = get_raw_window_size();
    ((w / 20) as u16, (h / 44) as u16)
}

fn get_raw_window_size() -> (usize, usize) {
    fn js_val_to_int<I: TryFrom<usize>>(val: JsValue) -> Option<I> {
        val.as_f64().and_then(|i| I::try_from(i as usize).ok())
    }

    web_sys::window()
        .and_then(|s| {
            Option::zip(
                s.inner_width().ok().and_then(js_val_to_int::<usize>),
                s.inner_height().ok().and_then(js_val_to_int::<usize>),
            )
        })
        .unwrap_or((120, 120))
}

fn to_css_color(c: Color) -> Option<Cow<'static, str>> {
    match c {
        Color::Reset => None,
        Color::Black => Some("black".into()),
        Color::Red => Some("red".into()),
        Color::Green => Some("green".into()),
        Color::Yellow => Some("yellow".into()),
        Color::Blue => Some("dodgerblue".into()),
        Color::Magenta => Some("magenta".into()),
        Color::Cyan => Some("cyan".into()),
        Color::Gray => Some("gray".into()),
        Color::DarkGray => Some("darkgray".into()),
        Color::LightRed => Some("#de2b56".into()),
        Color::LightGreen => Some("lightgreen".into()),
        Color::LightYellow => Some("LightGoldenRodYellow".into()),
        Color::LightBlue => Some("LightSkyBlue".into()),
        Color::LightMagenta => Some("#ff00ff".into()),
        Color::LightCyan => Some("lightcyan".into()),
        Color::White => Some("white".into()),
        Color::Rgb(r, g, b) => Some(format!("#{r:X}{g:X}{b:X}").into()),
        Color::Indexed(_) => unimplemented!(),
    }
}

fn extend_css(mods: Modifier, css: &mut String) {
    if mods.contains(Modifier::BOLD) {
        css.push_str(" font-weight: bolder;");
    }
    if mods.contains(Modifier::ITALIC) {
        css.push_str(" font-style: oblique;");
    }
    if mods.contains(Modifier::UNDERLINED) {
        css.push_str(" text-decoration: underline;");
    }
}
