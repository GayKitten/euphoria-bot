use std::convert::Infallible;
use std::env;

use actix::fut::{ready, Ready};
use actix_session::storage::RedisSessionStore;
use actix_session::{Session, SessionExt};
use actix_web::cookie::Key;
use actix_web::{dev::Payload, FromRequest, HttpRequest};

use crate::manager::{Manager, User};

use super::error::Result;

pub async fn setup_sessions() -> (RedisSessionStore, Key) {
	let redis_uri = env::var("REDIS_URI").expect("REDIS_URI not set");
	let store = RedisSessionStore::builder(redis_uri)
		.cache_keygen(|key| {
			let mut key = key.to_string();
			key.push_str(":session");
			key
		})
		.build()
		.await
		.expect("Couldn't connect to Redis");
	let key = Key::generate();
	(store, key)
}

pub struct UserSession(pub Session);

impl UserSession {
	pub fn get_id(&self) -> Result<Option<String>> {
		Ok(self.0.get::<String>("user")?)
	}

	pub async fn get_user(&self, manager: &Manager) -> Result<Option<User>> {
		let id = match self.0.get::<String>("user")? {
			Some(id) => id,
			None => return Ok(None),
		};
		let user = manager.get_user(&id).await?;
		Ok(user)
	}
}

impl FromRequest for UserSession {
	type Error = Infallible;
	type Future = Ready<Result<Self, Self::Error>>;
	fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
		let session = req.get_session();
		ready(Ok(UserSession(session)))
	}
}
