use serde::Serialize;

use crate::youtubei::context::Context;

#[derive(Serialize)]
pub struct Request<'r> {
    pub context: Context,
    #[serde(rename = "videoId")]
    pub video_id: &'r str,
}
