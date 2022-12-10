use std::{collections::HashMap, sync::Arc};

use actix::prelude::*;
use actix_buttplug::ButtplugContext;
use buttplug::client::{ButtplugClientDevice, ButtplugClientEvent, VibrateCommand};
use futures::{
	future::{join_all, BoxFuture},
	Future,
};
use log::{error, warn};
use regex::Regex;
use tokio::time::{Duration, Instant};

#[derive(Debug, Clone, Copy)]
pub enum Decay {
	/// Half life in seconds
	HalfLife(f64),
	/// Time it takes to go from 1 to 0 in seconds
	Linear(f64),
}

impl Decay {
	fn decay_power(self, current: f64, delta: f64) -> Option<f64> {
		match self {
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

enum DeviceFrame {
	/// The device either only has one feature, or it has many features with similar characteristics
	Simple {
		device: Arc<ButtplugClientDevice>,
		power: Option<f64>,
		decay_handle: Option<SpawnHandle>,
		step_count: u32,
	},
	// /// The device has a plethora of different features which need to be handled individually
	// Complex {
	// TODO
	// },
}

fn step_count_to_interval(count: u32) -> f64 {
	1.0 / (count as f64)
}

impl DeviceFrame {
	fn set_decay(
		&mut self,
		ctx: &mut ButtplugContext<ButtplugUser>,
		new_power: f64,
		decay: Decay,
	) -> Option<impl Future<Output = ()> + 'static> {
		match self {
			DeviceFrame::Simple {
				device,
				power,
				decay_handle,
				step_count,
			} => {
				if let Some(handle) = decay_handle.take() {
					ctx.cancel_future(handle);
				}
				if new_power < 1e-8 {
					*power = None;
					return None;
				}
				let fut = device.vibrate(&VibrateCommand::Speed(new_power));
				let fut = async move {
					if let Err(e) = fut.await {
						warn!("Failed to vibrate device: {}", e);
					}
				};
				let next_step_power =
					(new_power * *step_count as f64 - 1.0).ceil() / *step_count as f64;
				let idx = device.index();
				let time_til_next_step = match decay {
					Decay::Linear(time) => {
						let power_diff = new_power - next_step_power;
						time * power_diff
					}
					Decay::HalfLife(hl) => {
						let time_til_epsilon = hl * (new_power / 1e-8).log2();
						let time_til_next_hl = hl * (new_power / next_step_power).log2();
						time_til_epsilon.min(time_til_next_hl)
					}
				};
				let handle = ctx.run_later(
					Duration::from_secs_f64(time_til_next_step + 0.0001),
					move |user, ctx| {
						match user.devices.get_mut(&idx) {
							Some(frame) => {
								if let Some(fut) = frame.set_decay(ctx, next_step_power, decay) {
									ctx.spawn(fut.into_actor(user));
								}
							}
							None => return,
						};
					},
				);
				*decay_handle = Some(handle);
				Some(fut)
			}
		}
	}

	fn stop_device(
		&mut self,
		ctx: &mut ButtplugContext<ButtplugUser>,
	) -> impl Future<Output = ()> + 'static {
		match self {
			DeviceFrame::Simple {
				device,
				power,
				decay_handle,
				..
			} => {
				if let Some(handle) = decay_handle.take() {
					ctx.cancel_future(handle);
				}
				let fut = device.stop();
				let fut = async move {
					if let Err(e) = fut.await {
						warn!("Failed to stop device: {}", e);
					}
				};
				*power = None;
				fut
			}
		}
	}
}

pub struct ButtplugUser {
	power: Option<f64>,
	power_instant: Instant,
	devices: HashMap<u32, DeviceFrame>,
	decay: Decay,
	regex: Regex,
}

impl Actor for ButtplugUser {
	type Context = ButtplugContext<Self>;

	fn started(&mut self, ctx: &mut Self::Context) {
		println!("Started actor!");
		let fut = ctx.start_scanning();
		let fut = async move {
			if let Err(e) = fut.await {
				error!("Error starting scanning: {:?}", e)
			}
		};

		ctx.spawn(fut.into_actor(self));

		ctx.devices()
			.into_iter()
			.for_each(|device| self.add_device(ctx, device));
	}
}

impl StreamHandler<ButtplugClientEvent> for ButtplugUser {
	fn handle(&mut self, item: ButtplugClientEvent, ctx: &mut Self::Context) {
		match item {
			ButtplugClientEvent::DeviceAdded(device) => self.add_device(ctx, device),
			ButtplugClientEvent::DeviceRemoved(device) => {
				self.devices.remove(&device.index());
			}
			ButtplugClientEvent::Error(e) => {
				error!("Error: {:?}", e);
			}
			_ => {}
		}
	}
}

impl ButtplugUser {
	pub fn new() -> Self {
		Self {
			power: None,
			power_instant: Instant::now(),
			devices: HashMap::new(),
			decay: Decay::Linear(2.0),
			regex: Regex::new("(?i)(?:good (?:girl|kitt(?:y|en))|treat|reward|praise|slut|cum)")
				.unwrap(),
		}
	}

	fn add_device(&mut self, ctx: &mut ButtplugContext<Self>, device: Arc<ButtplugClientDevice>) {
		let attributes = device.message_attributes();
		if let Some(attrs) = attributes.scalar_cmd() {
			let same_count = attrs
				.windows(2)
				.all(|w| w[0].step_count() == w[1].step_count());
			match same_count {
				true => {
					let frame = DeviceFrame::Simple {
						device: device.clone(),
						decay_handle: None,
						power: None,
						step_count: *attrs[0].step_count(),
					};
					self.devices.insert(device.index(), frame);
				}
				false => {
					warn!("Non uniform step counts not yet supported: {:?}", device);
				}
			}
		} else {
			warn!("Non scalar devices not yet supported: {:?}", device);
		}
	}

	fn set_power(&mut self, ctx: &mut ButtplugContext<Self>, power: f64) {
		self.power = Some(power);
		self.power_instant = Instant::now();
		let decay = self.decay;
		let futs = self
			.devices
			.values_mut()
			.filter_map(|d| d.set_decay(ctx, power, decay));
		let fut = join_all(futs);
		let fut = async {
			fut.await;
		};
		ctx.spawn(fut.into_actor(self));
	}

	fn stop_devices(&mut self, ctx: &mut ButtplugContext<Self>) {
		self.power = None;
		let futs = self.devices.values_mut().map(|d| d.stop_device(ctx));
		let fut = join_all(futs);
		let fut = async {
			fut.await;
		};
		ctx.spawn(fut.into_actor(self));
	}
}

pub struct Flirt(pub String);

impl Message for Flirt {
	type Result = ();
}

impl Handler<Flirt> for ButtplugUser {
	type Result = ();

	fn handle(&mut self, msg: Flirt, ctx: &mut Self::Context) -> Self::Result {
		if self.regex.is_match(&msg.0) {
			let new_power = self.power.unwrap_or(0.0) + 0.3;
			self.power = Some(new_power);
			self.power_instant = Instant::now();
			self.set_power(ctx, new_power);
		}
	}
}

pub struct Reaction;

impl Message for Reaction {
	type Result = ();
}

impl Handler<Reaction> for ButtplugUser {
	type Result = ();

	fn handle(&mut self, _msg: Reaction, ctx: &mut Self::Context) -> Self::Result {
		let new_power = self.power.unwrap_or(0.0) + 0.3;
		self.power = Some(new_power);
		self.power_instant = Instant::now();
		self.set_power(ctx, new_power);
	}
}

pub struct SetDecay(pub Decay);

impl Message for SetDecay {
	type Result = ();
}

impl Handler<SetDecay> for ButtplugUser {
	type Result = ();

	fn handle(&mut self, msg: SetDecay, _ctx: &mut Self::Context) -> Self::Result {
		if let Some(last_power) = self.power {
			let now = Instant::now();
			let delta = now.duration_since(self.power_instant).as_secs_f64();
			self.power_instant = now;
			let new_power = self.decay.decay_power(last_power, delta);
			self.power = new_power;
		}
		self.decay = msg.0;
	}
}
