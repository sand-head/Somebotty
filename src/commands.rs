use std::{collections::HashMap, fs};

use bobascript::{compiler, vm::VM};
use if_chain::if_chain;
use once_cell::sync::Lazy;
use twitch_irc::{
  login::StaticLoginCredentials, message::PrivmsgMessage, SecureTCPTransport, TwitchIRCClient,
};

pub static COMMANDS: Lazy<HashMap<String, String>> = Lazy::new(get_commands);

fn get_commands() -> HashMap<String, String> {
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

  println!("found {} commands...", boba_files.len());
  for entry in boba_files {
    let entry = entry.unwrap();
    let contents = fs::read_to_string(entry.path()).unwrap();
    let name = entry.file_name().into_string().unwrap();
    let name = &name[..(name.len() - 5)];
    println!("loaded {} command", name);
    map.insert(name.to_string(), format!("{{{}}}", contents));
  }

  map
}

pub async fn handle_message(
  message: PrivmsgMessage,
  client: TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
) {
  if_chain! {
    if let Some(command) = message.message_text.split(' ').nth(0);
    let command = COMMANDS.get(command);
    if let Some(command) = command;
    then {
      let value = {
        let mut vm = VM::default();
        let function = compiler::compile_expr(command).unwrap();
        vm.evaluate(function).unwrap().to_string()
      };
      println!("sending: {:?}", value);
      client.say("sand_head".to_owned(), value).await.unwrap();
    }
  }
}
