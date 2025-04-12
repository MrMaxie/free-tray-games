use std::{
    ffi::OsStr, os::windows::ffi::OsStrExt, sync::{mpsc, Arc}
};
use tokio::sync::Mutex;
use anyhow::{Result, Context};
use windows_sys::Win32::UI::{
    HiDpi::{SetProcessDpiAwareness, PROCESS_PER_MONITOR_DPI_AWARE},
    Shell::SetCurrentProcessExplicitAppUserModelID,
};

use crate::tray_body::TrayBody;
use crate::{config::Config, games::Games};
use crate::resource::ResourceIcon;

pub enum Message {
    Quit,
    OpenGame(usize),
    OpenProjectHomepage,
    Refresh,
    ToggleNotifications,
}

pub struct Tray {
    current: Arc<Mutex<Option<TrayBody>>>,
    games: Games,
    tx: mpsc::SyncSender<Message>,
    config: Config,
}

impl Tray {
    pub async fn new(
        games: Games,
        config: Config,
    ) -> (Self, mpsc::Receiver<Message>) {
        let (tx, rx) = mpsc::sync_channel(1);
        let current = Arc::new(Mutex::new(None));

        let new_tray = Tray {
            games,
            current,
            tx,
            config,
        };

        new_tray.rebuild_tray().await.unwrap_or_else(|e| {
            eprintln!("Failed to rebuild tray: {}", e);
            std::process::exit(1);
        });

        (new_tray, rx)
    }

    pub fn get_tx(&self) -> mpsc::SyncSender<Message> {
        self.tx.clone()
    }

    pub async fn rebuild_tray(&self) -> Result<()> {
        let mut current = self.current.lock().await;
        let new_tray = self.build_tray().await.context("Failed to build tray")?;
        *current = Some(new_tray);
        Ok(())
    }

    async fn build_tray(&self) -> Result<TrayBody> {
        let mut tray = TrayBody::new("FreeTrayGames", &ResourceIcon::Main).context("Failed to create tray instance")?;
        let tx = self.tx.clone();

        let games = self.games.get_all().await;

        match (games.len(), self.games.get_error_reason().await) {
            (_, Some(error_reason)) => {
                tray.add_label(format!("Failed to load: {}", error_reason).as_str())
                    .context("Failed to add error reason label")?;
            }
            (0, None) => {
                tray.add_label("No active giveaways")
                    .context("Failed to add no active giveaways menu item")?;
            }
            (_, None)  => {
                for game in &games {
                    let title = game.title.clone();
                    let id = game.id;
                    let item_open_tx = tx.clone();

                    let icon = match game.platform.as_str() {
                        "steam" => Some(&ResourceIcon::BrandSteam),
                        "epic" => Some(&ResourceIcon::BrandEpic),
                        "gog" => Some(&ResourceIcon::BrandGog),
                        _ => None,
                    };

                    tray.add_menu_item(&title, move || {
                        let _ = item_open_tx.send(Message::OpenGame(id));
                    }, if let Some(
                        icon,
                    ) = icon {
                        Some(icon)
                    } else {
                        None
                    }).context("Failed to add game menu item")?;
                }
            }
        }

        tray.add_separator().context("Failed to add separator")?;

        let refresh_tx = tx.clone();
        tray.add_menu_item("Refresh", move || {
            let _ = refresh_tx.send(Message::Refresh);
        }, Some(&ResourceIcon::Refresh)).context("Failed to add refresh menu item")?;

        let open_project_homepage_tx = tx.clone();
        tray.add_menu_item("Homepage", move || {
            let _ = open_project_homepage_tx.send(Message::OpenProjectHomepage);
        }, Some(&ResourceIcon::BrandGithub))
            .context("Failed to add project homepage menu item")?;

        let notifications_tx = tx.clone();
        let is_notifications_enabled = self.config.is_notifications_enabled().await;
        let notifications_icon = if is_notifications_enabled {
            &ResourceIcon::NotificationsEnabled
        } else {
            &ResourceIcon::NotificationsDisabled
        };
        tray.add_menu_item("Toggle Notifications", move || {
            let _ = notifications_tx.send(Message::ToggleNotifications);
        }, Some(&notifications_icon))
            .context("Failed to add toggle notifications menu item")?;

        tray.add_separator().context("Failed to add separator")?;

        let quit_tx = tx.clone();
        tray.add_menu_item("Quit", move || {
            let _ = quit_tx.send(Message::Quit);
        }, None)
            .context("Failed to add quit menu item")?;

        Ok(tray)
    }

    pub fn make_tray_nice() {
        let app_id = OsStr::new(Config::get_app_id().as_str())
            .encode_wide()
            .chain(Some(0))
            .collect::<Vec<u16>>();

        unsafe {
            SetCurrentProcessExplicitAppUserModelID(app_id.as_ptr());
            SetProcessDpiAwareness(PROCESS_PER_MONITOR_DPI_AWARE);
        }
    }
}