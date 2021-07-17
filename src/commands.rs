use std::{collections::HashMap, fs, path::Path};

use bobascript::{compiler, value::Value, vm::VM};
use if_chain::if_chain;
use once_cell::sync::{Lazy, OnceCell};
use tokio::sync::RwLock;
use twitch_irc::{
  login::RefreshingLoginCredentials, message::PrivmsgMessage, SecureTCPTransport, TwitchIRCClient,
};

use crate::{functions::add_functions, tokens::SledTokenStorage};

const PREFIX: char = '!';
// I really want to compile expressions...
static COMMANDS: Lazy<RwLock<HashMap<String, OnceCell<String>>>> =
  Lazy::new(|| RwLock::new(get_commands()));

fn get_commands() -> HashMap<String, OnceCell<String>> {
  println!("getting commands...");
  let mut map = HashMap::new();
  let boba_files: Vec<_> = fs::read_dir("./commands")
    .unwrap()
    .filter(|e| {
      if let Some(ext) = e.as_ref().unwrap().path().extension() {
        ext.to_str().unwrap() == "boba"
      } else {
        false
      }
    })
    .collect();

  for entry in &boba_files {
    let entry = entry.as_ref().unwrap();
    let name = entry.file_name().into_string().unwrap();
    let name = format!("{}{}", PREFIX, &name[..(name.len() - 5)]);
    map.insert(name, OnceCell::new());
  }
  println!("found {} commands.", boba_files.len());

  map
}

pub async fn get_command(name: &'_ str) -> Option<String> {
  let commands = COMMANDS.read().await;
  commands.get(name).map(|command| {
    command
      .get_or_init(|| {
        println!("loading {} command", name);
        fs::read_to_string(format!("./commands/{}.boba", &name[1..]))
          .map(|c| format!("{{{}}}", c))
          .unwrap()
      })
      .clone()
  })
}

pub async fn create_command(path: &'_ Path) {
  let name = path.file_name().unwrap().to_str().unwrap();
  let name = format!("{}{}", PREFIX, &name[..(name.len() - 5)]);
  println!("loaded {} command", name);
  let mut commands = COMMANDS.write().await;
  commands.insert(name, OnceCell::new());
}

pub async fn delete_command(path: &'_ Path) {
  let name = path.file_name().unwrap().to_str().unwrap();
  let name = format!("{}{}", PREFIX, &name[..(name.len() - 5)]);
  println!("deleted {} command", name);
  let mut commands = COMMANDS.write().await;
  commands.remove(&name);
}

pub async fn update_command(path: &'_ Path) {
  delete_command(path).await;
  create_command(path).await;
}

pub async fn handle_command(
  message: PrivmsgMessage,
  client: TwitchIRCClient<SecureTCPTransport, RefreshingLoginCredentials<SledTokenStorage>>,
) {
  if_chain! {
    if let Some(command_name) = message.message_text.split(' ').next();
    let command = get_command(command_name).await;
    if let Some(command) = command;
    then {
      let value = {
        let mut vm = VM::default();
        add_functions(&mut vm);
        let function = compiler::compile_expr(command).unwrap();
        let value = vm.evaluate(function).unwrap();

        if let Value::String(str) = value {
          str
        } else {
          value.to_string()
        }
      };
      println!("sending: {:?}", value);
      client.say("sand_head".to_owned(), value).await.unwrap();
    }
  }
}
