use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex, mpsc};

use gtk4::prelude::*;
use gtk4::{self as gtk, gdk, glib};

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

    fn monitor_geometry(&self) -> (i32, i32) {
        let display = gdk::Display::default().unwrap();
        let monitors = display.monitors();

        let mut w = 1920;
        let mut h = 1080;

        for i in 0..monitors.n_items() {
            if let Some(obj) = monitors.item(i) {
                if let Ok(monitor) = obj.downcast::<gdk::Monitor>() {
                    let geo = monitor.geometry();
                    if geo.width() > w {
                        w = geo.width();
                    }
                    if geo.height() > h {
                        h = geo.height();
                    }
                }
            }
        }

        (w, h)
    }

    pub fn show(&self, text: &str, confirm_text: &str) {
        self.ensure_css();

        let (mw, mh) = self.monitor_geometry();

        let window = gtk::ApplicationWindow::new(&self.app);
        window.set_default_size(mw, mh);
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
        label.set_wrap_mode(gtk::pango::WrapMode::Word);
        label.add_css_class("notification-text");

        let button = gtk::Button::with_label(confirm_text);
        button.add_css_class("confirm-button");
        button.set_halign(gtk::Align::Center);

        vbox.append(&label);
        vbox.append(&button);

        window.set_child(Some(&vbox));

        let state = self.state.clone();
        let confirm_tx = self.confirm_tx.clone();

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

        window.fullscreen();
        window.present();

        *self.current.borrow_mut() = Some(window);
    }

    pub fn hide(&self) {
        if let Some(window) = self.current.borrow_mut().take() {
            window.close();
        }
    }
}
