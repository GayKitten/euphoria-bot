mod user;
mod voice;
mod flirting;

use std::{env, sync::Arc, collections::HashMap};

use user::{decay_loop, ButtplugUser, Decay, PowerSettings};
use voice::register_events;

use regex::Regex;

use buttplug::{
	client::ButtplugClient,
	connector::{ButtplugRemoteClientConnector, ButtplugWebsocketClientTransport},
	core::messages::serializer::ButtplugClientJSONSerializer,
};

use serenity::{
	async_trait,
	client::{Client, Context, EventHandler},
	framework::standard::{
		macros::{command, group},
		Args, CommandResult, StandardFramework,
	},
	model::{channel::Message, id::UserId, prelude::*},
	prelude::*,
	Result as SResult,
};

use songbird::{driver::DecodeMode, Config, SerenityInit};

pub type ButtplugMap = Arc<RwLock<HashMap<UserId, Arc<Mutex<ButtplugUser>>>>>;

struct ButtplugMapKey;

impl TypeMapKey for ButtplugMapKey {
	type Value = ButtplugMap;
}

struct PowerSettingsKey;

impl TypeMapKey for PowerSettingsKey {
	type Value = Arc<RwLock<PowerSettings>>;
}

struct DatabaseKey;

impl TypeMapKey for DatabaseKey {
	type Value = Mutex<Connection>;
}


#[group]
#[commands(ping, join, leave, stop, decay, settings)]
struct General;

struct Handler;

lazy_static! {
	static ref GOOD_GIRL_REGEX: Regex =
		Regex::new("(?i)(?:good (?:girl|kitt(?:y|en))|treat|reward|praise|slut|cum)").unwrap();
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
				return;
			}
		};
		let data = ctx.data.read().await;
		let settings: PowerSettings = *data
			.get::<PowerSettingsKey>()
			.expect("Expected settings")
			.read()
			.await;
		let map = data.get::<ButtplugMapKey>().expect("Expected buttplug map");
		let lock = map.read().await;
		let victim = &message.author.id;
		let mut victim = match lock.get(victim) {
			Some(c) => c.lock().await,
			None => return,
		};
		victim.add_power(settings.reaction_hit);
	}

	async fn message(&self, ctx: Context, msg: Message) {
		if msg.mentions.len() != 0 && GOOD_GIRL_REGEX.is_match(&msg.content) {
			let count = GOOD_GIRL_REGEX.find_iter(&msg.content).count();
			let data = ctx.data.read().await;
			let settings = *data
				.get::<PowerSettingsKey>()
				.expect("Expected settings")
				.read()
				.await;
			let map = data.get::<ButtplugMapKey>().expect("Expected buttplug map");
			let lock = map.read().await;
			let fut = msg
				.mentions
				.iter()
				.filter_map(|u| lock.get(&u.id).map(|v| Arc::clone(v)))
				.map(|v| async move {
					let mut lock = v.lock().await;
					lock.add_power(settings.praise_hit * (count as f64));
				});

			futures::future::join_all(fut).await;
		}
	}
}

pub async fn run_bot() -> Result<(), anyhow::Error> {
	let framework = StandardFramework::new()
		.configure(|c| c.prefix("a!")) // set the bot's prefix to "a!"
		.group(&GENERAL_GROUP);

	let songbird_config = Config::default().decode_mode(DecodeMode::Decrypt);

	let token = env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN var");

	let mut client = Client::builder(token)
		.event_handler(Handler)
		.framework(framework)
		.register_songbird_from_config(songbird_config)
		.await
		.expect("Error creating client");

	let res: Result<(), anyhow::Error> = client.start().await.map_err(anyhow::Error::from);
	res
}


lazy_static! {
	static ref WEB_SOCKET_REGEX : Regex = Regex::new(
		r"((25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9]?[0-9])\.){3}(25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9]?[0-9]):\d{0,5}"
	).unwrap();
}

#[command]
async fn join(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
	let arg = match args.single::<String>() {
		Ok(a) => a,
		Err(_) => {
			msg_result(
				msg.channel_id
					.say(ctx, "You forgot to give an IP for me to connect to!")
					.await,
			);
			return Ok(());
		}
	};
	if !WEB_SOCKET_REGEX.is_match(&arg) {
		msg_result(
			msg.channel_id
				.say(ctx, "Hmm, What you sent doesn't look like an IP...")
				.await,
		);
		return Ok(());
	};
	let ip = format!("ws://{}", arg);
	println!("Connecting to {}", ip);
	let connector = ButtplugRemoteClientConnector::<
		ButtplugWebsocketClientTransport,
		ButtplugClientJSONSerializer,
	>::new(ButtplugWebsocketClientTransport::new_insecure_connector(
		&ip,
	));

	let client = ButtplugClient::new("Euphoria bot");
	if let Err(why) = client.connect(connector).await {
		println!("Couldn't connect: {:#?}", why);
		msg.channel_id
			.say(ctx, "I couldn't connect with you ~w~")
			.await
			.ok();
		return Ok(());
	}
	client.start_scanning().await.ok();
	let data = ctx.data.read().await;
	let victim = Arc::new(Mutex::new(ButtplugUser::new(client)));
	let map = data.get::<ButtplugMapKey>().unwrap();
	let mut lock = map.write().await;
	lock.insert(msg.author.id, Arc::clone(&victim));
	let settings = data.get::<PowerSettingsKey>().expect("Expected Settings");
	msg_result(msg.channel_id.say(ctx, "I got connected!").await);

	tokio::spawn(decay_loop(victim, Arc::clone(settings)));

	Ok(())
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
	msg.reply(ctx, "pong!").await?;
	Ok(())
}

#[command]
async fn decay(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
	let decay = match args.single::<String>() {
		Ok(s) => s,
		Err(why) => {
			println!("Couldn't parse string: {:#?}", why);
			return Ok(());
		}
	};
	let decay = match decay.as_str() {
		"hl" | "halflife" => {
			let hl = match args.parse::<f64>() {
				Ok(hl) => hl,
				Err(_) => {
					msg_result(
						msg.channel_id
							.say(ctx, "The halflife needs to be a number!")
							.await,
					);
					return Ok(());
				}
			};
			Decay::HalfLife(hl)
		}
		"linear" => {
			let duration = match args.parse::<f64>() {
				Ok(dur) => dur,
				Err(_) => {
					msg_result(
						msg.channel_id
							.say(ctx, "The duration needs to be a number!")
							.await,
					);
					return Ok(());
				}
			};
			Decay::Linear(duration)
		}
		unsupported => {
			let content = format!("I don't know what {} is!", unsupported);
			msg_result(msg.channel_id.say(ctx, content).await);
			return Ok(());
		}
	};

	let data = ctx.data.read().await;
	let settings = data.get::<PowerSettingsKey>().expect("Expected settings");
	let mut lock = settings.write().await;
	lock.decay = decay;

	msg_result(msg.channel_id.say(ctx, "Decay updated!").await);

	Ok(())
}

#[command]
async fn settings(ctx: &Context, msg: &Message) -> CommandResult {
	let data = ctx.data.read().await;
	let settings = *data
		.get::<PowerSettingsKey>()
		.expect("Expected settings")
		.read()
		.await;
	let content = format!("Current decay: {:?}", settings.decay);
	msg_result(msg.channel_id.say(ctx, content).await);
	Ok(())
}

#[command]
async fn regex(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
	let data = ctx.data.read().await;
	let map = data.get::<ButtplugMapKey>().unwrap();
	let lock = map.read().await;
	let victim = if let Some(v) = lock.get(&msg.author.id) {
		v
	} else {
		msg_result(msg.channel_id.say(ctx, "You're not connected!").await);
		return Ok(());
	};
	if args.len() == 0 {
		msg_result(msg.channel_id.say(ctx, "").await);
		return Ok(());
	}
	Ok(())
}

#[command]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
	let data = ctx.data.read().await;
	let map = data.get::<ButtplugMapKey>().unwrap();
	let mut lock = map.write().await;
	let client = match lock.remove(&msg.author.id) {
		Some(c) => c,
		None => {
			msg_result(msg.channel_id.say(ctx, "You're not connected!").await);
			return Ok(());
		}
	};
	let mut lock = client.lock().await;
	if let Err(why) = lock.disconnect().await {
		println!("Couldn't disconnect: {:?}", why);
	};
	msg_result(
		msg.channel_id
			.say(ctx, "You have been disconnected :3")
			.await,
	);
	Ok(())
}

#[command]
async fn stop(ctx: &Context, msg: &Message) -> CommandResult {
	let data = ctx.data.read().await;
	let map = data.get::<ButtplugMapKey>().unwrap();
	let lock = map.read().await;
	if let Some(c) = lock.get(&msg.author.id) {
		c.lock().await.stop().await;
	};
	Ok(())
}

#[command]
async fn vc(ctx: &Context, msg: &Message) -> CommandResult {
	let guild = match msg.guild(ctx).await {
		Some(g) => g,
		None => return Ok(()),
	};

	let guild_id = guild.id;

	let voice_channel = guild
		.voice_states
		.get(&msg.author.id)
		.and_then(|vs| vs.channel_id);

	let channel_id = match voice_channel {
		Some(c) => c,
		None => {
			msg_result(msg.channel_id.say(ctx, "You're not in vc!").await);
			return Ok(());
		}
	};

	let manager = songbird::get(ctx)
		.await
		.expect("Songbird created during initialisation.");

	let (handler_lock, con_result) = manager.join(guild_id, channel_id).await;

	if let Ok(_) = con_result {
		let mut handler = handler_lock.lock().await;
		let buttplug_map = {
			let data = ctx.data.read().await;
			Arc::clone(
				data.get::<ButtplugMapKey>()
					.expect("Map inserted at startup"),
			)
		};
		register_events(&mut handler, buttplug_map);
	}

	Ok(())
}

fn msg_result(res: SResult<Message>) {
	if let Err(why) = res {
		println!("Couldn't send message: {:?}", why);
	}
}
