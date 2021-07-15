use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TwitchSettings {
  pub username: String,
  pub client_id: String,
  pub client_secret: String,
  pub redirect_uri: String,
  pub join_channels: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
  pub debug: bool,
  pub twitch: TwitchSettings,
}

impl Settings {
  pub fn new() -> Result<Self, ConfigError> {
    let mut settings = Config::default();

    settings.merge(File::with_name("settings"))?;
    settings.merge(File::with_name("settings-local").required(false))?;
    settings.merge(Environment::with_prefix("app"))?;

    settings.try_into()
  }
}
