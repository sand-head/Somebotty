use tokio::sync::mpsc::UnboundedReceiver;
use twitch_irc::{
  login::StaticLoginCredentials, message::ServerMessage, ClientConfig, SecureTCPTransport,
  TwitchIRCClient,
};

use commands::handle_message;

mod commands;

async fn read_messages(
  mut incoming_messages: UnboundedReceiver<ServerMessage>,
  client: TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
) {
  while let Some(message) = incoming_messages.recv().await {
    if let ServerMessage::Privmsg(privmsg) = message {
      handle_message(privmsg, client.clone()).await
    }
  }
}

#[tokio::main]
pub async fn main() {
  let config = ClientConfig::default();
  let (incoming_messages, client) =
    TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);

  // start consuming incoming messages
  let join_handle = tokio::spawn(read_messages(incoming_messages, client.clone()));

  client.join("sand_head".to_owned());

  join_handle.await.unwrap();
}
