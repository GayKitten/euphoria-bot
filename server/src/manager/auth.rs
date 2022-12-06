use std::env;

use super::User;

use log::warn;
use serde::{Deserialize, Serialize};

pub struct Auth {
	pub client: reqwest::Client,
	client_id: String,
	client_secret: String,
	redirect_uri: String,
}

#[derive(Serialize)]
pub struct ExchangeCode<'data> {
	client_id: &'data str,
	client_secret: &'data str,
	code: &'data str,
	grant_type: &'static str,
	redirect_uri: &'data str,
}

#[derive(Serialize)]
pub struct RefreshCode<'data> {
	client_id: &'data str,
	client_secret: &'data str,
	refresh_token: &'data str,
	grant_type: &'static str,
}

#[derive(Deserialize)]
pub struct AccessToken {
	pub access_token: String,
	pub token_type: String,
	pub expires_in: u64,
	pub refresh_token: String,
}

impl Auth {
	pub fn new() -> Self {
		let client_id = env::var("CLIENT_ID").expect("CLIENT_ID not set");
		let client_secret = env::var("CLIENT_SECRET").expect("CLIENT_SECRET not set");
		let redirect_uri = env::var("REDIRECT_URI").expect("REDIRECT_URI not set");
		Auth {
			client: reqwest::Client::new(),
			client_id,
			client_secret,
			redirect_uri,
		}
	}

	pub async fn exchange_code(&self, code: &str) -> Option<AccessToken> {
		let form = ExchangeCode {
			client_id: &self.client_id,
			client_secret: &self.client_secret,
			code,
			grant_type: "authorization_code",
			redirect_uri: &self.redirect_uri,
		};
		let res = self
			.client
			.post("https://discordapp.com/api/oauth2/token")
			.form(&form)
			.send()
			.await
			.expect("Failed to exchange code");
		if res.status().is_success() {
			let token = res.json::<AccessToken>().await.unwrap();
			Some(token)
		} else {
			warn!("Failed to exchange code: {}", res.status());
			None
		}
	}

	pub async fn refresh_token(&self, refresh_token: &str) -> AccessToken {
		let form = RefreshCode {
			client_id: &self.client_id,
			client_secret: &self.client_secret,
			refresh_token,
			grant_type: "refresh_token",
		};
		self.client
			.post("https://discordapp.com/api/oauth2/token")
			.form(&form)
			.send()
			.await
			.expect("Failed to refresh token")
			.json::<AccessToken>()
			.await
			.expect("Failed to parse access token")
	}

	pub async fn get_user(&self, access_token: &str) -> User {
		let res = self
			.client
			.get("https://discordapp.com/api/users/@me")
			.bearer_auth(access_token)
			.send()
			.await
			.expect("Failed to get user");
		res.json::<User>().await.expect("Failed to parse user")
	}
}
