use std::env;

use chrono::{Duration, Utc};
use sqlx::PgPool;

use super::{auth::AccessToken, User};

pub type Result<T, E = sqlx::error::Error> = std::result::Result<T, E>;

pub struct EuphoriaDB {
	pub pool: PgPool,
}

impl EuphoriaDB {
	pub async fn new() -> Self {
		Self {
			pool: PgPool::connect(
				env::var("DATABASE_URL")
					.expect("DATABASE_URL not set")
					.as_ref(),
			)
			.await
			.expect("Couldn't connect to the database"),
		}
	}

	pub async fn get_user(&self, id: &str) -> Result<Option<User>> {
		sqlx::query_as!(
			User,
			"SELECT id, username, avatar FROM users WHERE id = $1",
			id
		)
		.fetch_optional(&self.pool)
		.await
	}

	pub async fn save_user(&self, user: &User, token: &AccessToken) -> Result<()> {
		// first check if user id is already in database
		let user_id = user.id.as_str();
		let user_exists =
			sqlx::query!("SELECT EXISTS( SELECT 1 FROM users WHERE id = $1)", user_id)
				.map(|r| r.exists.unwrap_or(false))
				.fetch_one(&self.pool)
				.await?;
		if user_exists {
			let expires_at = Utc::now().naive_local() + Duration::seconds(token.expires_in as i64);
			sqlx::query!(
				"
			UPDATE users
			SET
				access_token = $1,
				refresh_token = $2,
				expires_at = $3
			WHERE id = $4
			",
				token.access_token,
				token.refresh_token,
				expires_at,
				user_id
			)
			.execute(&self.pool)
			.await?;
		} else {
			self.new_user(user, token).await?;
		}
		Ok(())
	}

	async fn new_user(&self, user: &User, token: &AccessToken) -> Result<(), sqlx::Error> {
		sqlx::query!(
			"INSERT INTO users (id, username, avatar, access_token, expires_at, refresh_token)
			VALUES ($1, $2, $3, $4, $5, $6)",
			user.id,
			user.username,
			user.avatar,
			token.access_token,
			Utc::now().naive_local() + Duration::seconds(token.expires_in as i64),
			token.refresh_token,
		)
		.execute(&self.pool)
		.await?;
		Ok(())
	}
}
