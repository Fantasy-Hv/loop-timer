mod config;
mod tray;
mod overlay;

use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;

use gtk4::prelude::*;
use gtk4::{self as gtk, glib};

use config::load_or_create;
use overlay::OverlayWindow;
use tray::{LoopTimerTray, TrayCommand};

#[derive(Debug)]
pub struct AppState {
    pub remaining_seconds: u64,
    pub config_countdown: u64,
    pub config_rest: u64,
    pub rest_remaining_seconds: u64,
    pub is_paused: bool,
    pub is_notifying: bool,
    pub notification_text: String,
    pub confirm_text: String,
}

impl AppState {
    fn from_config(cfg: &config::Config) -> Self {
        AppState {
            remaining_seconds: cfg.general.countdown_seconds,
            config_countdown: cfg.general.countdown_seconds,
            config_rest: cfg.general.rest_seconds,
            rest_remaining_seconds: 0,
            is_paused: false,
            is_notifying: false,
            notification_text: cfg.notification.text.clone(),
            confirm_text: cfg.notification.confirm_text.clone(),
        }
    }
}

fn main() -> glib::ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let config_path_arg = args
        .iter()
        .position(|a| a == "--config" || a == "-c")
        .and_then(|i| args.get(i + 1).map(PathBuf::from));

    let (initial_config, config_path) = load_or_create(config_path_arg);
    let state = Arc::new(Mutex::new(AppState::from_config(&initial_config)));

    let app = gtk::Application::new(Some("com.rest-timer.app"), Default::default());

    let state_c = state.clone();
    let config_path_c = config_path.clone();
    app.connect_activate(move |app| {
        activate(app, state_c.clone(), config_path_c.clone());
    });

    app.run_with_args(&["rest-timer"])
}

fn activate(app: &gtk::Application, state: Arc<Mutex<AppState>>, config_path: PathBuf) {
    std::mem::forget(app.hold());
    let (confirm_tx, confirm_rx) = mpsc::channel::<()>();
    let overlay = Rc::new(OverlayWindow::new(app, state.clone(), confirm_tx));

    let (tray_cmd_tx, tray_cmd_rx) = mpsc::channel::<TrayCommand>();

    let tray = LoopTimerTray {
        state: state.clone(),
        tx: tray_cmd_tx,
    };

    let tray_handle = match ksni::blocking::TrayMethods::spawn(tray) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("rest-timer: tray service failed: {e}");
            activate_no_tray(app, state, config_path, overlay, confirm_rx);
            return;
        }
    };

    {
        let state = state.clone();
        let overlay = overlay.clone();
        let tray_handle = tray_handle.clone();
        glib::timeout_add_seconds_local(1, move || {
            let mut s = state.lock().unwrap();

            if s.is_notifying {
                if s.rest_remaining_seconds > 0 {
                    s.rest_remaining_seconds -= 1;
                }
                drop(s);
                let _ = tray_handle.update(|_| {});
                return glib::ControlFlow::Continue;
            }

            if !s.is_paused && s.remaining_seconds > 0 {
                s.remaining_seconds -= 1;
            }

            if s.remaining_seconds == 0 && !s.is_paused {
                s.is_notifying = true;
                s.rest_remaining_seconds = s.config_rest;
                let notif = s.notification_text.clone();
                let conf = s.confirm_text.clone();
                drop(s);
                overlay.show(&notif, &conf);
            } else {
                drop(s);
            }

            let _ = tray_handle.update(|_| {});
            glib::ControlFlow::Continue
        });
    }

    {
        glib::timeout_add_local(Duration::from_millis(200), move || {
            while let Ok(cmd) = tray_cmd_rx.try_recv() {
                match cmd {
                    TrayCommand::Exit => {
                        std::process::exit(0);
                    }
                }
            }
            glib::ControlFlow::Continue
        });
    }

    {
        let overlay = overlay.clone();
        glib::timeout_add_local(Duration::from_millis(100), move || {
            while let Ok(()) = confirm_rx.try_recv() {
                overlay.hide();
            }
            glib::ControlFlow::Continue
        });
    }

    let state_watch = state.clone();
    let config_path_watch = config_path.clone();
    std::thread::spawn(move || {
        watch_config(state_watch, config_path_watch);
    });

    println!(
        "rest-timer started (config: {})",
        config_path.display()
    );
}

fn activate_no_tray(
    _app: &gtk::Application,
    state: Arc<Mutex<AppState>>,
    config_path: PathBuf,
    overlay: Rc<OverlayWindow>,
    confirm_rx: mpsc::Receiver<()>,
) {
    {
        let state = state.clone();
        let overlay = overlay.clone();
        glib::timeout_add_seconds_local(1, move || {
            let mut s = state.lock().unwrap();

            if s.is_notifying {
                if s.rest_remaining_seconds > 0 {
                    s.rest_remaining_seconds -= 1;
                }
                return glib::ControlFlow::Continue;
            }

            if !s.is_paused && s.remaining_seconds > 0 {
                s.remaining_seconds -= 1;
            }

            if s.remaining_seconds == 0 {
                s.is_notifying = true;
                s.rest_remaining_seconds = s.config_rest;
                let notif = s.notification_text.clone();
                let conf = s.confirm_text.clone();
                drop(s);
                overlay.show(&notif, &conf);
            }

            glib::ControlFlow::Continue
        });
    }

    {
        let overlay = overlay.clone();
        glib::timeout_add_local(Duration::from_millis(100), move || {
            while let Ok(()) = confirm_rx.try_recv() {
                overlay.hide();
            }
            glib::ControlFlow::Continue
        });
    }

    let state_watch = state.clone();
    let config_path_watch = config_path.clone();
    std::thread::spawn(move || {
        watch_config(state_watch, config_path_watch);
    });

    println!(
        "rest-timer started without tray (config: {})",
        config_path.display()
    );
}

fn watch_config(state: Arc<Mutex<AppState>>, config_path: PathBuf) {
    use notify::{recommended_watcher, Event, EventKind, RecursiveMode, Watcher};

    let (tx, rx) = mpsc::channel::<()>();
    let config_path_c = config_path.clone();

    let mut watcher = match recommended_watcher(move |res: Result<Event, notify::Error>| {
        if let Ok(event) = res {
            if event.paths.iter().any(|p| p == &config_path_c) {
                match event.kind {
                    EventKind::Modify(_) | EventKind::Create(_) => {
                        let _ = tx.send(());
                    }
                    _ => {}
                }
            }
        }
    }) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("rest-timer: cannot watch config: {e}");
            return;
        }
    };

    let watch_dir = config_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    if watcher.watch(&watch_dir, RecursiveMode::NonRecursive).is_err() {
        eprintln!("rest-timer: failed to watch config directory");
        return;
    }

    loop {
        match rx.recv() {
            Ok(()) => {
                std::thread::sleep(Duration::from_millis(300));
                while rx.try_recv().is_ok() {}

                if let Some(new_cfg) = config::reload(&config_path) {
                    let mut s = state.lock().unwrap();
                    s.notification_text = new_cfg.notification.text.clone();
                    s.confirm_text = new_cfg.notification.confirm_text.clone();
                    if new_cfg.general.countdown_seconds != s.config_countdown {
                        s.config_countdown = new_cfg.general.countdown_seconds;
                        s.remaining_seconds = s.config_countdown;
                    }
                    s.config_rest = new_cfg.general.rest_seconds;
                }
            }
            Err(_) => break,
        }
    }
}
