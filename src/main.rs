use once_cell::sync::Lazy;
use sled::Db;
use tokens::SledTokenStorage;
use tokio::sync::mpsc::UnboundedReceiver;
use twitch_irc::{
  login::RefreshingLoginCredentials, message::ServerMessage, ClientConfig, SecureTCPTransport,
  TwitchIRCClient,
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
    }
  }
}

#[tokio::main]
pub async fn main() {
  let storage = SledTokenStorage::default();
  let config = ClientConfig::new_simple(RefreshingLoginCredentials::new(
    SETTINGS.twitch.username.clone(),
    SETTINGS.twitch.client_id.clone(),
    SETTINGS.twitch.client_secret.clone(),
    storage,
  ));
  let (incoming_messages, client) = TwitchIRCClient::<SecureTCPTransport, _>::new(config);

  // start consuming incoming messages
  let join_handle = tokio::spawn(read_messages(incoming_messages, client.clone()));

  client.join("sand_head".to_owned());

  join_handle.await.unwrap();
}
