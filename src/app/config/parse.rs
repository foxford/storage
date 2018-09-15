use jose::Algorithm;
use serde::de;
use serde::de::{Deserializer, Error, SeqAccess, Unexpected, Visitor};
use std::fmt;
use std::time::Duration;
use tower_web::middleware::cors::AllowedOrigins;

struct AlgorithmVisitor;

impl<'de> Visitor<'de> for AlgorithmVisitor {
    type Value = Algorithm;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "a name of signature or MAC algorithm specified in RFC7518: JSON Web Algorithms (JWA)"
        )
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        use std::str::FromStr;

        Algorithm::from_str(v).map_err(|_| Error::invalid_value(Unexpected::Str(v), &self))
    }
}

pub(crate) fn algorithm<'de, D>(deserializer: D) -> Result<Algorithm, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_str(AlgorithmVisitor)
}

struct FileVisitor;

impl<'de> Visitor<'de> for FileVisitor {
    type Value = Vec<u8>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a path to an existing file")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        use std::fs::File;
        use std::io::Read;

        let mut data = Vec::new();
        File::open(v)
            .and_then(|mut file| file.read_to_end(&mut data).and_then(|_| Ok(data)))
            .map_err(|_| Error::invalid_value(Unexpected::Str(v), &self))
    }
}

pub(crate) fn file<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_str(FileVisitor)
}

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
