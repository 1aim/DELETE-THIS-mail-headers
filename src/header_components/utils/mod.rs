

pub mod text_partition;

#[cfg(feature="serde-impl")]
use serde::{
    Serializer, Deserializer,
    Deserialize,
    de::Error
};
#[cfg(feature="serde-impl")]
use chrono::{
    self, Utc
};

#[cfg(feature="serde-impl")]
pub fn deserialize_time<'de, D>(deserializer: D)
    -> Result<chrono::DateTime<Utc>, D::Error>
    where D: Deserializer<'de>
{
    let as_string = String::deserialize(deserializer)?;
    let date_time = chrono::DateTime::parse_from_rfc2822(&as_string)
        .map_err(|e| D::Error::custom(format!(
            "invalid rfc2822 date time: {}", e
        )))?;

    Ok(date_time.with_timezone(&Utc))
}

#[cfg(feature="serde-impl")]
pub fn deserialize_opt_time<'de, D>(deserializer: D)
    -> Result<Option<chrono::DateTime<Utc>>, D::Error>
    where D: Deserializer<'de>
{
    let opt_string = <Option<String>>::deserialize(deserializer)?;
    if let Some(as_string) = opt_string {
        let date_time = chrono::DateTime::parse_from_rfc2822(&as_string)
            .map_err(|e| D::Error::custom(format!(
                "invalid rfc2822 date time: {}", e
            )))?;
        Ok(Some(date_time.with_timezone(&Utc)))
    } else {
        Ok(None)
    }
}

#[cfg(feature="serde-impl")]
pub fn serialize_time<S>(dt: &chrono::DateTime<Utc>, serializer: S)
    -> Result<S::Ok, S::Error>
    where S: Serializer
{
    serializer.serialize_str(&dt.to_rfc2822())
}

#[cfg(feature="serde-impl")]
pub fn serialize_opt_time<S>(
    dt: &Option<chrono::DateTime<Utc>>,
    serializer: S
) -> Result<S::Ok, S::Error>
    where S: Serializer
{
    if let Some(time) = dt.as_ref() {
        serializer.serialize_str(&time.to_rfc2822())
    } else {
        serializer.serialize_none()
    }
}