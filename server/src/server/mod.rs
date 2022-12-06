pub mod error;
pub mod session;

use std::{env, sync::Arc};

use actix_session::{Session, SessionMiddleware};
use error::{Error, Result};

use actix_web::{
	dev::HttpServiceFactory,
	get,
	middleware::Logger,
	post,
	web::{self, Data},
	App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use anyhow::Error as AnyError;
use log::info;
use serde::{Deserialize, Serialize};

use actix_buttplug::ButtplugContext;

use crate::{
	bot::user::ButtplugUser,
	manager::{Manager, User},
};

#[get("/")]
async fn index() -> impl Responder {
	"Hello, there!"
}

#[get("/connect")]
async fn connect(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse> {
	println!("GOT REQUEST");
	let actor = ButtplugUser::new();
	let res = ButtplugContext::start_with_actix_ws_transport(
		actor,
		"Euphoria",
		&req,
		stream,
		|_| async {
			info!("Connected!");
		},
	)
	.await?;
	println!("SENDING RESPONSE");
	Ok(res)
}

#[derive(Deserialize)]
struct Code {
	code: String,
}

#[post("/login")]
async fn login(
	web::Query(Code { code }): web::Query<Code>,
	session: Session,
	manager: Data<Manager>,
) -> Result<HttpResponse> {
	let user = manager.login(&code).await?;
	if let Some(user) = user {
		session.insert("user", user.id.clone())?;
	}
	Ok(HttpResponse::Ok().finish())
}

#[derive(Serialize)]
#[serde(tag = "status", content = "user")]
enum UserLogin {
	LoggedIn(User),
	LoggedOut,
}

#[get("/me")]
async fn get_user_data(
	manager: Data<Manager>,
	ses: session::UserSession,
) -> Result<web::Json<UserLogin>> {
	match ses.get_user(manager.as_ref()).await? {
		Some(user) => Ok(web::Json(UserLogin::LoggedIn(user))),
		None => Ok(web::Json(UserLogin::LoggedOut)),
	}
}

fn endpoints() -> impl HttpServiceFactory {
	web::scope("/api")
		.service(index)
		.service(login)
		.service(connect)
		.service(get_user_data)
}

pub async fn run_http_server(manager: Arc<Manager>) -> Result<(), AnyError> {
	info!("Configuring and starting web server");
	let (store, key) = session::setup_sessions().await;
	HttpServer::new(move || {
		App::new()
			.app_data(Data::from(manager.clone()))
			.wrap(actix_cors::Cors::permissive())
			.wrap(Logger::new("%r %U %s"))
			.wrap(SessionMiddleware::new(store.clone(), key.clone()))
			.service(endpoints())
	})
	.bind(("127.0.0.1", 4000))
	.expect("Failed to bind to port 4000")
	.run()
	.await
	.map_err(AnyError::from)
}
