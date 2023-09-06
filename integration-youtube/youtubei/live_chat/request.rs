use serde::Serialize;

use crate::youtubei::context::Context;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Request<'r> {
    pub context: Context,
    pub continuation: &'r str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_player_state: Option<CurrentPlayerState>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrentPlayerState {
    pub player_offset_ms: String,
}
