use once_cell::sync::Lazy;
use sled::Db;
use tokens::SledTokenStorage;
use tokio::sync::mpsc::UnboundedReceiver;
use twitch_irc::{
  login::{RefreshingLoginCredentials, TokenStorage},
  message::ServerMessage,
  ClientConfig, SecureTCPTransport, TwitchIRCClient,
};

use commands::handle_command;
use settings::Settings;

mod commands;
mod settings;
mod tokens;

pub static DB: Lazy<Db> = Lazy::new(|| sled::open("somebotty-db").unwrap());
pub static SETTINGS: Lazy<Settings> = Lazy::new(|| Settings::new().unwrap());

async fn read_messages(
  mut incoming_messages: UnboundedReceiver<ServerMessage>,
  client: TwitchIRCClient<SecureTCPTransport, RefreshingLoginCredentials<SledTokenStorage>>,
) {
  while let Some(message) = incoming_messages.recv().await {
    if let ServerMessage::Privmsg(privmsg) = message {
      handle_command(privmsg, client.clone()).await
    } else {
      println!("{:?}", message);
    }
  }
}

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
  let mut storage = SledTokenStorage::default();
  if !storage.has_token()? {
    println!("no tokens detected, listening...");
    let tokens = tokens::get_tokens().await?;
    println!("ok we got em, thanks.");
    storage.update_token(&tokens.into()).await?;
  }

  let config = ClientConfig::new_simple(RefreshingLoginCredentials::new(
    SETTINGS.twitch.username.clone(),
    SETTINGS.twitch.client_id.clone(),
    SETTINGS.twitch.client_secret.clone(),
    storage,
  ));
  let (incoming_messages, client) = TwitchIRCClient::<SecureTCPTransport, _>::new(config);

  // start consuming incoming messages
  let join_handle = tokio::spawn(read_messages(incoming_messages, client.clone()));

  // join channels in settings
  for channel in &SETTINGS.twitch.join_channels {
    client.join(channel.to_owned());
  }

  join_handle.await?;

  Ok(())
}
