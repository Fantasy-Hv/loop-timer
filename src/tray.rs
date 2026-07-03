use std::sync::{Arc, Mutex, mpsc};

use ksni::{
    menu::{MenuItem, StandardItem},
    ToolTip, Tray,
};

use crate::AppState;

#[derive(Debug, Clone)]
pub enum TrayCommand {
    Exit,
}

pub struct LoopTimerTray {
    pub state: Arc<Mutex<AppState>>,
    pub tx: mpsc::Sender<TrayCommand>,
}

impl Tray for LoopTimerTray {
    fn id(&self) -> String {
        "loop-timer".into()
    }

    fn icon_name(&self) -> String {
        "empathy".into()
    }

    fn title(&self) -> String {
        "loop-timer".into()
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        let mut s = self.state.lock().unwrap();
        if !s.is_notifying {
            s.is_paused = !s.is_paused;
        }
    }

    fn tool_tip(&self) -> ToolTip {
        let s = self.state.lock().unwrap();

        let desc = if s.is_notifying {
            "Time's up!".into()
        } else if s.is_paused {
            "Paused".into()
        } else {
            let m = s.remaining_seconds / 60;
            let sec = s.remaining_seconds % 60;
            format!("{:02}:{:02}", m, sec)
        };

        ToolTip {
            icon_name: String::new(),
            icon_pixmap: Vec::new(),
            title: "loop-timer".into(),
            description: desc,
        }
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let s = self.state.lock().unwrap();

        let status = if s.is_notifying {
            "\u{23F0} Time's up!".into()
        } else {
            let m = s.remaining_seconds / 60;
            let sec = s.remaining_seconds % 60;
            if s.is_paused {
                format!("\u{23F8}\u{FE0F} {:02}:{:02}", m, sec)
            } else {
                format!("\u{23F3} {:02}:{:02}", m, sec)
            }
        };

        let pause_label = if s.is_paused {
            "\u{25B6}\u{FE0F} Resume"
        } else {
            "\u{23F8}\u{FE0F} Pause"
        };

        drop(s);

        let items: Vec<MenuItem<Self>> = vec![
            MenuItem::Standard(StandardItem {
                label: status,
                enabled: false,
                ..Default::default()
            }),
            MenuItem::Separator,
            MenuItem::Standard(StandardItem {
                label: pause_label.into(),
                enabled: true,
                activate: Box::new(|tray: &mut Self| {
                    let mut s = tray.state.lock().unwrap();
                    if !s.is_notifying {
                        s.is_paused = !s.is_paused;
                    }
                }),
                ..Default::default()
            }),
            MenuItem::Standard(StandardItem {
                label: "\u{1F504} Restart".into(),
                enabled: true,
                activate: Box::new(|tray: &mut Self| {
                    let mut s = tray.state.lock().unwrap();
                    if !s.is_notifying {
                        s.remaining_seconds = s.config_countdown;
                        s.is_paused = false;
                    }
                }),
                ..Default::default()
            }),
            MenuItem::Separator,
            MenuItem::Standard(StandardItem {
                label: "\u{274C} Exit".into(),
                enabled: true,
                activate: Box::new(|tray: &mut Self| {
                    let _ = tray.tx.send(TrayCommand::Exit);
                }),
                ..Default::default()
            }),
        ];

        items
    }
}
