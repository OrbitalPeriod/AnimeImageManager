use serde::Deserialize;

use crate::database::Rating;

#[derive(Debug, Deserialize)]
pub struct ImageRequest {
    pub token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FindImageRequest {
    pub characters: Option<String>,
    pub tags: Option<String>,
    pub rating: Option<Rating>,
    pub page: Option<i32>,
    pub per_page: Option<u32>,
    pub token: Option<String>,
}


