use std::fmt::Debug;

use async_trait::async_trait;
use bincode::{DefaultOptions, Options};
use twitch_irc::login::{TokenStorage, UserAccessToken};

use crate::DB;

const TOKEN_KEY: &'static str = "user_access_token";

pub struct SledTokenStorage {
  options: DefaultOptions,
}
impl Default for SledTokenStorage {
  fn default() -> Self {
    Self {
      options: DefaultOptions::new(),
    }
  }
}
impl Debug for SledTokenStorage {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "SledTokenStorage")
  }
}

#[async_trait]
impl TokenStorage for SledTokenStorage {
  type LoadError = sled::Error;
  type UpdateError = sled::Error;

  async fn load_token(&mut self) -> Result<UserAccessToken, Self::LoadError> {
    Ok(
      self
        .options
        .deserialize(&DB.get(TOKEN_KEY)?.unwrap())
        .unwrap(),
    )
  }

  async fn update_token(&mut self, token: &UserAccessToken) -> Result<(), Self::UpdateError> {
    DB.insert(TOKEN_KEY, self.options.serialize(token).unwrap())?;
    Ok(())
  }
}
