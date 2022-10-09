use chrono::{DateTime, Utc};
use sqlx::{postgres::PgQueryResult, types::Json, PgPool, Postgres, QueryBuilder, Result};

use super::{DonationKind, DonationValue};

pub struct AddDonationQuery {
    pub stream_id: i32,
    pub rows: Vec<AddDonationRow>,
}

pub struct AddDonationRow {
    pub time: DateTime<Utc>,
    pub value: DonationValue,
}

impl AddDonationQuery {
    pub async fn execute(self, pool: &PgPool) -> Result<PgQueryResult> {
        let mut query_builder: QueryBuilder<Postgres> =
            QueryBuilder::new("INSERT INTO donations (stream_id, time, kind, value) ");

        query_builder.push_values(self.rows.iter(), |mut b, row| {
            let kind = match row.value {
                DonationValue::YoutubeSuperChat { .. } => DonationKind::YoutubeSuperChat,
                DonationValue::YoutubeSuperSticker { .. } => DonationKind::YoutubeSuperSticker,
                DonationValue::YoutubeNewMember { .. } => DonationKind::YoutubeNewMember,
                DonationValue::YoutubeMemberMilestone { .. } => {
                    DonationKind::YoutubeMemberMilestone
                }
            };

            b.push_bind(self.stream_id)
                .push_bind(row.time)
                .push_bind(kind)
                .push_bind(Json(&row.value));
        });

        let query = query_builder.build().execute(pool);

        crate::otel::instrument("INSERT", "donations", query).await
    }
}

// TODO: add unit tests
