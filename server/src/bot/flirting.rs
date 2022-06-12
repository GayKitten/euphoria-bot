use std::{collections::{HashMap, HashSet}, time::{Instant, Duration}};

use serenity::model::id::UserId;

/// If someone hasn't flirted inside the flirt window, the next implicit flirt doesn't count.
const FLIRT_WINDOW: Duration = Duration::from_secs(300);

pub struct ChannelContext {
	users: HashMap<UserId, FlirtingUser>
}

/// After a user has done an explicit flirt, they may implicitly flirt after for a while.
pub struct FlirtingUser {
	/// Last time the user has flirted.
	/// After a while of no flirting, the implicit flirting should expire.
	last_flirt: Instant,
	/// Users for which implicit flirts will affect.
	flirting_with: Vec<UserId>
}

impl ChannelContext {

}

impl FlirtingUser {
	pub fn new() {

	}

	pub fn is_valid_flirt(&self, now: Instant) -> bool {
		self.last_flirt + FLIRT_WINDOW > now
	}
}

