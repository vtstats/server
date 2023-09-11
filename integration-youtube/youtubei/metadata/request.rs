use serde::Serialize;

use crate::youtubei::context::Context;

#[derive(Serialize)]
pub(crate) struct Request<'r> {
    pub context: Context,
    pub continuation: &'r str,
}
