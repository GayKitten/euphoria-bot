use std::collections::HashMap;
use std::env;
use std::fs;

use regex::Regex;
#[macro_use]
extern crate lazy_static;

use buttplug::{
    client::{
        ButtplugClient,
        device::VibrateCommand,
    },
    connector::{ButtplugRemoteClientConnector, ButtplugWebsocketClientTransport},
    core::messages::serializer::ButtplugClientJSONSerializer,
};

use serenity::{
    prelude::*,
    async_trait,
    client::{
        Client, Context, EventHandler,
    },
    model::{
        id::UserId,
        channel::{
        Message,
        }
    },
    framework::standard::{
        Args, CommandResult, StandardFramework,
        macros::{
            command, group
        }
    },
};

#[group]
#[commands(join, leave, stop, please)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {}

struct ButtplugMap;

impl TypeMapKey for ButtplugMap {
    type Value = HashMap<UserId, ButtplugClient>;
}

struct DatabasePath;

impl TypeMapKey for DatabasePath {
    type Value = String;
}

#[tokio::main]
async fn main() {
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("a!")) // set the bot's prefix to "a!"
        .group(&GENERAL_GROUP);

    // Get token from file
    let mut args = env::args().collect::<Vec<String>>().into_iter();
    let token = fs::read_to_string(args.next().expect("Need two arguments!"))
        .expect("Couldn't read file!");


    // Login with a bot token
    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<ButtplugMap>(HashMap::default());
    }

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}

lazy_static! {
    static ref web_socket_regex : Regex = Regex::new(
        r"((25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9]?[0-9])\.){3}(25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9]?[0-9]):\d{5}"
    ).unwrap();
}

#[command]
async fn join(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let arg = match args.single::<String>() {
        Ok(a) => a,
        Err(_) => {
            msg.channel_id.say(ctx, "You forgot to give an IP for me to connect to!").await;
            return Ok(());
        }
    };
    if !web_socket_regex.is_match(&arg) {
        msg.channel_id.say(ctx, "Hmm, What you sent doesn't look like an IP...").await;
        return Ok(());
    };
    let ip = format!("ws://{}", arg);
    let connector = ButtplugRemoteClientConnector::<
            ButtplugWebsocketClientTransport,
            ButtplugClientJSONSerializer,
        >::new(ButtplugWebsocketClientTransport::new_insecure_connector(
            &ip
        ));

    let client = ButtplugClient::new("Euphoria bot");
    if let Err(why) = client.connect(connector).await {
        println!("Couldn't connect: {:?}", why);
        msg.channel_id.say(ctx, "I couldn't connect with you ~w~").await;
        return Ok(());
    }
    client.start_scanning().await;
    let mut data = ctx.data.write().await;
    let map = data.get_mut::<ButtplugMap>().unwrap();
    map.insert(msg.author.id, client);

    msg.channel_id.say(ctx, "I got connected!").await;
    Ok(())
}

#[command]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let mut data = ctx.data.write().await;
    let map = data.get_mut::<ButtplugMap>().unwrap();
    let client = match map.remove(&msg.author.id) {
        Some(c) => c,
        None => {
            msg.channel_id.say(ctx, "You're not connected!").await;
            return Ok(());
        }
    };
    client.stop_all_devices().await;
    if let Err(why) = client.disconnect().await {
        println!("Couldn't disconnect: {:?}", why);
    };
    msg.channel_id.say(ctx, "You have been disconnected :3").await;
    Ok(())
}

#[command]
async fn stop(ctx: &Context, msg: &Message) -> CommandResult {
    let mut data = ctx.data.write().await;
    let map = data.get_mut::<ButtplugMap>().unwrap();
    if let Some(c) =  map.get(&msg.author.id) {
        c.stop_all_devices().await;
    };
    Ok(())
}

#[command]
async fn please(ctx: &Context, msg: &Message) -> CommandResult {
    let mut data = ctx.data.write().await;
    let map = data.get_mut::<ButtplugMap>().unwrap();
    let client = match map.get(&msg.author.id) {
        Some(c) => c,
        None => {
            msg.channel_id.say(ctx, "You're not connected!!!").await;
            return Ok(());
        }
    };
    let devices = client.devices();
    let cmds = devices.iter().map(|d| d.vibrate(VibrateCommand::Speed(1.0))).collect::<Vec<_>>();
    futures::future::join_all(cmds).await;
    msg.channel_id.say(ctx, "brrr~~").await;
    Ok(())
}
