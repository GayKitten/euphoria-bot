use actix::Addr;
use dashmap::DashMap;
use twilight_model::id::{marker::UserMarker, Id};

use crate::user::ButtplugUser;

#[derive(Default)]
pub struct UserManager {
	map: DashMap<Id<UserMarker>, Addr<ButtplugUser>>,
}

impl UserManager {
	pub fn insert(&self, id: Id<UserMarker>, addr: Addr<ButtplugUser>) {
		self.map.insert(id, addr);
	}

	pub fn get(&self, id: Id<UserMarker>) -> Option<Addr<ButtplugUser>> {
		self.map.get(&id).map(|v| v.value().clone())
	}
}
