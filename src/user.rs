
use std::sync::Arc;

use tokio::{
	time::{
		Instant,
		Duration,
		sleep,
	},
	sync::{
		Mutex,
		RwLock
	},
};
use buttplug::client::{ButtplugClient, VibrateCommand, ButtplugClientError};

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
			praise_hit: 0.9,
			reaction_hit: 0.3
		}
	}
}

pub struct ButtplugUser {
	power: Option<f64>,
	client: Option<ButtplugClient>,
	last_update: Instant
}

impl ButtplugUser {
	pub fn new(client: ButtplugClient) -> Self {
		Self {
			power: None,
			client: Some(client),
			last_update: Instant::now()
		}
	}

	/// Decay power and send it to the server.
	pub async fn decay_power(&mut self, decay: Decay) {
		let client = match self.client {
			Some(ref c) => c,
			None => return,
		};
		let now = Instant::now();
		let delta = now.duration_since(self.last_update).as_secs_f64();
		let current = match self.power {
			Some(c) => c,
			None => return,
		};
		self.last_update = now;
		self.power = get_next_power(current, delta, decay);
		let devices = client.devices();
		match self.power {
			Some(power) => {
				let fut = devices.iter().map(|d| d.vibrate(VibrateCommand::Speed(power)));
				futures::future::join_all(fut).await;
			},
			None => {
				if let Err(why) = client.stop_all_devices().await {
					println!("Couldn't stop devices: {:#?}", why);
				}
			},
		}
	}

	pub fn add_power(&mut self, power: f64) {
		self.power = Some(self.power.unwrap_or(0.0) + power);
		self.last_update = Instant::now();
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

pub async fn decay_loop(
		victim: Arc<Mutex<ButtplugUser>>,
		settings: Arc<RwLock<PowerSettings>>
	) {
	loop {
		{
			let mut victim_lock = victim.lock().await;
			if !victim_lock.check_connected() {
				return
			}
			let settings_lock = settings.read().await;
			victim_lock.decay_power(settings_lock.decay).await;
		}
		sleep(Duration::from_millis(20)).await;
	}
}

fn get_next_power(current: f64, delta: f64, decay: Decay) -> Option<f64> {
	match decay {
		Decay::HalfLife(hl) => Some(current * (2.0 as f64).powf(- delta / hl)),
		Decay::Linear(time) => Some((current - delta / time).max(0.0)),
	}
}