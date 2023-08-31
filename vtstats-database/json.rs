use serde::de::DeserializeOwned;
use serde_json::Value;
use sqlx::{Error, Result};

pub fn decode_json_value<T: DeserializeOwned>(value: Value) -> Result<T> {
    serde_json::from_value(value).map_err(|err| Error::Decode(Box::new(err)))
}
