use serde::Serialize;

#[derive(Serialize)]
pub struct Context {
    pub client: Client,
}

#[derive(Serialize)]
pub struct Client {
    #[serde(rename = "hl")]
    pub language: String,
    #[serde(rename = "clientName")]
    pub client_name: String,
    #[serde(rename = "clientVersion")]
    pub client_version: String,
}

impl Context {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Context {
            client: Client {
                language: "en".into(),
                client_name: std::env::var("INNERTUBE_CLIENT_NAME")?,
                client_version: std::env::var("INNERTUBE_CLIENT_VERSION")?,
            },
        })
    }
}
