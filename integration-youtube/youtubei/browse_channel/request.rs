use serde::Serialize;

use crate::youtubei::context::Context;

#[derive(Serialize)]
pub struct Request<'r> {
    pub context: Context,
    #[serde(rename = "browseId")]
    pub browse_id: &'r str,
}
