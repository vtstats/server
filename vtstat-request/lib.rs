mod channels;
pub mod telegram;
mod thumbnail;

use reqwest::Client;

pub use channels::*;
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
