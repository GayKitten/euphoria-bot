mod server;
mod bot;
mod database;
mod regex;

use bot::run_bot;
use futures::TryFutureExt;
use server::run_http_server;
#[macro_use]
extern crate lazy_static;
use color_eyre::Result;
use dotenv::dotenv;
use tokio::task::{LocalSet};




fn main() -> Result<()> {
	color_eyre::install()?;

	dotenv().ok();


	let rt = tokio::runtime::Builder::new_multi_thread()
		.worker_threads(8)
		.enable_all()
		.build()
		.expect("Couldn't start runtime");

	rt.block_on(async move {
		let set = LocalSet::new();
		let server = set.spawn_local(run_http_server()).map_err(anyhow::Error::from);

		let bot = tokio::spawn(run_bot()).map_err(anyhow::Error::from);


		if let Err(why) = futures::try_join!(server, bot) {
			println!("Backend failed: {:#?}", why);
		}

	});

	Ok(())
}
