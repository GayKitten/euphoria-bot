use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag="type", rename_all="snake_case")]
/// The pattern with which one may be flirted with.
pub enum FlirtPattern {
	/// A regex to use as a 
	Regex(String),
	Words(Vec<String>),
}