mod channels;
pub mod chat;
mod metadata;
mod otel;
mod pubsub;
mod rss;
mod streams;
pub mod telegram;
mod thumbnail;
mod upload;

use reqwest::Client;

pub use channels::*;
pub use chat::*;
pub use streams::*;

#[derive(Clone)]
pub struct RequestHub {
    pub client: Client,
}

impl RequestHub {
    pub fn new() -> Self {
        RequestHub {
            client: Client::new(),
        }
    }
}

impl Default for RequestHub {
    fn default() -> Self {
        RequestHub::new()
    }
}

pub use reqwest::Error as RequestError;
