use chrono::serde::ts_seconds;
use chrono::{DateTime, Utc};
use chrono_tz::{Tz, US::Eastern};
use serde::{Deserializer, Serializer};

pub fn deserialize<'de, D>(d: D) -> Result<DateTime<Tz>, D::Error>
where
    D: Deserializer<'de>,
{
    ts_seconds::deserialize(d).map(|res| res.with_timezone(&Eastern))
}
pub fn serialize<S>(dt: &DateTime<Tz>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    ts_seconds::serialize(&dt.with_timezone(&Utc), serializer)
}
