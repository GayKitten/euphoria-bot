use actix::prelude::*;
use actix_buttplug::ButtplugContext;
use buttplug::client::{ButtplugClientError, ButtplugClientEvent, VibrateCommand};
use futures::future::join_all;
use log::{error, info};
use regex::Regex;
use tokio::time::{Duration, Instant};

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
	last_update: Instant,
	last_praise: Instant,
	sustain: Sustain,
	decay: Decay,
	regex: Regex,
}

impl Actor for ButtplugUser {
	type Context = ButtplugContext<Self>;

	fn started(&mut self, ctx: &mut Self::Context) {
		println!("Started actor!");
		ctx.run_interval(Duration::from_millis(1), |user, ctx| {
			let power = user.decay_power();
			match power {
				Power::Set(power) => {
					let futs = ctx.devices().into_iter().map(move |d| async move {
						if let Err(e) = d.vibrate(VibrateCommand::Speed(power)).await {
							error!("Error vibrating device: {:?}", e)
						}
					});
					let fut = async move {
						join_all(futs).await;
					};
					ctx.spawn(fut.into_actor(user));
				}
				Power::Sustain => {}
				Power::Off => {
					ctx.stop_all_devices();
				}
			}
		});
	}
}

impl StreamHandler<ButtplugClientEvent> for ButtplugUser {
	fn handle(&mut self, item: ButtplugClientEvent, ctx: &mut Self::Context) {}
}

enum Sustain {
	Praised,
	Praising,
	Release,
}

#[derive(Debug, Clone, Copy)]
enum Power {
	Set(f64),
	Sustain,
	Off,
}

impl ButtplugUser {
	pub fn new() -> Self {
		Self {
			power: None,
			last_update: Instant::now(),
			last_praise: Instant::now(),
			sustain: Sustain::Release,
			decay: Decay::Linear(3.0),
			regex: Regex::new("(?i)(?:good (?:girl|kitt(?:y|en))|treat|reward|praise|slut|cum)")
				.unwrap(),
		}
	}

	/// Decay power and send it to the server.
	/// Return indicates if it should sustain.
	fn decay_power(&mut self) -> Power {
		let now = Instant::now();
		let delta = now.duration_since(self.last_update).as_secs_f64();
		self.last_update = now;
		match self.sustain {
			Sustain::Praising => {
				self.sustain = Sustain::Praised;
				self.last_praise = now;
				return Power::Sustain;
			}
			Sustain::Praised => {
				if self.last_praise.elapsed() >= Duration::from_secs(1) {
					self.sustain = Sustain::Release;
				}
				return Power::Sustain;
			}
			Sustain::Release => {
				// Continue
			}
		}
		let current = match self.power {
			Some(c) => c,
			None => return Power::Sustain,
		};
		self.power = get_next_power(current, delta, self.decay);
		match self.power {
			Some(power) => Power::Set(power),
			None => Power::Off,
		}
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
