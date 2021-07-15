use std::{collections::HashMap, fmt::Debug};

use anyhow::bail;
use async_trait::async_trait;
use bincode::{DefaultOptions, Options};
use reqwest::{Client, Url};
use twitch_irc::login::{GetAccessTokenResponse, TokenStorage, UserAccessToken};

use crate::{DB, SETTINGS};

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

impl SledTokenStorage {
  pub fn has_token(&self) -> anyhow::Result<bool> {
    let tokens = DB.get(TOKEN_KEY)?;
    if let None = tokens {
      Ok(false)
    } else {
      Ok(true)
    }
  }
}

#[async_trait]
impl TokenStorage for SledTokenStorage {
  type LoadError = anyhow::Error;
  type UpdateError = anyhow::Error;

  async fn load_token(&mut self) -> Result<UserAccessToken, Self::LoadError> {
    let tokens = DB.get(TOKEN_KEY)?;
    if let None = tokens {
      bail!("no token in database");
    }
    Ok(self.options.deserialize(&tokens.unwrap()).unwrap())
  }

  async fn update_token(&mut self, token: &UserAccessToken) -> Result<(), Self::UpdateError> {
    DB.insert(TOKEN_KEY, self.options.serialize(token).unwrap())?;
    Ok(())
  }
}

pub async fn get_tokens() -> anyhow::Result<GetAccessTokenResponse> {
  let server = tiny_http::Server::http("0.0.0.0:80").unwrap();
  Ok(loop {
    match server.recv() {
      Ok(request) => {
        let url = Url::parse(&format!(
          "{}{}",
          SETTINGS.twitch.redirect_uri,
          request.url()
        ))?;
        let query: HashMap<String, String> = url.query_pairs().into_owned().collect();

        if let Some(code) = query.get("code") {
          let client = Client::new();
          let response = client
            .post("https://id.twitch.tv/oauth2/token")
            .form(&[
              ("client_id", SETTINGS.twitch.client_id.clone()),
              ("client_secret", SETTINGS.twitch.client_secret.clone()),
              ("code", code.to_owned()),
              ("grant_type", "authorization_code".to_owned()),
              ("redirect_uri", SETTINGS.twitch.redirect_uri.clone()),
            ])
            .send()
            .await?;

          let tokens = response.json::<GetAccessTokenResponse>().await?;
          request.respond(tiny_http::Response::empty(200))?;
          break tokens;
        } else {
          request.respond(tiny_http::Response::empty(400))?;
        }
      }
      Err(e) => {
        eprintln!("Error: {}", e);
      }
    };
  })
}
