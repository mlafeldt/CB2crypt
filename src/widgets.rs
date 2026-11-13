//! The few widgets saudade doesn't ship.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use saudade::{
    Container, Event, EventCtx, Key, MenuBar, MouseButton, NamedKey, Painter, Point, PopupRequest, Rect, Size,
    TextEditor, Theme, Widget,
};

/// Puts a widget into the tree while callbacks elsewhere keep a handle on it.
pub struct Shared<W: ?Sized>(pub Rc<RefCell<W>>);

impl<W: Widget + ?Sized> Widget for Shared<W> {
    fn bounds(&self) -> Rect {
        self.0.borrow().bounds()
    }
    fn paint(&mut self, painter: &mut Painter, theme: &Theme) {
        self.0.borrow_mut().paint(painter, theme);
    }
    fn paint_overlay(&mut self, painter: &mut Painter, theme: &Theme) {
        self.0.borrow_mut().paint_overlay(painter, theme);
    }
    fn event(&mut self, event: &Event, ctx: &mut EventCtx) {
        self.0.borrow_mut().event(event, ctx);
    }
    fn on_cancel(&mut self, ctx: &mut EventCtx) {
        self.0.borrow_mut().on_cancel(ctx);
    }
    fn captures_pointer(&self) -> bool {
        self.0.borrow().captures_pointer()
    }
    fn focusable(&self) -> bool {
        self.0.borrow().focusable()
    }
    fn set_focused(&mut self, focused: bool) {
        self.0.borrow_mut().set_focused(focused);
    }
    fn accepts_accelerators(&self) -> bool {
        self.0.borrow().accepts_accelerators()
    }
    fn layout(&mut self, bounds: Rect) {
        self.0.borrow_mut().layout(bounds);
    }
    fn popup_request(&self) -> Option<PopupRequest> {
        self.0.borrow().popup_request()
    }
    fn collect_popups(&self, out: &mut Vec<PopupRequest>) {
        self.0.borrow().collect_popups(out);
    }
    fn focus_first(&mut self) -> bool {
        self.0.borrow_mut().focus_first()
    }
    fn paints_own_background(&self) -> bool {
        self.0.borrow().paints_own_background()
    }
    fn wants_ticks(&self) -> bool {
        self.0.borrow().wants_ticks()
    }
}

/// The menu bar, opening its menus the way Windows does: a click on a menu
/// shows it with nothing picked out yet.
///
/// saudade's bar highlights the first item once the click is over, and drops
/// the highlight again as soon as the pointer stirs. So the bar is told right
/// away that the pointer moved: to a spot just above it, where there is
/// nothing to pick, since it shrugs off a move that goes nowhere. A menu
/// opened from the keyboard still starts on its first item.
pub struct Menus(pub MenuBar);

impl Widget for Menus {
    fn event(&mut self, event: &Event, ctx: &mut EventCtx) {
        self.0.event(event, ctx);
        let bar = self.0.bounds();
        if let Event::PointerUp {
            pos,
            button: MouseButton::Left,
            ..
        } = event
            && bar.contains(*pos)
        {
            let pos = Point::new(pos.x, bar.y - 1);
            self.0.event(&Event::PointerMove { pos }, ctx);
        }
    }

    fn bounds(&self) -> Rect {
        self.0.bounds()
    }
    fn paint(&mut self, painter: &mut Painter, theme: &Theme) {
        self.0.paint(painter, theme);
    }
    fn paint_overlay(&mut self, painter: &mut Painter, theme: &Theme) {
        self.0.paint_overlay(painter, theme);
    }
    fn captures_pointer(&self) -> bool {
        self.0.captures_pointer()
    }
    fn accepts_accelerators(&self) -> bool {
        self.0.accepts_accelerators()
    }
    fn layout(&mut self, bounds: Rect) {
        self.0.layout(bounds);
    }
    fn popup_request(&self) -> Option<PopupRequest> {
        self.0.popup_request()
    }
}

/// Where a child of a [`Form`] goes, given the form's width and height.
type Place = fn(i32, i32) -> Rect;

/// A dialog whose controls follow the window size.
///
/// The children live in a flat `Container`, so Tab walks them in the order
/// they were added. Nested `Row`s and `Column`s would give the resizing for
/// free, but there Tab hops from row to row instead of from control to
/// control.
pub struct Form {
    inner: Container,
    slots: Vec<(Rc<RefCell<dyn Widget>>, Place)>,
    focus: Rc<Cell<Option<usize>>>,
    tabbing: Rc<Cell<bool>>,
}

impl Form {
    /// `focus` is how the rest of the program moves the keyboard: put the
    /// index of a child there, counted in the order they were added, and the
    /// form focuses that child.
    ///
    /// `tabbing` is how a child tells Tab from any other way of getting the
    /// keyboard: it is set while the form handles a press of Tab.
    pub fn new(focus: Rc<Cell<Option<usize>>>, tabbing: Rc<Cell<bool>>) -> Self {
        Self {
            inner: Container::new(0, 0),
            slots: Vec::new(),
            focus,
            tabbing,
        }
    }

    pub fn add(mut self, widget: impl Widget + 'static, place: Place) -> Self {
        let widget: Rc<RefCell<dyn Widget>> = Rc::new(RefCell::new(widget));
        self.inner.push(Shared(widget.clone()));
        self.slots.push((widget, place));
        self
    }

    fn move_focus(&mut self) {
        if let Some(index) = self.focus.take() {
            self.inner.focus_child(index);
        }
    }
}

impl Widget for Form {
    fn layout(&mut self, bounds: Rect) {
        self.inner.size = Size::new(bounds.w, bounds.h);
        self.inner.layout(bounds);
        for (widget, place) in &self.slots {
            let r = place(bounds.w, bounds.h);
            widget
                .borrow_mut()
                .layout(Rect::new(bounds.x + r.x, bounds.y + r.y, r.w, r.h));
        }
    }

    // A button asks for the focus to move while the form handles its click; a
    // menu item asks from outside the form, and the paint that follows every
    // action is the next time the form gets to look.
    fn event(&mut self, event: &Event, ctx: &mut EventCtx) {
        let tab = Key::Named(NamedKey::Tab);
        self.tabbing
            .set(matches!(event, Event::KeyDown { key, .. } if *key == tab));
        self.inner.event(event, ctx);
        self.tabbing.set(false);
        self.move_focus();
    }
    fn paint(&mut self, painter: &mut Painter, theme: &Theme) {
        self.move_focus();
        self.inner.paint(painter, theme);
    }

    fn bounds(&self) -> Rect {
        self.inner.bounds()
    }
    fn paint_overlay(&mut self, painter: &mut Painter, theme: &Theme) {
        self.inner.paint_overlay(painter, theme);
    }
    fn captures_pointer(&self) -> bool {
        self.inner.captures_pointer()
    }
    fn focusable(&self) -> bool {
        self.inner.focusable()
    }
    fn set_focused(&mut self, focused: bool) {
        self.inner.set_focused(focused);
    }
    fn accepts_accelerators(&self) -> bool {
        self.inner.accepts_accelerators()
    }
    fn popup_request(&self) -> Option<PopupRequest> {
        self.inner.popup_request()
    }
    fn collect_popups(&self, out: &mut Vec<PopupRequest>) {
        self.inner.collect_popups(out);
    }
    fn focus_first(&mut self) -> bool {
        self.inner.focus_first()
    }
    fn wants_ticks(&self) -> bool {
        self.inner.wants_ticks()
    }
}

/// The heading over a text box: its words centred in a thin frame, much as
/// the Windows program had them.
pub struct Caption {
    rect: Rect,
    text: &'static str,
}

impl Caption {
    pub fn new(text: &'static str) -> Self {
        Self {
            rect: Rect::new(0, 0, 0, 0),
            text,
        }
    }
}

impl Widget for Caption {
    fn bounds(&self) -> Rect {
        self.rect
    }
    fn layout(&mut self, bounds: Rect) {
        self.rect = bounds;
    }
    fn paint(&mut self, painter: &mut Painter, theme: &Theme) {
        painter.fill_rect(self.rect, theme.background);
        // Halfway between paper and ink, so the frame shows on either theme
        // without passing for a text field's.
        painter.stroke_rect(self.rect, theme.background.mix(theme.text, 0.5));
        painter.text_centered(self.rect, self.text, theme.font_size, theme.text);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Input,
    Output,
}

/// One of the two text boxes.
///
/// It records which box had the keyboard last, because the Edit menu acts on
/// that one and an open menu takes the focus away before its item fires.
///
/// Tabbing into a box selects all of its text, as tabbing into an edit
/// control of a Windows dialog does. A click, or an action handing the
/// keyboard back, leaves the selection alone.
pub struct Pane {
    pub side: Side,
    pub editor: Shared<TextEditor>,
    pub active: Rc<Cell<Side>>,
    pub tabbing: Rc<Cell<bool>>,
}

impl Widget for Pane {
    fn set_focused(&mut self, focused: bool) {
        if focused {
            self.active.set(self.side);
            if self.tabbing.get() {
                self.editor.0.borrow_mut().select_all();
            }
        }
        self.editor.set_focused(focused);
    }

    fn bounds(&self) -> Rect {
        self.editor.bounds()
    }
    fn paint(&mut self, painter: &mut Painter, theme: &Theme) {
        self.editor.paint(painter, theme);
    }
    fn paint_overlay(&mut self, painter: &mut Painter, theme: &Theme) {
        self.editor.paint_overlay(painter, theme);
    }
    fn event(&mut self, event: &Event, ctx: &mut EventCtx) {
        self.editor.event(event, ctx);
    }
    fn captures_pointer(&self) -> bool {
        self.editor.captures_pointer()
    }
    fn focusable(&self) -> bool {
        self.editor.focusable()
    }
    fn layout(&mut self, bounds: Rect) {
        self.editor.layout(bounds);
    }
    fn collect_popups(&self, out: &mut Vec<PopupRequest>) {
        self.editor.collect_popups(out);
    }
    fn wants_ticks(&self) -> bool {
        self.editor.wants_ticks()
    }
}
