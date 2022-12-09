use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
/// The pattern with which one may be flirted with.
pub enum FlirtPattern {
	/// A regex to use as a
	Regex(String),
	Words(Vec<String>),
}

impl FlirtPattern {
	pub fn to_regex(&self) -> regex::Regex {
		match self {
			FlirtPattern::Regex(regex) => regex::Regex::new(regex).unwrap(),
			FlirtPattern::Words(words) => {
				let regex = words.join("|");
				regex::Regex::new(&regex).unwrap()
			}
		}
	}
}
