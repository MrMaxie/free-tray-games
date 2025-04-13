use std::{collections::HashMap, env, fs::{create_dir_all, File}, io::{BufReader, BufWriter, Write}, path::PathBuf, sync::Arc};
use chrono::{DateTime, Duration, Utc};
use tokio::sync::Mutex;
use anyhow::{Result, Context};

use crate::notify_body::WinToastNotify;
use crate::{config::Config, games::{Game, Games}};

type NotifiedMap = HashMap<usize, DateTime<Utc>>;

#[derive(Clone)]
pub struct Notifications {
    games: Games,
    config: Config,
    notified: Arc<Mutex<NotifiedMap>>,
}

impl Notifications {
    pub fn new(
        games: Games,
        config: Config,
    ) -> Self {
        Notifications {
            games,
            config,
            notified: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn clear_notified(&self) {
        let mut notified = self.notified.lock().await;
        *notified = HashMap::new();
    }

    pub async fn push_all_new_games(&self) -> Result<()> {
        let is_notfications_enabled = self.config.is_notifications_enabled().await;

        if !is_notfications_enabled {
            return Ok(());
        }

        let games = self.games.get_all().await;

        for game in games {
            self.push_notification(game).await.context("Failed to push notification")?;
        }

        self.save().await.context("Failed to save notification state")?;

        Ok(())
    }

    pub async fn push_notification(&self, game: Game) -> Result<()> {
        let is_notfications_enabled = self.config.is_notifications_enabled().await;

        if !is_notfications_enabled {
            return Ok(());
        }

        let mut notified = self.notified.lock().await;
        if notified.contains_key(&game.id) {
            return Ok(());
        }

        notified.insert(game.id, Utc::now());

        let image_path = Self::download_image(&game.image).await.context("Failed to download image")?;

        WinToastNotify::new(Config::get_app_id().as_str())
            .set_title(format!("{} ({})", game.title.as_str(), game.platform.as_str()).as_str())
            .set_messages(vec!["Click to claim"])
            .set_image(&image_path)
            .set_open(game.open_giveaway_url.as_str())
            .show()
            .expect("Failed to show notification");

        Ok(())
    }

    pub async fn load_or_init(&self) -> Result<()> {
        let path = Self::get_notifications_log_path();

        if !path.exists() {
            self.clear_notified().await;
            return Ok(());
        }

        let file = match File::open(&path) {
            Ok(f) => f,
            Err(_) => {
                self.clear_notified().await;
                return Ok(());
            },
        };

        let reader = BufReader::new(file);
        let raw : HashMap<usize, i64> = serde_json::from_reader(reader).unwrap_or_default();
        let raw: NotifiedMap = raw.into_iter()
            .map(|(id, ts)| (id, DateTime::<Utc>::from_timestamp(ts, 0).unwrap()))
            .collect();

        let week_ago = Utc::now() - Duration::days(7);
        let new_raw = raw.into_iter()
            .filter(|(_, ts)| *ts > week_ago)
            .collect();

        let mut notified = self.notified.lock().await;
        *notified = new_raw;

        Ok(())
    }

    pub fn get_notifications_log_path() -> PathBuf {
        let mut exe_path = env::current_exe().unwrap();
        exe_path.set_file_name("notifications.log");
        exe_path.set_extension("json");
        exe_path
    }

    async fn download_image(url: &str) -> Result<String> {
        let mut temp_dir = env::temp_dir();
        temp_dir.push("free_tray_games");

        create_dir_all(temp_dir.clone()).context("Failed to create temp directory for images")?;

        let hash_name = format!("{:x}", md5::compute(url));

        let ext = url.split('.').last().unwrap_or("jpg");
        let filename = format!("{}.{}", hash_name, ext);

        let mut file_path = temp_dir.clone();
        file_path.push(filename);

        if file_path.exists() {
            return Ok(file_path.to_str().unwrap().to_string());
        }

        let response = reqwest::get(url).await.context("Failed to download image")?;
        if !response.status().is_success() {
            return Ok("".to_string());
        }

        let content = response.bytes().await.context("Failed to read image content")?;
        let mut file = File::create(file_path.clone()).context("Failed to create image file")?;
        std::io::copy(&mut content.as_ref(), &mut file).context("Failed to write image content")?;

        Ok(file_path.to_str().unwrap().to_string())
    }

    pub async fn save(&self) -> Result<()> {
        let path = Self::get_notifications_log_path();
        let file = File::create(&path).expect("cannot write notified cache");
        let mut writer = BufWriter::new(file);

        let map = self.notified.lock().await.clone();
        let timestamp_map: HashMap<usize, i64> = map.iter()
            .map(|(id, dt)| (*id, dt.timestamp()))
            .collect();

        let json = serde_json::to_string(&timestamp_map).expect("notifications log serialization failed");
        writer.write_all(json.as_bytes()).expect("notifications log write failed");

        Ok(())
    }
}