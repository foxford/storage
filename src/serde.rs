use serde::de;
use serde::de::{Deserializer, Error, SeqAccess, Unexpected, Visitor};
use std::fmt;
use std::time::Duration;
use tower_web::middleware::cors::AllowedOrigins;

////////////////////////////////////////////////////////////////////////////////

struct DurationVisitor;

impl<'de> Visitor<'de> for DurationVisitor {
    type Value = Duration;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "an u64")
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Duration::new(v, 0))
    }
}

pub(crate) fn duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_u64(DurationVisitor)
}

////////////////////////////////////////////////////////////////////////////////

struct AllowedOriginsVisitor;

impl<'de> Visitor<'de> for AllowedOriginsVisitor {
    type Value = AllowedOrigins;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a list of strings or '*' character")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match v {
            "*" => Ok(AllowedOrigins::Any { allow_null: true }),
            _ => Err(Error::invalid_value(Unexpected::Str(v), &self)),
        }
    }

    #[allow(clippy::mutable_key_type)]
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        use http::header::HeaderValue;
        use std::collections::BTreeSet;

        let mut origins = BTreeSet::new();
        while let Some(value) = seq.next_element()? {
            let value: String = value;
            let _ = HeaderValue::from_str(&value).map(|v| origins.insert(v));
        }
        Ok(AllowedOrigins::Origins(origins))
    }
}

pub(crate) fn allowed_origins<'de, D>(deserializer: D) -> Result<AllowedOrigins, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_seq(AllowedOriginsVisitor)
}
