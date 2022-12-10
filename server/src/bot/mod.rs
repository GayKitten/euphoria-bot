//mod flirting;
//mod voice;

use std::{env, sync::Arc};

use futures::{FutureExt, StreamExt};
use log::info;
use tokio::{select, sync::Notify};
use twilight_gateway::{Event, EventTypeFlags, Intents, Shard};
use twilight_model::{
	channel::Message,
	gateway::{
		payload::outgoing::UpdatePresence,
		presence::{Activity, ActivityType, MinimalActivity, Status},
		GatewayReaction,
	},
};

use crate::{manager::Manager, user::Flirt};

pub async fn run_bot(manager: Arc<Manager>, notify_term: Arc<Notify>) -> Result<(), anyhow::Error> {
	let intents =
		Intents::GUILD_MESSAGES | Intents::MESSAGE_CONTENT | Intents::GUILD_MESSAGE_REACTIONS;
	let event_types = EventTypeFlags::MESSAGE_CREATE | EventTypeFlags::REACTION_ADD;

	let token = env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN var");

	let (shard, mut events) = Shard::builder(token, intents)
		.event_types(event_types)
		.build();

	let shard = Arc::new(shard);

	shard.start().await.expect("failed to start shard");

	let minimal_activity = MinimalActivity {
		kind: ActivityType::Custom,
		name: "Brrr".into(),
		url: None,
	};

	let update_presence = UpdatePresence::new(
		vec![Activity::from(minimal_activity)],
		false,
		None,
		Status::Online,
	)
	.expect("Failed to create UpdatePresence");

	let shard_clone = shard.clone();

	tokio::spawn(async move {
		tokio::time::sleep(std::time::Duration::from_secs(2)).await;
		shard_clone
			.command(&update_presence)
			.await
			.expect("Failed to send UpdatePresence");
		info!("Sent UpdatePresence");
	});

	loop {
		select! {
			Some(event) = events.next() => {
				match event {
					Event::MessageCreate(message) => {
						tokio::spawn(handle_message(message.0, manager.clone()));
					}
					Event::ReactionAdd(reaction) => {
						tokio::spawn(handle_reaction(reaction.0, manager.clone()));
					}
					_ => {}
				}
			}
			_ = notify_term.notified().fuse() => {
				info!("Shutting down");
				shard.shutdown();
				break;
			}
		}
	}
	Ok(())
}

async fn handle_message(message: Message, manager: Arc<Manager>) {
	message
		.mentions
		.iter()
		.filter_map(|mention| {
			manager.get(mention.id).map(|a| {
				info!("Brr-ing user: {}", mention.name);
				a
			})
		})
		.for_each(|user| user.do_send(Flirt(message.content.clone())));
}

async fn handle_reaction(reaction: GatewayReaction, manager: Arc<Manager>) {}
