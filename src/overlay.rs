use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex, mpsc};

use gtk4::prelude::*;
use gtk4::{self as gtk, gdk, glib};

use gtk4_layer_shell::LayerShell;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer};

pub struct OverlayWindow {
    app: gtk::Application,
    css_loaded: Rc<RefCell<bool>>,
    state: Arc<Mutex<crate::AppState>>,
    confirm_tx: mpsc::Sender<()>,
    current: Rc<RefCell<Option<gtk::ApplicationWindow>>>,
}

impl OverlayWindow {
    pub fn new(
        app: &gtk::Application,
        state: Arc<Mutex<crate::AppState>>,
        confirm_tx: mpsc::Sender<()>,
    ) -> Self {
        OverlayWindow {
            app: app.clone(),
            css_loaded: Rc::new(RefCell::new(false)),
            state,
            confirm_tx,
            current: Rc::new(RefCell::new(None)),
        }
    }

    fn ensure_css(&self) {
        if *self.css_loaded.borrow() {
            return;
        }
        let display = gdk::Display::default().expect("Cannot open display");
        let css = gtk::CssProvider::new();
        css.load_from_data(include_str!("style.css"));
        gtk::style_context_add_provider_for_display(
            &display,
            &css,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        *self.css_loaded.borrow_mut() = true;
    }

    pub fn show(&self, text: &str) {
        self.ensure_css();

        let window = gtk::ApplicationWindow::new(&self.app);
        window.set_default_size(2, 2);
        window.set_decorated(false);
        window.set_resizable(false);
        window.add_css_class("overlay-window");

        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 30);
        vbox.set_halign(gtk::Align::Center);
        vbox.set_valign(gtk::Align::Center);
        vbox.set_hexpand(true);
        vbox.set_vexpand(true);

        let label = gtk::Label::new(Some(text));
        label.set_justify(gtk::Justification::Center);
        label.set_wrap(true);
        label.add_css_class("notification-text");

        let button = gtk::Button::with_label("我知道了 / Got it");
        button.add_css_class("confirm-button");
        button.set_halign(gtk::Align::Center);

        vbox.append(&label);
        vbox.append(&button);

        window.set_child(Some(&vbox));

        let state = self.state.clone();
        let confirm_tx = self.confirm_tx.clone();
        let current = self.current.clone();

        button.connect_clicked(move |_| {
            let mut s = state.lock().unwrap();
            s.is_notifying = false;
            s.remaining_seconds = s.config_countdown;
            let _ = confirm_tx.send(());
        });

        let state2 = self.state.clone();
        let confirm_tx2 = self.confirm_tx.clone();
        let key_ctrl = gtk::EventControllerKey::new();
        key_ctrl.connect_key_pressed(move |_, key, _, _| {
            if key == gdk::Key::Escape
                || key == gdk::Key::Return
                || key == gdk::Key::KP_Enter
            {
                let mut s = state2.lock().unwrap();
                s.is_notifying = false;
                s.remaining_seconds = s.config_countdown;
                let _ = confirm_tx2.send(());
            }
            glib::Propagation::Stop
        });
        window.add_controller(key_ctrl);

        window.init_layer_shell();
        window.set_layer(Layer::Overlay);
        window.set_keyboard_mode(KeyboardMode::Exclusive);

        for e in [Edge::Top, Edge::Bottom, Edge::Left, Edge::Right] {
            window.set_anchor(e, true);
            window.set_margin(e, 0);
        }

        window.present();
        window.fullscreen();

        *current.borrow_mut() = Some(window);
    }

    pub fn hide(&self) {
        if let Some(window) = self.current.borrow_mut().take() {
            window.close();
        }
    }
}
