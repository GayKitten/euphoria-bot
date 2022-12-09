mod bot;
mod manager;
mod regex;
mod server;
mod user;

use std::sync::Arc;

use bot::run_bot;
use color_eyre::Result;
use dotenv::dotenv;
use futures::TryFutureExt;
use log::{error, info};
use server::run_http_server;
use tokio::{sync::Notify, task::LocalSet};

fn init_logger() {
	pretty_env_logger::formatted_builder()
		.filter_level(log::LevelFilter::Info)
		.filter(Some("tracing::span"), log::LevelFilter::Warn)
		.filter(Some("serenity"), log::LevelFilter::Warn)
		.init();
}

fn main() -> Result<()> {
	init_logger();
	color_eyre::install()?;
	dotenv().ok();

	let notify_term = Arc::new(Notify::new());

	let rt = tokio::runtime::Builder::new_multi_thread()
		.worker_threads(8)
		.enable_all()
		.build()
		.expect("Couldn't start runtime");
	info!("Created runtime");

	let term = notify_term.clone();

	rt.spawn(async move {
		if let Err(e) = tokio::signal::ctrl_c().await {
			error!("Error waiting for ctrl-c: {}", e);
		}
		term.notify_waiters();
	});

	rt.block_on(async move {
		let manager = Arc::new(manager::Manager::new().await);

		let server_set = LocalSet::new();
		let server = server_set
			.spawn_local(run_http_server(manager.clone()))
			.map_err(anyhow::Error::from);

		let discord_set = LocalSet::new();
		let bot = discord_set
			.spawn_local(run_bot(notify_term))
			.map_err(anyhow::Error::from);

		info!("Starting backend");

		futures::join!(server_set, discord_set);
	});
	Ok(())
}
