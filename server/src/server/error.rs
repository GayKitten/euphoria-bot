use actix_buttplug::error::Error as ABError;
use actix_session::SessionGetError;
use actix_web::{HttpResponse, ResponseError};
use thiserror::Error;

use log::error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
	#[error("actix-buttplug error: {0}")]
	ABError(#[from] ABError),
	#[error("Sqlx error: {0}")]
	SqlxError(#[from] sqlx::Error),
	#[error("Invalid auth code passed")]
	BadCode,
	#[error("Session get error: {0}")]
	SessionGetError(#[from] actix_session::SessionGetError),
	#[error("Session insert error: {0}")]
	SessionInsertError(#[from] actix_session::SessionInsertError),
}

impl ResponseError for Error {
	fn error_response(&self) -> actix_web::HttpResponse {
		match self {
			Error::ABError(_)
			| Error::SqlxError(_)
			| Error::SessionGetError(_)
			| Error::SessionInsertError(_) => {
				error!("Internal server error: {:?}", self);
				HttpResponse::InternalServerError().finish()
			}
			Error::BadCode => HttpResponse::BadRequest().body("Invalid auth code passed"),
		}
	}
}
