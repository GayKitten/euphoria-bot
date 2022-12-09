use std::{
	collections::HashMap,
	sync::Arc,
	time::{Duration, Instant},
};

use actix::{Actor, Context as ActorContext, Handler, Message as ActixMessage};
use serenity::{
	client::Context,
	model::{
		channel::Message,
		id::{ChannelId, UserId},
	},
};

/// If someone hasn't flirted inside the flirt window, the next implicit flirt doesn't count.
const FLIRT_WINDOW: Duration = Duration::from_secs(300);

pub struct ChannelContext {
	users: HashMap<UserId, FlirtingUser>,
}

impl ChannelContext {}

/// After a user has done an explicit flirt, they may implicitly flirt after for a while.
pub struct FlirtingUser {
	/// Last time the user has flirted.
	/// After a while of no flirting, the implicit flirting should expire.
	last_flirt: Instant,
	/// Users for which implicit flirts will affect.
	flirting_with: Vec<UserId>,
}

impl FlirtingUser {
	pub fn new() {}

	pub fn is_valid_flirt(&self, now: Instant) -> bool {
		self.last_flirt + FLIRT_WINDOW > now
	}
}

pub async fn flirt_msg(ctx: Context, msg: Message) {
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

#[derive(Default)]
pub struct ChannelContextManager {
	channels: HashMap<ChannelId, ChannelContext>,
}

impl Actor for ChannelContextManager {
	type Context = ActorContext<Self>;
}

#[derive(ActixMessage)]
#[rtype(return = "()")]
struct ImplicitFlirt(pub Message);

impl Handler<ImplicitFlirt> for ChannelContextManager {
	type Result = ();

	fn handle(&mut self, msg: ImplicitFlirt, _ctx: &mut Self::Context) -> Self::Result {
		if let Some(flirter) = self
			.channels
			.get_mut(&msg.0.channel_id)
			.and_then(|channel| channel.users.get_mut(&msg.0.author.id))
		{
			flirter.last_flirt = Instant::now();
			for flirtee in flirter.flirting_with.iter() {}
		}
	}
}

#[derive(ActixMessage)]
#[rtype(return = "()")]
struct ExplicitFlirt(pub Message);

impl Handler<ExplicitFlirt> for ChannelContextManager {
	type Result = ();

	fn handle(&mut self, msg: ExplicitFlirt, _ctx: &mut Self::Context) -> Self::Result {
		todo!()
	}
}
