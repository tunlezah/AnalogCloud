//! Persistent settings store. Reads and writes a single TOML file at
//! `$XDG_CONFIG_HOME/analog-cloud/config.toml`.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use parking_lot::RwLock;
use thiserror::Error;
use tokio::fs;

use analog_cloud_shared::settings::Settings;

#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("toml decode: {0}")]
    Decode(#[from] toml::de::Error),
    #[error("toml encode: {0}")]
    Encode(#[from] toml::ser::Error),
    #[error("no config directory available on this platform")]
    NoConfigDir,
}

pub type Result<T> = std::result::Result<T, SettingsError>;

#[derive(Clone)]
pub struct SettingsStore {
    inner: Arc<Inner>,
}

struct Inner {
    path: PathBuf,
    settings: RwLock<Settings>,
}

impl SettingsStore {
    /// Open the store at the platform-default location.
    pub async fn open_default() -> Result<Self> {
        let path = default_config_path()?;
        Self::open_at(path).await
    }

    /// Open the store at an explicit path.
    pub async fn open_at(path: PathBuf) -> Result<Self> {
        let settings = match fs::read_to_string(&path).await {
            Ok(contents) => toml::from_str(&contents)?,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                tracing::info!(?path, "no config file yet, writing defaults");
                let defaults = Settings::default();
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent).await.ok();
                }
                fs::write(&path, toml::to_string_pretty(&defaults)?).await?;
                defaults
            }
            Err(err) => return Err(err.into()),
        };

        Ok(Self {
            inner: Arc::new(Inner {
                path,
                settings: RwLock::new(settings),
            }),
        })
    }

    pub fn snapshot(&self) -> Settings {
        self.inner.settings.read().clone()
    }

    pub async fn update<F>(&self, mutate: F) -> Result<Settings>
    where
        F: FnOnce(&mut Settings),
    {
        let new = {
            let mut guard = self.inner.settings.write();
            mutate(&mut *guard);
            guard.clone()
        };
        let text = toml::to_string_pretty(&new)?;
        fs::write(&self.inner.path, text).await?;
        tracing::info!(path = ?self.inner.path, "settings persisted");
        Ok(new)
    }

    pub fn path(&self) -> &Path {
        &self.inner.path
    }
}

fn default_config_path() -> Result<PathBuf> {
    let dirs = directories::ProjectDirs::from("cloud", "Analog", "analog-cloud")
        .ok_or(SettingsError::NoConfigDir)?;
    Ok(dirs.config_dir().join("config.toml"))
}
