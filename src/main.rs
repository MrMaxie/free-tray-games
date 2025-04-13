#![windows_subsystem = "windows"]
use anyhow::{Context, Result};

mod config;
use config::Config;

mod games;
use games::Games;

mod notifications;
use notifications::Notifications;

mod tray_body;
mod resource;

mod tray;
use tray::{Message, Tray};

mod logger;
use logger::init_logger;

mod notify_body;

#[tokio::main]
async fn main() -> Result<()> {
    init_logger()?;

    if let Err(e) = body().await {
        tracing::error!("Error: {}", e);
        return Err(e);
    }

    Ok(())
}

async fn body() -> Result<()> {
    Tray::make_tray_nice();

    let mut config = Config::new();
    config.load().await;

    let mut games = Games::fetch().await.context("Failed to initialize games")?;

    let notifications = Notifications::new(
        games.clone(),
        config.clone(),
    );
    notifications.load_or_init().await.context("Failed to load notifications")?;
    notifications.push_all_new_games().await.context("Failed to push notifications")?;

    let (tray, rx) = Tray::new(
        games.clone(),
        config.clone(),
    ).await;

    let auto_refresh_tx = tray.get_tx().clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(3 * 60 * 60)).await;
            let _ = auto_refresh_tx.send(Message::Refresh);
        }
    });

    println!("FreeTrayGames is running...");

    loop {
        match rx.recv() {
            Ok(Message::Quit) => {
                break;
            }
            Ok(Message::OpenGame(id)) => {
                let games = games.get_all().await;

                for game in &games {
                    if game.id == id {
                        let target_url = game.open_giveaway_url.clone();
                        let _ = open::that(target_url);
                        break;
                    }
                }
            }
            Ok(Message::OpenProjectHomepage) => {
                let _ = open::that("https://github.com/MrMaxie/free-tray-games");
            }
            Ok(Message::Refresh) => {
                games.refetch().await?;
                notifications.push_all_new_games().await?;
                tray.rebuild_tray().await?;
            }
            Ok(Message::ToggleNotifications) => {
                config.toggle_notifications().await;
                config.save().await;
                notifications.push_all_new_games().await?;
                tray.rebuild_tray().await?;
            }
            _ => {}
        }
    }

    Ok(())
}
