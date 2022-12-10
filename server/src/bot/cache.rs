use std::sync::Arc;

use anyhow::Result;
use dashmap::DashMap;
use twilight_cache_inmemory::InMemoryCache;
use twilight_http::Client;
use twilight_model::id::{
	marker::{ChannelMarker, MessageMarker, UserMarker},
	Id,
};

pub struct Cache {
	cache: Arc<InMemoryCache>,
	client: Arc<Client>,
	message_author_map: DashMap<Id<MessageMarker>, Id<UserMarker>>,
}

impl Cache {
	pub fn new(cache: Arc<InMemoryCache>, client: Arc<Client>) -> Self {
		Self {
			cache,
			message_author_map: Default::default(),
			client,
		}
	}

	pub async fn get_author(
		&self,
		message_id: Id<MessageMarker>,
		channel_id: Id<ChannelMarker>,
	) -> Result<Id<UserMarker>> {
		if let Some(author) = self.message_author_map.get(&message_id) {
			return Ok(*author.value());
		}

		if let Some(entry) = self.cache.message(message_id) {
			let author = entry.author();
			self.message_author_map.insert(message_id, author);
			return Ok(author);
		}

		let author = self
			.client
			.message(channel_id, message_id)
			.await?
			.model()
			.await?
			.author
			.id;
		self.message_author_map.insert(message_id, author);
		Ok(author)
	}
}
