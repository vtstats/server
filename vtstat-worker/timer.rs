use chrono::{DateTime, Duration, Timelike, Utc};

pub enum Calendar {
    Daily,
    Hourly,
    FifteenSeconds,
}

impl Calendar {
    #[inline(always)]
    fn to_seconds(self) -> i64 {
        match self {
            Calendar::Daily => 24 * 60 * 60,
            Calendar::Hourly => 60 * 60,
            Calendar::FifteenSeconds => 15,
        }
    }
}

pub fn timer(calendar: Calendar) -> (DateTime<Utc>, DateTime<Utc>) {
    let now = Utc::now();

    let span = calendar.to_seconds();

    let current = trunc_secs(now, span as u32);

    (current, current + Duration::seconds(span))
}

pub fn trunc_secs(dt: DateTime<Utc>, span: u32) -> DateTime<Utc> {
    let delta_down = dt.second() % span;

    if delta_down > 0 {
        dt - chrono::Duration::seconds(delta_down.into())
    } else {
        dt // unchanged
    }
}
