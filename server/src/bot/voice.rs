use std::{collections::HashMap, sync::Arc};

use futures::stream::iter;
use futures::stream::StreamExt;
use serenity::{async_trait, model::id::UserId, prelude::Mutex};

use songbird::{
	events::context_data::SpeakingUpdateData, model::payload::Speaking, Call, CoreEvent, Event,
	EventContext, EventHandler as VoiceEventHandler,
};

use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use super::{user::ButtplugUser, ButtplugMap};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum VoiceEvent {
	AddUser(u32, UserId),
	/// Whenever someone starts or stops speaking.
	SpeakingState(u32, bool),
}

pub struct Voicecall {
	/// Map between the ssrc id, identifying each caller, to their user ID.
	ssrc_map: HashMap<u32, UserId>,
	/// Buttplug map from UserId to Buttplug User.
	/// Used to get the user's buttplug as they join VC.
	map: ButtplugMap,
	/// Buttplug users in the call.
	victims: Vec<(UserId, Arc<Mutex<ButtplugUser>>)>,
	/// Event receiver.
	rx: UnboundedReceiver<VoiceEvent>,
}

impl Voicecall {
	fn new(map: ButtplugMap) -> (Self, VoiceHandler) {
		let (tx, rx) = unbounded_channel();
		let vc = Voicecall {
			ssrc_map: HashMap::default(),
			map,
			victims: Default::default(),
			rx,
		};
		(vc, VoiceHandler(tx))
	}

	async fn listen(mut self) {
		while let Some(event) = self.rx.recv().await {
			match event {
				VoiceEvent::AddUser(ssrc, user) => {
					self.ssrc_map.insert(ssrc, user);
				}
				VoiceEvent::SpeakingState(ssrc, speaking) => {
					let user = self.ssrc_map.get(&ssrc);
					let victims = self.victims.iter().filter_map(|(id, victim)| {
						if user.map(|uid| uid != id).unwrap_or(true) {
							Some(victim)
						} else {
							None
						}
					});
					iter(victims)
						.for_each_concurrent(None, |v| async move {
							let mut lock = v.lock().await;
							lock.bump_voice_praisers(speaking)
						})
						.await;
				}
			}
		}
	}
}

#[derive(Clone)]
pub struct VoiceHandler(UnboundedSender<VoiceEvent>);

#[async_trait]
impl VoiceEventHandler for VoiceHandler {
	async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
		use EventContext as Ctx;
		match ctx {
			// New person speaking, map Ssrc to id.
			Ctx::SpeakingStateUpdate(Speaking { ssrc, user_id, .. }) => {
				if let Some(id) = user_id {
					let user_id = UserId(id.0);
					let res = self.0.send(VoiceEvent::AddUser(*ssrc, user_id));
					dbg!(res).ok();
				}
			}
			&Ctx::SpeakingUpdate(SpeakingUpdateData { ssrc, speaking, .. }) => {
				self.0.send(VoiceEvent::SpeakingState(ssrc, speaking)).ok();
			}
			_ => (),
		}
		None
	}
}

pub fn register_events(call: &mut Call, map: ButtplugMap) {
	let (vc, handler) = Voicecall::new(map);

	call.add_global_event(CoreEvent::SpeakingStateUpdate.into(), handler.clone());

	call.add_global_event(CoreEvent::SpeakingUpdate.into(), handler.clone());

	tokio::spawn(vc.listen());
}
