use actix::Addr;
use serde::{Deserialize, Serialize};
use twilight_model::id::{marker::UserMarker, Id};

use crate::user::ButtplugUser;

mod auth;
mod database;
mod users;

#[derive(Deserialize, Serialize, Debug)]
pub struct User {
	pub id: String,
	pub username: String,
	pub avatar: String,
}

pub struct Manager {
	pub auth: auth::Auth,
	pub db: database::EuphoriaDB,
	pub user_manager: users::UserManager,
}

impl Manager {
	pub async fn new() -> Self {
		Self {
			auth: auth::Auth::new(),
			db: database::EuphoriaDB::new().await,
			user_manager: Default::default(),
		}
	}
}

/// auth impls
impl Manager {
	pub async fn login(&self, code: &str) -> database::Result<Option<User>> {
		let token = match self.auth.exchange_code(code).await {
			Some(token) => token,
			None => return Ok(None),
		};
		let user = self.auth.get_user(&token.access_token).await;
		self.db.save_user(&user, &token).await?;
		Ok(Some(user))
	}

	pub async fn get_user(&self, id: &str) -> database::Result<Option<User>> {
		self.db.get_user(id).await
	}
}

/// user impls
impl Manager {
	pub fn insert(&self, id: Id<UserMarker>, addr: Addr<ButtplugUser>) {
		self.user_manager.insert(id, addr);
	}

	pub fn get(&self, id: Id<UserMarker>) -> Option<Addr<ButtplugUser>> {
		self.user_manager.get(id)
	}
}
