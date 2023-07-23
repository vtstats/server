use serde::Serialize;

use crate::youtubei::context::Context;

#[derive(Serialize)]
pub struct Request<'r> {
    pub context: Context,
    pub continuation: &'r str,
}
