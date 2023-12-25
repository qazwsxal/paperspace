use platform_dirs::AppDirs;
use serde::{Deserialize, Serialize};
use std::default::Default;
use std::fs::{self};
use std::path::{Path, PathBuf};
use toml;

//TOML crate can't serialize Enums, so be careful here.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config {
    #[serde(default)]
    pub sqlite_config: SqliteConfig,
    #[serde(default)]
    config_path: PathBuf,

}

impl Default for Config {
    fn default() -> Self {
        Self {
            sqlite_config: SqliteConfig::default(),
            config_path: Self::default_dir().join("config.toml"),
        }
    }
}

impl Config {
    pub fn read(config_path: Option<PathBuf>) -> Result<Config, ConfigReadError> {
        let path = config_path.unwrap_or_else(|| Self::default_dir().join("config.toml"));

        let conf_str = fs::read_to_string(path).map_err(ConfigReadError::IOError)?;

        toml::from_str::<Config>(&conf_str).map_err(ConfigReadError::ParseError)
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        let config_string = toml::to_string_pretty(&self).unwrap();
        let config_str = config_string.as_str();
        fs::write(&self.config_path, config_str)
    }
    pub fn default_dir() -> PathBuf {
        let app_dirs = AppDirs::new(Some("paperspace"), true).unwrap();
        fs::create_dir_all(&app_dirs.config_dir).unwrap();
        app_dirs.config_dir
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SqliteConfig {
    pub db_path: PathBuf,
    pub max_connections: u32,
    pub read_only: bool
}

impl Default for SqliteConfig {
    fn default() -> Self {
        Self {
            db_path: Config::default_dir().join("data.db"),
            max_connections: 64,
            read_only: false
        }
    }
}

#[derive(Debug)]
pub enum ConfigReadError {
    ParseError(toml::de::Error),
    IOError(std::io::Error),
}
