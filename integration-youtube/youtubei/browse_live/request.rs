use serde::Serialize;

use crate::youtubei::context::Context;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Request<'r> {
    pub context: Context,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub browse_id: Option<&'r str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<&'r str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continuation: Option<&'r str>,
}
