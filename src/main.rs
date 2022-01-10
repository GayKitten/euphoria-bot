use std::{collections::HashMap, time::Duration};
use std::env;

use regex::Regex;
#[macro_use]
extern crate lazy_static;

use color_eyre::Result;

use dotenv::dotenv;

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
				prelude::*,
        id::UserId,
        channel::{
        Message,
        },
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

lazy_static!{
	static ref GOOD_GIRL_REGEX: Regex = Regex::new(
		"(?i)good (?:girl|kitt(?:y|en))"
	).unwrap();
}

async fn vibrate_all(client: &ButtplugClient) {
	let devices = client.devices();
	let cmds = devices.iter()
		.map(|d| d.vibrate(VibrateCommand::Speed(1.0))).collect::<Vec<_>>();
	futures::future::join_all(cmds).await;
	tokio::time::sleep(Duration::from_secs(1)).await;
	client.stop_all_devices().await;
}

#[async_trait]
impl EventHandler for Handler {
	async fn ready(&self, _ctx: Context, ready: Ready) {
		println!("{} is connected!", ready.user.name);
	}

	async fn reaction_add(&self, ctx: Context, reaction: Reaction) {
		let channel_id = reaction.channel_id;
		let message_id = reaction.message_id;
		let message = match channel_id.message(&ctx.http, message_id).await {
			Ok(m) => m,
			Err(why) => {
				println!("Couldn't fetch message: {:#?}", why);
				return
			},
		};
		let data = ctx.data.read().await;
		let map = data.get::<ButtplugMap>().expect("Expected buttplug map");
		let victim = &message.author.id;
		let client = match map.get(victim) {
			Some(c) => c,
			None => return,
		};
		vibrate_all(client).await;
	}

	async fn message(&self, ctx: Context, msg: Message) {
		if GOOD_GIRL_REGEX.is_match(&msg.content) {
			let victims = msg.mentions.iter();
			let data = ctx.data.read().await;
			let map = data.get::<ButtplugMap>().expect("Expected buttplug map");
			let fut = victims
				.filter_map(|v| map.get(&v.id))
				.map(|client| vibrate_all(client));
				futures::future::join_all(fut).await;
		}
	}
}

struct ButtplugMap;

impl TypeMapKey for ButtplugMap {
    type Value = HashMap<UserId, ButtplugClient>;
}

struct DatabasePath;

impl TypeMapKey for DatabasePath {
    type Value = String;
}

fn main() -> Result<()> {
	color_eyre::install()?;

	dotenv().ok();

	let framework = StandardFramework::new()
		.configure(|c| c.prefix("a!")) // set the bot's prefix to "a!"
		.group(&GENERAL_GROUP);

	let token = env::var("DISCORD_TOKEN").expect("No discord token");

	println!("token: {:?}", token);

	let rt = tokio::runtime::Builder::new_multi_thread()
		.worker_threads(8).enable_all()
		.build().expect("Couldn't start runtime");

	rt.block_on(async move {
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
	});

	Ok(())
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
            msg.channel_id.say(ctx, "You forgot to give an IP for me to connect to!").await.ok();
            return Ok(());
        }
    };
    if !web_socket_regex.is_match(&arg) {
        msg.channel_id.say(ctx, "Hmm, What you sent doesn't look like an IP...").await.ok();
        return Ok(());
    };
    let ip = format!("ws://{}", arg);
		println!("Connecting to {}", ip);
    let connector = ButtplugRemoteClientConnector::<
            ButtplugWebsocketClientTransport,
            ButtplugClientJSONSerializer,
        >::new(ButtplugWebsocketClientTransport::new_insecure_connector(
            &ip
        ));

    let client = ButtplugClient::new("Euphoria bot");
    if let Err(why) = client.connect(connector).await {
        println!("Couldn't connect: {:#?}", why);
        msg.channel_id.say(ctx, "I couldn't connect with you ~w~").await.ok();
        return Ok(());
    }
    client.start_scanning().await.ok();
    let mut data = ctx.data.write().await;
    let map = data.get_mut::<ButtplugMap>().unwrap();
    map.insert(msg.author.id, client);

    msg.channel_id.say(ctx, "I got connected!").await.ok();
    Ok(())
}

#[command]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let mut data = ctx.data.write().await;
    let map = data.get_mut::<ButtplugMap>().unwrap();
    let client = match map.remove(&msg.author.id) {
        Some(c) => c,
        None => {
            msg.channel_id.say(ctx, "You're not connected!").await.ok();
            return Ok(());
        }
    };
    client.stop_all_devices().await.ok();
    if let Err(why) = client.disconnect().await {
        println!("Couldn't disconnect: {:?}", why);
    };
    msg.channel_id.say(ctx, "You have been disconnected :3").await.ok();
    Ok(())
}

#[command]
async fn stop(ctx: &Context, msg: &Message) -> CommandResult {
    let mut data = ctx.data.write().await;
    let map = data.get_mut::<ButtplugMap>().unwrap();
    if let Some(c) =  map.get(&msg.author.id) {
        c.stop_all_devices().await.ok();
    };
    Ok(())
}

#[command]
async fn please(ctx: &Context, msg: &Message) -> CommandResult {
	let victim = msg.mentions
		.iter().next()
		.unwrap_or(&msg.author);
	
    let mut data = ctx.data.write().await;
    let map = data.get_mut::<ButtplugMap>().unwrap();
    let client = match map.get(&victim.id) {
        Some(c) => c,
        None => {
						let content = {
							if victim.id == msg.author.id {String::from("You're not connected!")}
							else {format!("{} isn't connected!", victim.name)}
						};
            msg.channel_id.say(ctx, content).await.ok();
            return Ok(());
        }
    };
    let devices = client.devices();
    let cmds = devices.iter().map(|d| d.vibrate(VibrateCommand::Speed(1.0)));
    futures::future::join_all(cmds).await;
    msg.channel_id.say(ctx, "brrr~~").await.ok();
		tokio::time::sleep(Duration::from_secs(1)).await;
		client.stop_all_devices().await;
    Ok(())
}
