mod thumbnail;

use reqwest::{Client, ClientBuilder};

#[derive(Clone)]
pub struct RequestHub {
    pub client: Client,
}

impl RequestHub {
    pub fn new() -> Self {
        RequestHub {
            client: ClientBuilder::new()
                .brotli(true)
                .deflate(true)
                .gzip(true)
                .build()
                .unwrap(),
        }
    }
}

impl Default for RequestHub {
    fn default() -> Self {
        RequestHub::new()
    }
}

pub use reqwest::Error as RequestError;
