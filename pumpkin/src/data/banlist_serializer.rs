use std::net::IpAddr;

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct BannedPlayerEntry {
    pub uuid: Uuid,
    pub name: String,
    #[serde(with = "format::date")]
    pub created: DateTime<FixedOffset>,
    pub source: String,
    #[serde(with = "format::option_date")]
    pub expires: Option<DateTime<FixedOffset>>,
    pub reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BannedIpEntry {
    pub ip: IpAddr,
    #[serde(with = "format::date")]
    pub created: DateTime<FixedOffset>,
    pub source: String,
    #[serde(with = "format::option_date")]
    pub expires: Option<DateTime<FixedOffset>>,
    pub reason: String,
}

mod format {
    const FORMAT: &str = "%Y-%m-%d %H:%M:%S %z";

    pub mod date {
        use chrono::{DateTime, FixedOffset};
        use serde::{self, Deserialize, Deserializer, Serializer};

        use super::FORMAT;

        pub fn serialize<S>(date: &DateTime<FixedOffset>, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let s = date.format(FORMAT).to_string();
            serializer.serialize_str(&s)
        }

        pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<FixedOffset>, D::Error>
        where
            D: Deserializer<'de>,
        {
            let s = String::deserialize(deserializer)?;
            DateTime::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
        }
    }

    pub mod option_date {
        use chrono::{DateTime, FixedOffset};
        use serde::{self, Deserialize, Deserializer, Serializer};

        use super::FORMAT;

        #[allow(clippy::ref_option)]
        pub fn serialize<S>(
            date: &Option<DateTime<FixedOffset>>,
            serializer: S,
        ) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            if let Some(date) = date {
                let s = date.format(FORMAT).to_string();
                serializer.serialize_str(&s)
            } else {
                serializer.serialize_str("forever")
            }
        }

        pub fn deserialize<'de, D>(
            deserializer: D,
        ) -> Result<Option<DateTime<FixedOffset>>, D::Error>
        where
            D: Deserializer<'de>,
        {
            let s = String::deserialize(deserializer)?;
            if s == "forever" {
                Ok(None)
            } else {
                DateTime::parse_from_str(&s, FORMAT)
                    .map(Some)
                    .map_err(serde::de::Error::custom)
            }
        }
    }
}
