use std::{sync::mpsc::channel, time::Duration};

use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use once_cell::sync::Lazy;
use sled::Db;
use tokens::SledTokenStorage;
use tokio::sync::mpsc::UnboundedReceiver;
use twitch_irc::{
  login::{RefreshingLoginCredentials, TokenStorage},
  message::ServerMessage,
  ClientConfig, SecureTCPTransport, TwitchIRCClient,
};

use commands::{delete_command, handle_command, update_command};
use settings::Settings;

use crate::commands::create_command;

mod commands;
mod functions;
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
    } else if SETTINGS.debug {
      println!("{:?}", message);
    }
  }
}

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
  // use sled for storing access tokens
  let mut storage = SledTokenStorage::default();
  if !storage.has_token()? {
    println!("no tokens detected, listening...");
    let tokens = tokens::get_tokens().await?;
    println!("ok we got em, thanks.");
    storage.update_token(&tokens.into()).await?;
  }

  // watch commands directory, for hot-reload
  let (tx, rx) = channel();
  let mut watcher = watcher(tx, Duration::from_secs(10))?;
  watcher.watch("./commands", RecursiveMode::NonRecursive)?;

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

  loop {
    match rx.recv() {
      Ok(event) => match event {
        // make sure we're only tracking files containing BobaScript
        DebouncedEvent::Create(path) if path.ends_with(".boba") => {
          create_command(path.as_path()).await;
        }
        DebouncedEvent::NoticeWrite(path) if path.ends_with(".boba") => {
          update_command(path.as_path()).await;
        }
        DebouncedEvent::NoticeRemove(path) if path.ends_with(".boba") => {
          delete_command(path.as_path()).await;
        }
        DebouncedEvent::Rename(old, new) if old.ends_with(".boba") && new.ends_with(".boba") => {
          delete_command(old.as_path()).await;
          create_command(new.as_path()).await;
        }
        _ => {
          if SETTINGS.debug {
            println!("event: {:?}", event)
          }
        }
      },
      Err(e) => {
        eprintln!("error: {:?}", e);
        break;
      }
    }
  }

  join_handle.await?;
  Ok(())
}
