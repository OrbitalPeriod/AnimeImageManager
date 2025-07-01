use serde::{Deserialize, Deserializer};
use std::str::FromStr;

use crate::database::Rating;

#[derive(Debug, Deserialize, Copy, Clone)]
pub struct Paginated {
    #[serde(default, deserialize_with = "option_from_str_or_number")]
    pub page: Option<u32>,
    #[serde(default, deserialize_with = "option_from_str_or_number")]
    pub per_page: Option<u32>,
}

fn option_from_str_or_number<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr + serde::Deserialize<'de>,
    T::Err: std::fmt::Display,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    match opt {
        Some(s) => {
            let parsed = s.parse::<T>().map_err(serde::de::Error::custom)?;
            Ok(Some(parsed))
        }
        None => Ok(None),
    }
}

#[derive(Debug, Deserialize)]
pub struct ImageRequest {
    pub token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FindImageRequest {
    pub characters: Option<String>,
    pub tags: Option<String>,
    pub rating: Option<Rating>,
    pub token: Option<String>,
    #[serde(flatten)]
    pub pages: Paginated,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FindTagQuery {
    pub tag: Option<String>,
    #[serde(flatten)]
    pub pages: Paginated,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FindCharacterQuery{
    pub character: Option<String>,
    #[serde(flatten)]
    pub pages: Paginated,
}
