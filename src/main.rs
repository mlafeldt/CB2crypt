//! CB2crypt, the CodeBreaker PS2 crypto program.
//!
//! A port of the Windows program from 2006: same dialog, same menus, same
//! parser. The crypto comes from the `codebreaker` crate.

mod about;
mod parser;
mod widgets;

use std::cell::{Cell, RefCell};
use std::fs;
use std::path::Path;
use std::rc::Rc;

use saudade::{
    Accel, App, Button, Checkbox, Column, Dialog, EventCtx, FileDialog, FileFilter, Menu, MenuBar, MenuItem, Modal,
    NamedKey, Rect, SvgImage, TextEditor, Theme, Widget, WindowConfig, include_svg,
};

use about::About;
use parser::{Mode, Options};
use widgets::{Caption, Form, Menus, Pane, Shared, Side};

const APP_NAME: &str = "CB2crypt";
const ICON: SvgImage = include_svg!("assets/lock.svg");

const WINDOW_W: i32 = 720;
const WINDOW_H: i32 = 480;
const MENU_H: i32 = 20;

// The dialog below the menu bar: a caption row, the two text boxes, and a
// row of buttons.
const MARGIN: i32 = 10;
const GAP: i32 = 10;
const BOXES_Y: i32 = 36;
const BUTTON_H: i32 = 24;
const BUTTON_W: i32 = 76;
const CLEAR_W: i32 = 60;
const NOWHERE: Rect = Rect::new(0, 0, 0, 0);

/// Where the input box comes in the form: after the two captions.
const INPUT_BOX: usize = 2;

fn main() {
    // Windows 3.1 had no dark mode, so it takes asking for: `--dark`, or
    // saudade's own SAUDADE_THEME, which can also follow the system.
    let dark = std::env::args().skip(1).any(|arg| arg == "--dark");
    if dark || std::env::var_os("SAUDADE_THEME").is_none() {
        // SAFETY: no other thread exists yet.
        unsafe { std::env::set_var("SAUDADE_THEME", if dark { "dark" } else { "light" }) };
    }

    let (config, root, _) = window();
    App::new(config, root)
        .with_theme(theme(Theme::windows_31()))
        .with_dark_theme(theme(Theme::windows_31_dark()))
        .run();
}

/// A saudade theme with a paler scrollbar track, as the Windows program had
/// it, so the bars don't outweigh the codes.
fn theme(base: Theme) -> Theme {
    Theme {
        scroll_track: base.highlight,
        ..base
    }
}

/// Everything the menus and buttons act on.
#[derive(Clone)]
struct Ui {
    input: Rc<RefCell<TextEditor>>,
    output: Rc<RefCell<TextEditor>>,
    /// The text box that had the keyboard last.
    active: Rc<Cell<Side>>,
    /// Set to a child of the form to give it the keyboard.
    focus: Rc<Cell<Option<usize>>>,
    common_v7: Rc<RefCell<Checkbox>>,
    blank_line: Rc<Cell<bool>>,
    dialog: Rc<RefCell<Dialog>>,
    files: Rc<RefCell<FileDialog>>,
    about: Rc<RefCell<Modal>>,
}

impl Ui {
    fn new() -> Self {
        let common_v7 = Checkbox::new(NOWHERE, "Use common V7 encryption").on_toggle(|cx, _| cx.request_paint());

        let mut files = FileDialog::new().with_filters(vec![
            FileFilter::new("Text Files (*.txt)", ["*.txt"]),
            FileFilter::all_files(),
        ]);
        if let Some(home) = std::env::home_dir() {
            files.set_directory(home);
        }

        // Like the Windows edit controls, the boxes don't wrap their lines.
        // Like a Windows edit control, a box shows its caret only while it
        // has the keyboard.
        let text_box = || {
            TextEditor::new(NOWHERE)
                .with_horizontal_scrollbar(true)
                .with_unfocused_caret(false)
        };

        Self {
            input: Rc::new(RefCell::new(text_box())),
            output: Rc::new(RefCell::new(text_box().with_read_only(true))),
            active: Rc::new(Cell::new(Side::Input)),
            focus: Rc::new(Cell::new(None)),
            common_v7: Rc::new(RefCell::new(common_v7)),
            blank_line: Rc::new(Cell::new(false)),
            dialog: Rc::new(RefCell::new(Dialog::new())),
            files: Rc::new(RefCell::new(files)),
            about: Rc::new(RefCell::new(Modal::new())),
        }
    }

    /// Wraps an action for a menu item.
    fn on(&self, action: fn(&Ui, &mut EventCtx)) -> impl FnMut(&mut EventCtx) + 'static {
        let ui = self.clone();
        move |cx| {
            action(&ui, cx);
            cx.request_paint();
        }
    }

    /// Wraps an action for a button. As in the Windows program, the keyboard
    /// goes back to the input box afterwards.
    fn on_button(&self, action: fn(&Ui)) -> impl FnMut(&mut EventCtx) + 'static {
        let ui = self.clone();
        move |cx| {
            action(&ui);
            ui.focus_input();
            cx.request_paint();
        }
    }

    /// Wraps a condition for a menu item.
    fn when(&self, condition: fn(&Ui) -> bool) -> impl Fn() -> bool + 'static {
        let ui = self.clone();
        move || condition(&ui)
    }

    fn pane(&self, side: Side) -> Pane {
        Pane {
            side,
            editor: Shared(self.editor(side).clone()),
            active: self.active.clone(),
        }
    }

    fn editor(&self, side: Side) -> &Rc<RefCell<TextEditor>> {
        match side {
            Side::Input => &self.input,
            Side::Output => &self.output,
        }
    }

    fn active_editor(&self) -> &Rc<RefCell<TextEditor>> {
        self.editor(self.active.get())
    }

    fn focus_input(&self) {
        self.focus.set(Some(INPUT_BOX));
    }

    fn options(&self) -> Options {
        Options {
            common_v7: self.common_v7.borrow().is_checked(),
            blank_line: self.blank_line.get(),
        }
    }

    // What the menus grey their items by. The Edit menu acts on the box used
    // last, and only the input box can be changed.

    fn has_input(&self) -> bool {
        !self.input.borrow().text().trim().is_empty()
    }

    fn has_output(&self) -> bool {
        !self.output.borrow().is_empty()
    }

    fn input_is_active(&self) -> bool {
        self.active.get() == Side::Input
    }

    fn can_undo(&self) -> bool {
        self.input_is_active() && self.input.borrow().can_undo()
    }

    fn can_copy(&self) -> bool {
        self.active_editor().borrow().has_selection()
    }

    fn can_cut(&self) -> bool {
        self.input_is_active() && self.can_copy()
    }

    fn can_select(&self) -> bool {
        !self.active_editor().borrow().is_empty()
    }

    fn crypt(&self, mode: Mode) {
        if !self.has_input() {
            self.dialog
                .borrow_mut()
                .show_info(APP_NAME, "You haven't entered any codes.");
            return;
        }
        let lines = parser::process(&self.input.borrow().text(), mode, self.options());
        self.output.borrow_mut().set_text(&lines.join("\n"));
    }

    fn copy_output_to_input(&self) {
        let text = self.output.borrow().text();
        self.input.borrow_mut().set_text(&text);
        self.focus_input();
    }

    fn load(&self) {
        let ui = self.clone();
        self.files.borrow_mut().show_open(move |cx, path| {
            match fs::read(path) {
                Ok(bytes) => {
                    let text = String::from_utf8_lossy(&bytes).replace("\r\n", "\n");
                    ui.input.borrow_mut().set_text(&text);
                    ui.focus_input();
                }
                Err(_) => ui
                    .dialog
                    .borrow_mut()
                    .show_error(APP_NAME, format!("Can't read from file \"{}\"", path.display())),
            }
            cx.request_paint();
        });
    }

    fn save(&self) {
        let ui = self.clone();
        self.files.borrow_mut().show_save("codes.txt", move |cx, path| {
            // The file dialog takes any name, so asking is left to us.
            if path.exists() {
                let (ui, path) = (ui.clone(), path.to_path_buf());
                let question = format!("{} already exists.\nDo you want to replace it?", path.display());
                ui.clone()
                    .dialog
                    .borrow_mut()
                    .show_confirm(APP_NAME, question, "Replace", move |cx| {
                        ui.write(&path);
                        cx.request_paint();
                    });
            } else {
                ui.write(path);
            }
            cx.request_paint();
        });
    }

    fn write(&self, path: &Path) {
        if fs::write(path, self.output.borrow().text() + "\n").is_err() {
            self.dialog
                .borrow_mut()
                .show_error(APP_NAME, format!("Can't write to file \"{}\"", path.display()));
        }
    }

    fn about(&self) {
        self.about
            .borrow_mut()
            .show(format!("About {APP_NAME}"), about::SIZE, Box::new(About::new()));
    }
}

/// The window and the widget tree inside it.
fn window() -> (WindowConfig, impl Widget, Ui) {
    let ui = Ui::new();

    // Tab walks the controls in this order, so the boxes come first.
    let form = Form::new(ui.focus.clone())
        .add(Caption::new("Codes to decrypt/encrypt"), |w, _| caption(Side::Input, w))
        .add(Caption::new("Codes decrypted/encrypted"), |w, _| {
            caption(Side::Output, w)
        })
        .add(ui.pane(Side::Input), |w, h| text_box(Side::Input, w, h))
        .add(ui.pane(Side::Output), |w, h| text_box(Side::Output, w, h))
        .add(
            Button::new(NOWHERE, "Decrypt").on_click(ui.on_button(|ui| ui.crypt(Mode::Decrypt))),
            |_, h| Rect::new(MARGIN, buttons_y(h), BUTTON_W, BUTTON_H),
        )
        .add(
            Button::new(NOWHERE, "Encrypt").on_click(ui.on_button(|ui| ui.crypt(Mode::Encrypt))),
            |_, h| Rect::new(MARGIN + BUTTON_W + 6, buttons_y(h), BUTTON_W, BUTTON_H),
        )
        .add(
            Button::new(NOWHERE, "Reformat only").on_click(ui.on_button(|ui| ui.crypt(Mode::Reformat))),
            |_, h| Rect::new(MARGIN + 2 * (BUTTON_W + 6), buttons_y(h), 112, BUTTON_H),
        )
        .add(Shared(ui.common_v7.clone()), |w, h| {
            let (x, width) = column(Side::Output, w);
            Rect::new(x, buttons_y(h) + 4, width - BUTTON_W - 8, 16)
        })
        .add(Button::new(NOWHERE, "Close").on_click(|cx| cx.close()), |w, h| {
            Rect::new(w - MARGIN - BUTTON_W, buttons_y(h), BUTTON_W, BUTTON_H)
        })
        .add(
            Button::new(NOWHERE, "Clear").on_click(ui.on_button(|ui| ui.input.borrow_mut().set_text(""))),
            |w, _| clear_button(Side::Input, w),
        )
        .add(
            Button::new(NOWHERE, "Clear").on_click(ui.on_button(|ui| ui.output.borrow_mut().set_text(""))),
            |w, _| clear_button(Side::Output, w),
        );

    let root = Column::new()
        .add_fixed(Menus(menu_bar(&ui)), MENU_H)
        .add_fill(form)
        .add_overlay(Shared(ui.dialog.clone()))
        .add_overlay(Shared(ui.files.clone()))
        .add_overlay(Shared(ui.about.clone()));

    let title = format!("{APP_NAME} v{}", env!("CARGO_PKG_VERSION"));
    let config = WindowConfig::new(title, WINDOW_W, WINDOW_H)
        .resizable(true)
        .min_size(600, 300)
        .icon(ICON);

    (config, root, ui)
}

fn menu_bar(ui: &Ui) -> MenuBar {
    let item = |label: &str, action: fn(&Ui, &mut EventCtx)| MenuItem::action(label, ui.on(action));
    // An item of the Edit menu, which acts on the box used last.
    let edit = |label: &str, action: fn(&mut TextEditor)| {
        let ui = ui.clone();
        MenuItem::action(label, move |cx| {
            action(&mut ui.active_editor().borrow_mut());
            cx.request_paint();
        })
    };

    MenuBar::new(Rect::new(0, 0, WINDOW_W, MENU_H))
        .add_menu(Menu::new(
            "&File",
            vec![
                item("&Load Text File...", |ui, _| ui.load()).with_accel(Accel::primary('o')),
                item("&Save As Text File...", |ui, _| ui.save())
                    .with_accel(Accel::primary('s'))
                    .with_enabled(ui.when(Ui::has_output)),
                MenuItem::separator(),
                // Esc in the Windows program, where it is one stray key
                // away from losing the codes just typed.
                item("E&xit", |_, cx| cx.close()).with_accel(Accel::primary('q')),
            ],
        ))
        // A greyed item's shortcut is not taken, so the keys below still do
        // in a text box what they always do: with nothing selected, Delete
        // removes the character after the caret.
        .add_menu(Menu::new(
            "&Edit",
            vec![
                edit("&Undo", TextEditor::undo)
                    .with_accel(Accel::primary('z'))
                    .with_enabled(ui.when(Ui::can_undo)),
                MenuItem::separator(),
                edit("Cu&t", TextEditor::cut)
                    .with_accel(Accel::primary('x'))
                    .with_enabled(ui.when(Ui::can_cut)),
                edit("&Copy", TextEditor::copy)
                    .with_accel(Accel::primary('c'))
                    .with_enabled(ui.when(Ui::can_copy)),
                edit("&Paste", TextEditor::paste)
                    .with_accel(Accel::primary('v'))
                    .with_enabled(ui.when(Ui::input_is_active)),
                edit("&Delete", TextEditor::delete)
                    .with_accel(Accel::new(NamedKey::Delete))
                    .with_enabled(ui.when(Ui::can_cut)),
                MenuItem::separator(),
                edit("Select &All", TextEditor::select_all)
                    .with_accel(Accel::primary('a'))
                    .with_enabled(ui.when(Ui::can_select)),
                edit("Clea&r", |editor| editor.set_text("")).with_enabled(ui.when(Ui::can_select)),
                MenuItem::separator(),
                item("Copy Output To &Input", |ui, _| ui.copy_output_to_input()).with_enabled(ui.when(Ui::has_output)),
            ],
        ))
        .add_menu(Menu::new(
            "&Codes",
            vec![
                // Never greyed, unlike in the Windows program: a greyed item
                // takes its shortcut with it, and with nothing to convert the
                // shortcut should say so, as the buttons do.
                item("&Decrypt", |ui, _| ui.crypt(Mode::Decrypt)).with_accel(Accel::primary('d')),
                item("&Encrypt", |ui, _| ui.crypt(Mode::Encrypt)).with_accel(Accel::primary('e')),
                item("&Reformat Only", |ui, _| ui.crypt(Mode::Reformat)).with_accel(Accel::primary('r')),
            ],
        ))
        .add_menu(Menu::new(
            "&Options",
            vec![
                item("Use Common &V7 Encryption", |ui, _| {
                    let checked = ui.common_v7.borrow().is_checked();
                    ui.common_v7.borrow_mut().set_checked(!checked);
                })
                .with_checked(ui.when(|ui| ui.common_v7.borrow().is_checked())),
                item("Add &Blank Line Between Codes", |ui, _| {
                    ui.blank_line.set(!ui.blank_line.get());
                })
                .with_checked(ui.when(|ui| ui.blank_line.get())),
            ],
        ))
        .add_menu(Menu::new(
            "?",
            vec![item("&About...", |ui, _| ui.about()).with_accel(Accel::new(NamedKey::F1))],
        ))
}

/// Left edge and width of one half of the dialog.
fn column(side: Side, w: i32) -> (i32, i32) {
    let width = (w - 2 * MARGIN - GAP) / 2;
    match side {
        Side::Input => (MARGIN, width),
        Side::Output => (w - MARGIN - width, width),
    }
}

fn caption(side: Side, w: i32) -> Rect {
    let (x, width) = column(side, w);
    Rect::new(x, 10, width - CLEAR_W - 4, 20)
}

fn clear_button(side: Side, w: i32) -> Rect {
    let (x, width) = column(side, w);
    Rect::new(x + width - CLEAR_W, 10, CLEAR_W, 20)
}

fn text_box(side: Side, w: i32, h: i32) -> Rect {
    let (x, width) = column(side, w);
    Rect::new(x, BOXES_Y, width, buttons_y(h) - 10 - BOXES_Y)
}

fn buttons_y(h: i32) -> i32 {
    h - MARGIN - BUTTON_H
}

#[cfg(test)]
mod tests {
    use saudade::mock::MockBackend;
    use saudade::{Event, Key, Modifiers, MouseButton, Point};

    use super::*;

    struct Harness<W> {
        backend: MockBackend,
        root: W,
        ui: Ui,
    }

    fn harness() -> Harness<impl Widget> {
        let (_, mut root, ui) = window();
        root.layout(Rect::new(0, 0, WINDOW_W, WINDOW_H));
        root.focus_first();
        Harness {
            backend: MockBackend::new(WINDOW_W, WINDOW_H).with_theme(theme(Theme::windows_31())),
            root,
            ui,
        }
    }

    impl<W: Widget> Harness<W> {
        /// Presses and releases a key. Returns whether that closed the window.
        fn key(&mut self, key: impl Into<Key>, modifiers: Modifiers) -> bool {
            let key = key.into();
            let down = self
                .backend
                .dispatch(&mut self.root, &Event::KeyDown { key, modifiers });
            if let Key::Char(ch) = key {
                self.backend.dispatch(&mut self.root, &Event::Char { ch, modifiers });
            }
            self.backend.dispatch(&mut self.root, &Event::KeyUp { key, modifiers });
            down.close_requested
        }

        fn press(&mut self, key: NamedKey) -> bool {
            self.key(key, Modifiers::default())
        }

        /// Picks a menu item with the keyboard: Alt and the menu's letter,
        /// then the item's. The frame painted afterwards is where a request
        /// to move the focus lands.
        fn menu(&mut self, menu: char, item: char) {
            let alt = Modifiers {
                alt: true,
                ..Modifiers::default()
            };
            self.backend.dispatch(
                &mut self.root,
                &Event::KeyDown {
                    key: Key::Char(menu),
                    modifiers: alt,
                },
            );
            self.backend.dispatch(
                &mut self.root,
                &Event::KeyDown {
                    key: Key::Char(item),
                    modifiers: Modifiers::default(),
                },
            );
            self.backend.render(&mut self.root);
        }

        fn type_text(&mut self, text: &str) {
            for ch in text.chars() {
                self.key(ch, Modifiers::default());
            }
        }

        /// Clicks a point of the dialog below the menu bar.
        fn click(&mut self, rect: Rect) {
            let pos = Point::new(rect.x + rect.w / 2, MENU_H + rect.y + rect.h / 2);
            let (button, modifiers) = (MouseButton::Left, Modifiers::default());
            self.backend
                .dispatch(&mut self.root, &Event::PointerDown { pos, button, modifiers });
            self.backend
                .dispatch(&mut self.root, &Event::PointerUp { pos, button, modifiers });
        }

        fn input(&self) -> String {
            self.ui.input.borrow().text()
        }

        fn output(&self) -> String {
            self.ui.output.borrow().text()
        }
    }

    /// Cmd on macOS, Ctrl elsewhere.
    fn primary() -> Modifiers {
        Modifiers {
            logo: cfg!(target_os = "macos"),
            control: !cfg!(target_os = "macos"),
            ..Modifiers::default()
        }
    }

    const SHIFT: Modifiers = Modifiers {
        shift: true,
        control: false,
        alt: false,
        logo: false,
        alt_graph: false,
    };

    fn decrypt_button() -> Rect {
        Rect::new(MARGIN, buttons_y(WINDOW_H - MENU_H), BUTTON_W, BUTTON_H)
    }

    #[test]
    fn typing_goes_to_the_input_box() {
        let mut h = harness();
        h.type_text("2aff014c 2411ffff");
        assert_eq!(h.input(), "2aff014c 2411ffff");
        assert_eq!(h.output(), "");
    }

    #[test]
    fn decrypt_button_fills_the_output_box() {
        let mut h = harness();
        h.type_text("Slot 1   1A7D6192 0000????");
        h.click(decrypt_button());
        assert_eq!(h.output(), "Slot 1\n10754244 0000????");
        assert_eq!(h.input(), "Slot 1   1A7D6192 0000????");
    }

    #[test]
    fn shortcuts_run_the_codes_menu() {
        let mut h = harness();
        h.type_text("2033BA10 FFFFFFFF");
        h.key('e', primary());
        assert_eq!(h.output(), "2ABB1147 FFFFFFFF");
        assert_eq!(h.input(), "2033BA10 FFFFFFFF", "the shortcut must not type");

        h.ui.input.borrow_mut().set_text("2ABB1147 FFFFFFFF");
        h.key('d', primary());
        assert_eq!(h.output(), "2033BA10 FFFFFFFF");

        h.ui.input.borrow_mut().set_text("x 2abb1147ffffffff");
        h.key('r', primary());
        assert_eq!(h.output(), "x\n2ABB1147 FFFFFFFF");
    }

    #[test]
    fn common_v7_checkbox_changes_the_result() {
        let mut h = harness();
        h.type_text("2033BA10 FFFFFFFF");
        h.ui.common_v7.borrow_mut().set_checked(true);
        h.key('e', primary());
        assert_eq!(h.output(), "AE17E302 4AE8B74D");
    }

    #[test]
    fn empty_input_asks_for_codes() {
        let mut h = harness();
        h.click(decrypt_button());
        assert!(h.ui.dialog.borrow().is_open());
        assert_eq!(h.output(), "");
    }

    #[test]
    fn shortcuts_and_menu_ask_for_codes_too() {
        for ch in ['d', 'e', 'r'] {
            let mut h = harness();
            h.key(ch, primary());
            assert!(h.ui.dialog.borrow().is_open(), "shortcut {ch}");

            let mut h = harness();
            h.menu('c', ch);
            assert!(h.ui.dialog.borrow().is_open(), "menu item {ch}");
        }
    }

    #[test]
    fn tab_moves_from_the_input_box_to_the_output_box() {
        let mut h = harness();
        assert!(h.ui.input.borrow().is_focused());
        h.key(NamedKey::Tab, Modifiers::default());
        assert!(h.ui.output.borrow().is_focused());
        assert!(!h.ui.input.borrow().is_focused());
        assert_eq!(h.input(), "", "Tab must not type");

        h.key(NamedKey::Tab, SHIFT);
        assert!(h.ui.input.borrow().is_focused());
    }

    #[test]
    fn output_box_is_read_only() {
        let mut h = harness();
        h.type_text("2033BA10 FFFFFFFF");
        h.key('r', primary());
        h.key(NamedKey::Tab, Modifiers::default());

        h.type_text("oops");
        h.key(NamedKey::Backspace, Modifiers::default());
        h.key(NamedKey::Enter, Modifiers::default());
        h.key('a', primary());
        h.key('x', primary());
        h.key('v', primary());
        assert_eq!(h.output(), "2033BA10 FFFFFFFF");
    }

    #[test]
    fn edit_menu_acts_on_the_box_used_last() {
        let h = harness();
        h.ui.input.borrow_mut().set_text("in");
        h.ui.output.borrow_mut().set_text("out");
        assert!(h.ui.input_is_active());

        let mut h = h;
        h.key(NamedKey::Tab, Modifiers::default());
        assert!(!h.ui.input_is_active());
        h.ui.active_editor().borrow_mut().set_text("");
        assert_eq!((h.input().as_str(), h.output().as_str()), ("in", ""));
    }

    #[test]
    fn undo_takes_back_what_was_typed() {
        let mut h = harness();
        assert!(!h.ui.can_undo());
        h.type_text("2033BA10");
        assert!(h.ui.can_undo());
        h.key('z', primary());
        assert_eq!(h.input(), "");
        assert!(!h.ui.can_undo());

        // Shift+Z is the editor's own redo; the menu has no item for it.
        let redo = Modifiers {
            shift: true,
            ..primary()
        };
        h.key('z', redo);
        assert_eq!(h.input(), "2033BA10");
    }

    #[test]
    fn delete_removes_the_selection_or_the_next_character() {
        let mut h = harness();
        h.type_text("abcd");
        h.press(NamedKey::Home);
        // Nothing selected: the menu item is grey and the key is the box's.
        assert!(!h.ui.can_cut());
        h.press(NamedKey::Delete);
        assert_eq!(h.input(), "bcd");

        h.key(NamedKey::Right, SHIFT);
        h.key(NamedKey::Right, SHIFT);
        assert!(h.ui.can_cut() && h.ui.can_copy());
        h.press(NamedKey::Delete);
        assert_eq!(h.input(), "d");

        h.menu('e', 'u');
        assert_eq!(h.input(), "bcd", "Edit > Undo");
    }

    #[test]
    fn edit_menu_is_greyed_by_the_box_used_last() {
        let mut h = harness();
        assert!(!h.ui.can_select() && !h.ui.can_copy() && !h.ui.can_undo());
        h.type_text("2033BA10 FFFFFFFF");
        h.key('r', primary());
        assert!(h.ui.can_select() && h.ui.can_undo() && !h.ui.can_copy());

        // The output box can be selected from and copied, never changed.
        h.press(NamedKey::Tab);
        h.key('a', primary());
        assert!(h.ui.can_select() && h.ui.can_copy());
        assert!(!h.ui.can_cut() && !h.ui.can_undo() && !h.ui.input_is_active());

        h.menu('e', 'r');
        assert_eq!((h.input().as_str(), h.output().as_str()), ("2033BA10 FFFFFFFF", ""));
        assert!(!h.ui.can_select(), "Edit > Clear emptied the output box");
    }

    #[test]
    fn the_quit_shortcut_exits_and_escape_does_not() {
        let mut h = harness();
        assert!(!h.press(NamedKey::Escape));
        assert!(h.key('q', primary()));
    }

    #[test]
    fn escape_closes_a_message_box_and_stops_there() {
        let mut h = harness();
        h.click(decrypt_button());
        assert!(h.ui.dialog.borrow().is_open());
        // The box takes shape in the frame that follows the click.
        h.backend.render(&mut h.root);
        assert!(!h.press(NamedKey::Escape));
        assert!(!h.ui.dialog.borrow().is_open());
        assert!(!h.press(NamedKey::Escape));
    }

    #[test]
    fn escape_closes_a_right_click_menu_and_stops_there() {
        let mut h = harness();
        h.type_text("abcd");
        h.key('a', primary());
        let input = h.ui.input.borrow().bounds();
        let pos = Point::new(input.x + 20, input.y + 8);
        let (button, modifiers) = (MouseButton::Right, Modifiers::default());
        h.backend
            .dispatch(&mut h.root, &Event::PointerDown { pos, button, modifiers });
        h.backend
            .dispatch(&mut h.root, &Event::PointerUp { pos, button, modifiers });
        h.backend.render(&mut h.root);

        // The menu is up, so "d" picks its Delete instead of being typed.
        // (The runtime drops the character that follows a picked key.)
        let key = Key::Char('d');
        h.backend.dispatch(&mut h.root, &Event::KeyDown { key, modifiers });
        assert_eq!(h.input(), "");
        h.backend
            .dispatch(&mut h.root, &Event::PointerDown { pos, button, modifiers });
        assert!(!h.press(NamedKey::Escape), "Escape closes the menu");
        assert!(!h.press(NamedKey::Escape), "and not the program after it");
    }

    #[test]
    fn f1_opens_the_about_box() {
        let mut h = harness();
        h.press(NamedKey::F1);
        assert!(h.ui.about.borrow().is_open());
        assert!(!h.press(NamedKey::Escape));
        assert!(!h.ui.about.borrow().is_open());

        h.press(NamedKey::F1);
        h.press(NamedKey::Enter);
        assert!(!h.ui.about.borrow().is_open(), "Enter presses OK");
    }

    #[test]
    fn buttons_hand_the_keyboard_back_to_the_input_box() {
        let mut h = harness();
        h.type_text("2033BA10 FFFFFFFF");
        h.press(NamedKey::Tab);
        assert!(h.ui.output.borrow().is_focused());

        h.click(decrypt_button());
        assert!(h.ui.input.borrow().is_focused());
        assert!(!h.ui.output.borrow().is_focused());
        h.type_text("!");
        assert_eq!(h.input(), "2033BA10 FFFFFFFF!");
    }

    #[test]
    fn copy_output_to_input_moves_the_text_and_the_keyboard() {
        let mut h = harness();
        h.type_text("x 2033ba10ffffffff");
        h.key('r', primary());
        h.press(NamedKey::Tab);

        h.menu('e', 'i');
        assert_eq!(h.input(), "x\n2033BA10 FFFFFFFF");
        assert!(h.ui.input.borrow().is_focused());
        assert!(!h.ui.can_undo(), "a new text has no history");
    }

    #[test]
    fn check_box_and_options_menu_set_the_options() {
        let mut h = harness();
        assert_eq!(h.ui.options(), Options::default(), "every run starts with both off");

        let (x, width) = column(Side::Output, WINDOW_W);
        h.click(Rect::new(x, buttons_y(WINDOW_H - MENU_H) + 4, width - BUTTON_W - 8, 16));
        assert!(h.ui.options().common_v7);
        h.menu('o', 'b');
        let both = Options {
            common_v7: true,
            blank_line: true,
        };
        assert_eq!(h.ui.options(), both);

        h.menu('o', 'v');
        let blank_line_only = Options {
            common_v7: false,
            blank_line: true,
        };
        assert_eq!(h.ui.options(), blank_line_only);
    }

    #[test]
    fn platform_keys_jump_to_the_ends_of_line_and_text() {
        let mac = cfg!(target_os = "macos");
        let (top, bottom) = if mac {
            (NamedKey::Up, NamedKey::Down)
        } else {
            (NamedKey::Home, NamedKey::End)
        };
        let mut h = harness();
        h.ui.input.borrow_mut().set_text("ab\ncd\nef");

        h.key(bottom, primary());
        h.type_text("1");
        assert_eq!(h.input(), "ab\ncd\nef1");
        h.key(top, primary());
        h.type_text("2");
        assert_eq!(h.input(), "2ab\ncd\nef1");
        if mac {
            h.key(NamedKey::Right, primary());
            h.type_text("3");
            assert_eq!(h.input(), "2ab3\ncd\nef1");
            h.key(NamedKey::Left, primary());
            h.type_text("4");
            assert_eq!(h.input(), "42ab3\ncd\nef1");
            h.key(top, primary());
        }

        // With Shift the way there is selected.
        let shifted = Modifiers {
            shift: true,
            ..primary()
        };
        h.key(bottom, shifted);
        h.press(NamedKey::Backspace);
        assert_eq!(h.input(), "");
    }

    #[test]
    fn a_clicked_menu_opens_with_nothing_picked() {
        let mut h = harness();
        h.backend.render(&mut h.root);
        // The File menu's name, at the very left of the bar.
        let pos = Point::new(2, MENU_H / 2);
        let (button, modifiers) = (MouseButton::Left, Modifiers::default());
        h.backend
            .dispatch(&mut h.root, &Event::PointerDown { pos, button, modifiers });
        h.backend
            .dispatch(&mut h.root, &Event::PointerUp { pos, button, modifiers });

        h.press(NamedKey::Enter);
        assert!(!h.ui.files.borrow().is_open(), "no item to fire yet");
        h.press(NamedKey::Down);
        h.press(NamedKey::Enter);
        assert!(h.ui.files.borrow().is_open(), "Load, the first item");

        // From the keyboard the first item is ready at once.
        let mut h = harness();
        let alt = Modifiers {
            alt: true,
            ..Modifiers::default()
        };
        h.key('f', alt);
        h.press(NamedKey::Enter);
        assert!(h.ui.files.borrow().is_open());
    }

    #[test]
    fn alt_and_a_letter_switch_from_an_open_menu_to_another() {
        let mut h = harness();
        let alt = Modifiers {
            alt: true,
            ..Modifiers::default()
        };
        h.key('f', alt);
        h.key('c', alt);
        // Decrypt, the first item of the Codes menu, with no codes to work on.
        h.press(NamedKey::Enter);
        assert!(h.ui.dialog.borrow().is_open());
        assert!(!h.ui.files.borrow().is_open(), "the File menu is closed");
    }

    #[test]
    fn text_boxes_scroll_sideways() {
        let h = harness();
        // Room for the horizontal scrollbar is taken out of both boxes: a
        // press on it must not land in the text.
        let input = h.ui.input.borrow().bounds();
        let mut h = h;
        h.ui.input.borrow_mut().set_text("a\nb");
        h.click(Rect::new(input.x + 40, input.bottom() - MENU_H - 8, 1, 1));
        h.type_text("x");
        assert_eq!(h.input(), "xa\nb", "the caret stayed where it was");
    }

    #[test]
    fn controls_follow_the_window_size() {
        let (_, mut root, ui) = window();
        for (w, h) in [(600, 300), (WINDOW_W, WINDOW_H), (1200, 900)] {
            root.layout(Rect::new(0, 0, w, h));
            let (input, output) = (ui.input.borrow().bounds(), ui.output.borrow().bounds());
            assert_eq!(input.x, MARGIN);
            assert_eq!(output.right(), w - MARGIN);
            assert_eq!(output.x - input.right(), GAP);
            assert_eq!(input.y, MENU_H + BOXES_Y);
            assert_eq!(input.bottom(), h - MARGIN - BUTTON_H - 10);
        }
    }
}
