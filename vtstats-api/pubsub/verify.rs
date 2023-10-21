use axum::extract::Query;

#[derive(serde::Deserialize)]
pub struct VerifyIntentRequestQuery {
    #[serde(rename = "hub.challenge")]
    challenge: String,
}

pub async fn verify_intent(Query(query): Query<VerifyIntentRequestQuery>) -> String {
    tracing::debug!("challenge={}", query.challenge);
    query.challenge
}
