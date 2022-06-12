use std::sync::Arc;

use actix::prelude::*;
use buttplug::client::{ButtplugClient, ButtplugClientError, VibrateCommand};
use regex::Regex;
use tokio::{
	sync::{Mutex, RwLock},
	time::{Duration, Instant},
};

#[derive(Debug, Clone, Copy)]
pub enum Decay {
	/// Half life in seconds
	HalfLife(f64),
	/// Time it takes to go from 1 to 0 in seconds
	Linear(f64),
}

#[derive(Debug, Clone, Copy)]
pub struct PowerSettings {
	pub decay: Decay,
	pub praise_hit: f64,
	pub reaction_hit: f64,
}

impl Default for PowerSettings {
	fn default() -> Self {
		Self {
			decay: Decay::HalfLife(1.0),
			praise_hit: 0.3,
			reaction_hit: 0.3,
		}
	}
}

pub struct ButtplugUser {
	power: Option<f64>,
	client: Option<ButtplugClient>,
	last_update: Instant,
	last_praise: Instant,
	voice_praisers: u8,
	sustain: Sustain,
	decay: Decay,
	regex: Regex,
}

impl Actor for ButtplugUser {
	type Context = Context<Self>;
}

enum Sustain {
	Praised,
	Praising,
	Release,
}

impl ButtplugUser {
	pub fn new(client: ButtplugClient) -> Self {
		Self {
			power: None,
			client: Some(client),
			last_update: Instant::now(),
			last_praise: Instant::now(),
			voice_praisers: 0,
			sustain: Sustain::Release,
			decay: Decay::Linear(3.0),
			regex: super::GOOD_GIRL_REGEX.clone(),
		}
	}

	async fn vibrate_all(&self, power: f64) {
		let client = match self.client.as_ref() {
			Some(c) => c,
			None => return,
		};
		let devices = client.devices();
		let fut = devices
			.iter()
			.map(|d| d.vibrate(VibrateCommand::Speed(power)));
		futures::future::join_all(fut).await;
	}

	/// Decay power and send it to the server.
	/// Return indicates if it should sustain.
	pub async fn decay_power(&mut self, decay: Decay) {
		let client = match self.client {
			Some(ref c) => c,
			None => return,
		};
		let now = Instant::now();
		let delta = now.duration_since(self.last_update).as_secs_f64();
		self.last_update = now;
		match self.sustain {
			Sustain::Praising => {
				self.sustain = Sustain::Praised;
				self.vibrate_all(self.power.unwrap_or(0.0)).await;
				self.last_praise = now;
				return;
			}
			Sustain::Praised => {
				if self.last_praise.elapsed() >= Duration::from_secs(1) {
					self.sustain = Sustain::Release;
				}
				return;
			}
			Sustain::Release => {
				// Continue
			}
		}
		let current = match self.power {
			Some(c) => c,
			None => return,
		};
		self.power = get_next_power(current, delta, decay);
		let devices = client.devices();
		match self.power {
			Some(power) => {
				let fut = devices
					.iter()
					.map(|d| d.vibrate(VibrateCommand::Speed(power)));
				futures::future::join_all(fut).await;
			}
			None => {
				if let Err(why) = client.stop_all_devices().await {
					println!("Couldn't stop devices: {:#?}", why);
				}
			}
		}
	}

	pub fn add_power(&mut self, power: f64) {
		self.power = Some(self.power.unwrap_or(0.0) + power);
		self.last_update = Instant::now();
		self.sustain = Sustain::Praising;
	}

	pub fn bump_voice_praisers(&mut self, up: bool) {
		if up {
			self.voice_praisers += 1;
		} else {
			self.voice_praisers -= 1;
		}
		println!("Praisers: {}", self.voice_praisers)
	}

	pub async fn stop(&mut self) {
		let client = match self.client {
			Some(ref c) => c,
			None => return,
		};
		self.power = None;
		if let Err(why) = client.stop_all_devices().await {
			println!("Couldn't stop devices: {:#?}", why);
		}
	}

	/// Because of the arc mutex context the user is in, we can't close and
	/// drop the user, and must instead drop the client, and then on all of
	/// its uses check if it is connected before continuing or dropping it.
	fn check_connected(&self) -> bool {
		self.client.is_some()
	}

	pub async fn disconnect(&mut self) -> Result<(), ButtplugClientError> {
		let client = match self.client.take() {
			Some(c) => c,
			None => return Ok(()),
		};

		if let Err(why) = client.stop_all_devices().await {
			println!("Couldn't stop devices: {:#?}", why);
		}
		client.disconnect().await
	}
}

pub async fn decay_loop(victim: Arc<Mutex<ButtplugUser>>, settings: Arc<RwLock<PowerSettings>>) {
	loop {
		{
			let mut victim_lock = victim.lock().await;
			if !victim_lock.check_connected() {
				return;
			}
			let settings_lock = settings.read().await;
			victim_lock.decay_power(settings_lock.decay).await;
		}
	}
}

struct DecayTick;

impl Message for DecayTick {
	type Result = ();
}

impl Handler<DecayTick> for ButtplugUser {
	type Result = ();

	fn handle(&mut self, msg: DecayTick, ctx: &mut Self::Context) -> Self::Result {
		if self.check_connected() {
			return;
		}
		let fut = self.decay_power(self.decay);
	}
}

fn get_next_power(current: f64, delta: f64, decay: Decay) -> Option<f64> {
	match decay {
		Decay::HalfLife(hl) => {
			if current < 1e-8 {
				None
			} else {
				Some(current * (2.0 as f64).powf(-delta / hl))
			}
		}
		Decay::Linear(time) => {
			let next = current - delta / time;
			if next <= 0.0 {
				None
			} else {
				Some(next)
			}
		}
	}
}
