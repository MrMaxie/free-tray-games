use std::sync::Arc;
use reqwest::StatusCode;
use serde::Deserialize;
use tokio::sync::Mutex;
use anyhow::{Context, Result};

#[derive(Debug, Deserialize, Clone)]
pub struct Game {
    pub id: usize,
    pub title: String,
    pub platforms: String,
    pub image: String,
    #[serde(default)]
    pub platform: String,
    pub open_giveaway_url: String,
    #[serde(rename = "type")]
    pub game_type: String,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct Games {
    games: Arc<Mutex<Vec<Game>>>,
    empty_reason: Arc<Mutex<Option<String>>>,
}

impl Games {
    pub fn new() -> Self {
        Games {
            games: Arc::new(Mutex::new(Vec::new())),
            empty_reason: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn get_error_reason(&self) -> Option<String> {
        let empty_reason = self.empty_reason.lock().await;
        empty_reason.clone()
    }

    async fn set_error_reason(&self, reason: String) {
        let mut empty_reason = self.empty_reason.lock().await;
        *empty_reason = Some(reason);
    }

    async fn clear_error_reason(&self) {
        let mut empty_reason = self.empty_reason.lock().await;
        *empty_reason = None;
    }

    async fn consume_response(&self, response: reqwest::Response) -> Result<()> {
        self.clear_error_reason().await;

        let raw_games: Vec<Game> = response.json().await.context("Failed to parse JSON from API")?;

        let filtered_games: Vec<Game> = raw_games
            .into_iter()
            .filter(|g| g.game_type.eq_ignore_ascii_case("game"))
            .filter(|g| g.status.eq_ignore_ascii_case("active"))
            .filter_map(|mut g| {
                let p = g.platforms.to_lowercase();
                g.platform = if p.contains("steam") {
                    "steam".into()
                } else if p.contains("epic") {
                    "epic".into()
                } else if p.contains("gog") {
                    "gog".into()
                } else {
                    return None;
                };
                Some(g)
            })
            .collect();

        let mut data = self.games.lock().await;
        *data = filtered_games;

        Ok(())
    }

    pub async fn refetch(&mut self) -> Result<()> {
        let url = "https://www.gamerpower.com/api/giveaways?platform=pc";
        match reqwest::get(url).await {
            Ok(response) => {
                if !response.status().is_success() {
                    match response.status() {
                        StatusCode::NOT_FOUND => {
                            self.set_error_reason("No active giveaways".to_string()).await;
                        }
                        StatusCode::INTERNAL_SERVER_ERROR => {
                            self.set_error_reason("Server error".to_string()).await;
                        }
                        _ => {
                            self.set_error_reason("Unknown error".to_string()).await;
                        }
                    }
                    return Ok(());
                }

                self.consume_response(response).await.context("Failed to consume response")
            }
            Err(e) => {
                let reason = if e.is_connect() {
                    "Network unavailable".to_string()
                } else if e.is_timeout() {
                    "Request timed out".to_string()
                } else {
                    "Unknown network error".to_string()
                };

                self.set_error_reason(reason).await;
                return Ok(());
            }
        }
    }

    pub async fn get_all(&self) -> Vec<Game> {
        let data = self.games.lock().await;
        data.clone()
    }

    pub async fn fetch() -> Result<Self> {
        let mut games = Games::new();
        games.refetch().await.context("Failed to fetch games")?;
        Ok(games)
    }
}