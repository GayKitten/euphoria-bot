mod user;

use user::{ButtplugUser, PowerSettings, Decay, decay_loop};

use std::sync::Arc;
use std::{collections::HashMap};
use std::env;

use regex::Regex;
#[macro_use]
extern crate lazy_static;
use color_eyre::Result;
use dotenv::dotenv;
use buttplug::{
    client::{
        ButtplugClient,
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
#[commands(ping, join, leave, stop, decay)]
struct General;

struct Handler;

lazy_static!{
	static ref GOOD_GIRL_REGEX: Regex = Regex::new(
		"(?i)good (?:girl|kitt(?:y|en))"
	).unwrap();
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
		let settings: PowerSettings = *data.get::<PowerSettingsKey>().expect("Expected settings").read().await;
		let map = data.get::<ButtplugMap>().expect("Expected buttplug map");
		let victim = &message.author.id;
		let mut victim = match map.get(victim) {
			Some(c) => c.lock().await,
			None => return,
		};
		victim.add_power(settings.reaction_hit);
	}

	async fn message(&self, ctx: Context, msg: Message) {
		if GOOD_GIRL_REGEX.is_match(&msg.content) {
			let data = ctx.data.read().await;
			let settings = *data.get::<PowerSettingsKey>().expect("Expected settings").read().await;
			let map = data.get::<ButtplugMap>().expect("Expected buttplug map");
			let fut = msg.mentions.iter().filter_map(|u| map.get(&u.id).map(|v| Arc::clone(v))).map(|v| async move {
				let mut lock = v.lock().await;
				lock.add_power(settings.praise_hit);
			});

			futures::future::join_all(fut).await;
		}
	}
}

struct ButtplugMap;

impl TypeMapKey for ButtplugMap {
    type Value = HashMap<UserId, Arc<Mutex<ButtplugUser>>>;
}

struct PowerSettingsKey;

impl TypeMapKey for PowerSettingsKey {
	type Value = Arc<RwLock<PowerSettings>>;
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
			data.insert::<PowerSettingsKey>(Arc::new(RwLock::new(PowerSettings::default())));
		}

		// start listening for events by starting a single shard
		if let Err(why) = client.start().await {
			println!("An error occurred while running the client: {:?}", why);
		}
	});

	Ok(())
}

lazy_static! {
	static ref WEB_SOCKET_REGEX : Regex = Regex::new(
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
	if !WEB_SOCKET_REGEX.is_match(&arg) {
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
	let victim = Arc::new(Mutex::new(ButtplugUser::new(client)));
	map.insert(msg.author.id, Arc::clone(&victim));
	let settings = data.get::<PowerSettingsKey>().expect("Expected Settings");
	msg.channel_id.say(ctx, "I got connected!").await.ok();

	tokio::spawn(decay_loop(victim, Arc::clone(settings)));

	Ok(())
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
	msg.reply(ctx, "pong!").await?;

	Ok(())
}

#[command]
async fn decay(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
	let decay = match args.parse::<String>() {
		Ok(s) => s,
		Err(why) => {
			println!("Couldn't parse string: {:#?}", why);
			return Ok(());
		}
	};
	let decay = match decay.as_str() {
		"halflife" => {
			let hl = match args.parse::<f64>() {
				Ok(hl) => hl,
				Err(_) => {
					msg.channel_id.say(ctx, "The halflife needs to be a number!").await.ok();
					return Ok(());
				}
			};
			Decay::HalfLife(hl)
		},
		"linear" => {
			let duration = match args.parse::<f64>() {
				Ok(dur) => dur,
				Err(_) => {
					msg.channel_id.say(ctx, "The duration needs to be a number!").await.ok();
					return Ok(())
				}
			};
			Decay::Linear(duration)
		},
		unsupported => {
			let content = format!("I don't know what {} is!", unsupported);
			msg.channel_id.say(ctx, content).await.ok();
			return Ok(())
		}
	};

	let data = ctx.data.read().await;
	let settings = data.get::<PowerSettingsKey>().expect("Expected settings");
	let mut lock = settings.write().await;
	lock.decay = decay;

	msg.channel_id.say(ctx, "Decay updated!").await.ok();

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
	let mut lock = client.lock().await;
	if let Err(why) = lock.disconnect().await {
		println!("Couldn't disconnect: {:?}", why);
	};
	msg.channel_id.say(ctx, "You have been disconnected :3").await.ok();
	Ok(())
}

#[command]
async fn stop(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let map = data.get::<ButtplugMap>().unwrap();
    if let Some(c) =  map.get(&msg.author.id) {
        c.lock().await.stop().await;
    };
    Ok(())
}
