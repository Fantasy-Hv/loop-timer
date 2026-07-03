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
    rest_timer_id: Rc<RefCell<Option<glib::SourceId>>>,
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
            rest_timer_id: Rc::new(RefCell::new(None)),
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

        let rest_label = gtk::Label::new(None);
        rest_label.set_justify(gtk::Justification::Center);
        rest_label.add_css_class("rest-text");

        let button = gtk::Button::with_label(confirm_text);
        button.add_css_class("confirm-button");
        button.set_halign(gtk::Align::Center);
        button.set_sensitive(false);

        vbox.append(&label);
        vbox.append(&rest_label);
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

        let state = self.state.clone();
        let rest_label_c = rest_label.clone();
        let button_c = button.clone();
        let rest_timer_id = self.rest_timer_id.clone();

        let timer_id = glib::timeout_add_local(std::time::Duration::from_secs(1), move || {
            let s = state.lock().unwrap();
            let rest = s.rest_remaining_seconds;
            drop(s);

            if rest > 0 {
                let m = rest / 60;
                let sec = rest % 60;
                rest_label_c.set_text(&format!("{:02}:{:02}", m, sec));
                glib::ControlFlow::Continue
            } else {
                rest_label_c.set_text("");
                button_c.set_sensitive(true);
                button_c.remove_css_class("confirm-button");
                button_c.add_css_class("confirm-button-ready");
                *rest_timer_id.borrow_mut() = None;
                glib::ControlFlow::Break
            }
        });

        *self.rest_timer_id.borrow_mut() = Some(timer_id);

        {
            let s = self.state.lock().unwrap();
            let rest = s.rest_remaining_seconds;
            let m = rest / 60;
            let sec = rest % 60;
            rest_label.set_text(&format!("{:02}:{:02}", m, sec));
        }

        window.fullscreen();
        window.present();

        *self.current.borrow_mut() = Some(window);
    }

    pub fn hide(&self) {
        if let Some(id) = self.rest_timer_id.borrow_mut().take() {
            id.remove();
        }
        if let Some(window) = self.current.borrow_mut().take() {
            window.close();
        }
    }
}
