//! The About box: the padlock, what the program is, and where it lives.

use std::process::Command;

use saudade::{Button, Color, Cursor, Event, EventCtx, MouseButton, Painter, Rect, Size, Theme, Widget};

use crate::ICON;

pub const SIZE: Size = Size::new(340, 132);

const HOMEPAGE: &str = "https://github.com/mlafeldt/CB2crypt";
const TEXT_X: i32 = 64;
const LINK_Y: i32 = 62;
const LINE_H: i32 = 16;

pub struct About {
    rect: Rect,
    ok: Button,
    /// Where the homepage was painted last; what a click is tested against.
    link: Rect,
}

impl About {
    pub fn new() -> Self {
        Self {
            rect: Rect::new(0, 0, SIZE.w, SIZE.h),
            ok: Button::new(Rect::new(0, 0, 0, 0), "OK")
                .default(true)
                .on_click(|cx| cx.request_dismiss()),
            link: Rect::new(0, 0, 0, 0),
        }
    }
}

impl Widget for About {
    fn bounds(&self) -> Rect {
        self.rect
    }

    fn layout(&mut self, bounds: Rect) {
        self.rect = bounds;
        self.ok
            .layout(Rect::new(bounds.x + (bounds.w - 76) / 2, bounds.bottom() - 34, 76, 24));
    }

    fn paint(&mut self, painter: &mut Painter, theme: &Theme) {
        let Rect { x, y, .. } = self.rect;
        let size = theme.font_size;
        painter.draw_svg(&ICON, Rect::new(x + 16, y + 14, 32, 32));

        let version = env!("CARGO_PKG_VERSION");
        let lines = [
            format!("CodeBreaker PS2 Crypto Program v{version}"),
            "Copyright (C) 2006-2026 misfire".to_string(),
        ];
        for (line, text) in lines.iter().enumerate() {
            painter.text(x + TEXT_X, y + 14 + line as i32 * LINE_H, text, size, theme.text);
        }

        // The homepage, underlined in the selection colour like the e-mail
        // address the Windows program had here; lightened on a dark theme,
        // where navy all but vanishes.
        let luma = |c: Color| c.red() as u32 + c.green() as u32 + c.blue() as u32;
        let dark = luma(theme.background) < luma(theme.text);
        let ink = theme.highlight_bg.mix(theme.text, if dark { 0.5 } else { 0.0 });
        let label = HOMEPAGE.trim_start_matches("https://");
        let width = painter.measure_text(label, size).w;
        self.link = Rect::new(x + TEXT_X, y + LINK_Y, width, LINE_H);
        painter.text(self.link.x, self.link.y, label, size, ink);
        painter.h_line(self.link.x, self.link.bottom() - 2, width, ink);

        self.ok.paint(painter, theme);
    }

    fn event(&mut self, event: &Event, ctx: &mut EventCtx) {
        match event {
            Event::PointerMove { pos } if self.link.contains(*pos) => ctx.set_cursor(Cursor::Hand),
            Event::PointerDown {
                pos,
                button: MouseButton::Left,
                ..
            } if self.link.contains(*pos) => {
                open(HOMEPAGE);
                return;
            }
            _ => {}
        }
        self.ok.event(event, ctx);
    }

    fn captures_pointer(&self) -> bool {
        self.ok.captures_pointer()
    }
    fn focusable(&self) -> bool {
        true
    }
    fn set_focused(&mut self, focused: bool) {
        self.ok.set_focused(focused);
    }
}

/// Hands a URL to whatever the system opens URLs with. If that fails there
/// is nothing useful to say: the address is on screen.
fn open(url: &str) {
    let mut command = if cfg!(target_os = "macos") {
        Command::new("open")
    } else if cfg!(windows) {
        let mut command = Command::new("cmd");
        command.args(["/c", "start", ""]);
        command
    } else {
        Command::new("xdg-open")
    };
    let _ = command.arg(url).spawn();
}
