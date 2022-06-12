use std::env;

use anyhow::Error;
use actix_web::{get, Responder, App, HttpServer, middleware::Logger, post, web::{self, Data}};
use oauth2::{basic::BasicClient, ClientId, ClientSecret, AuthUrl, TokenUrl, RedirectUrl, AuthorizationCode, reqwest::async_http_client};
use serde::Deserialize;
//use oauth2::

#[get("/")]
async fn index() -> impl Responder {
	"Hello, there!"
}

#[derive(Deserialize)]
struct Code {
	code: String,
}

#[post("/login")]
async fn login(code: web::Json<Code>, oauth_client: Data<BasicClient>) -> impl Responder {
	println!("code: {}", code.code);
	let code = code.into_inner().code;
	let res = oauth_client.exchange_code(AuthorizationCode::new(code))
		.request_async(async_http_client)
		.await;
	"oke"
}

pub async fn run_http_server() -> Result<(), Error> {
	println!("running server");
	HttpServer::new(||{
		let oauth_client = BasicClient::new(
			ClientId::new(env::var("CLIENT_ID").expect("missing CLIENT_ID var")),
			Some(ClientSecret::new(env::var("CLIENT_SECRET").expect("missing CLIENT_SECRET var"))),
			AuthUrl::new("https://discord.com/api/oauth2/authorize".to_string()).unwrap(),
			Some(TokenUrl::new("https://discord.com/api/oauth2/token".to_string()).unwrap())
		).set_redirect_uri(RedirectUrl::new(env::var("REDIRECT_URL").expect("missing REDIRECT_URL var")).unwrap());

		App::new()
			.app_data(oauth_client)
			.wrap(Logger::default())
			.service(index)
			.service(login)
	})
	.bind(("0.0.0.0", 4000)).expect("Failed to create server.")
	.run()
	.await
	.map_err(Error::from)
}